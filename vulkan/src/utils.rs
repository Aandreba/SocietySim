use std::{io::{Read, ErrorKind}, ptr::NonNull, sync::{atomic::{AtomicUsize, Ordering}}, alloc::Layout, ops::{Range}};
use docfg::docfg;
use futures::{AsyncRead, AsyncReadExt};

/// Casts `usize` to `u32`
/// 
/// # Panics
/// This method will panic when, in debug mode, the `usize` value doesn't fit inside a `u32`.
/// In release mode, the value will be truncated.
#[inline(always)]
pub fn usize_to_u32 (v: usize) -> u32 {
    #[cfg(debug_assertions)]
    return u32::try_from(v).unwrap();
    #[cfg(not(debug_assertions))]
    return v as u32
}


/// Casts `u64` to `u32`
/// 
/// # Panics
/// This method will panic when, in debug mode, the `u64` value doesn't fit inside a `u32`.
/// In release mode, the value will be truncated.
#[inline(always)]
pub fn u64_to_u32 (v: u64) -> u32 {
    #[cfg(debug_assertions)]
    return u32::try_from(v).unwrap();
    #[cfg(not(debug_assertions))]
    return v as u32
}

/// Casts `u64` to `usize`
/// 
/// # Panics
/// This method will panic when, in debug mode, the `u64` value doesn't fit inside a `usize`.
/// In release mode, the value will be truncated.
#[inline(always)]
pub fn u64_to_usize (v: u64) -> usize {
    #[cfg(debug_assertions)]
    return usize::try_from(v).unwrap();
    #[cfg(not(debug_assertions))]
    return v as usize
}

#[inline]
pub fn read_spv<R: ?Sized + Read> (r: &mut R) -> std::io::Result<Vec<u32>> {
    const WORD_SIZE: usize = core::mem::size_of::<u32>();

    let mut words = Vec::<u32>::new();
    loop {
        let len = words.len();
        words.reserve(1);

        let bytes = unsafe {
            core::slice::from_raw_parts_mut(words.as_mut_ptr().add(len).cast::<u8>(), WORD_SIZE)
        };

        match r.read(bytes)? {
            WORD_SIZE => unsafe { words.set_len(len + 1) },
            0 => break,
            _ => unreachable!()
        }
    }
    
    return Ok(words)
}

macro_rules! read_spv_async {
    (|$b:ident| $f:expr, $endian:expr) => {{
        const WORD_SIZE: usize = core::mem::size_of::<u32>();

        let mut words = Vec::<u32>::new();
        let mut $b = [0; WORD_SIZE];

        loop {
            match ($f).await? {
                WORD_SIZE => words.push($endian($b)),
                0 => break,
                _ => return Err(std::io::Error::from(ErrorKind::UnexpectedEof)),
            }
        }
        
        return Ok(words)
    }}
}

pub async fn read_spv_futures_le<R: ?Sized + Unpin + AsyncRead> (r: &mut R) -> std::io::Result<Vec<u32>> {
    read_spv_async!(|b| r.read(&mut b), u32::from_le_bytes)
}

pub async fn read_spv_futures_be<R: ?Sized + Unpin + AsyncRead> (r: &mut R) -> std::io::Result<Vec<u32>> {
    read_spv_async!(|b| r.read(&mut b), u32::from_be_bytes)
}

pub async fn read_spv_futures<R: ?Sized + Unpin + AsyncRead> (r: &mut R) -> std::io::Result<Vec<u32>> {
    read_spv_async!(|b| r.read(&mut b), u32::from_ne_bytes)
}

#[docfg(feature = "tokio")]
pub async fn read_spv_tokio_le<R: ?Sized + Unpin + tokio::io::AsyncRead> (r: &mut R) -> std::io::Result<Vec<u32>> {
    read_spv_async!(|b| tokio::io::AsyncReadExt::read(r, &mut b), u32::from_le_bytes)
}

#[docfg(feature = "tokio")]
pub async fn read_spv_tokio_be<R: ?Sized + Unpin + tokio::io::AsyncRead> (r: &mut R) -> std::io::Result<Vec<u32>> {
    read_spv_async!(|b| tokio::io::AsyncReadExt::read(r, &mut b), u32::from_be_bytes)
}

#[docfg(feature = "tokio")]
pub async fn read_spv_tokio<R: ?Sized + Unpin + tokio::io::AsyncRead> (r: &mut R) -> std::io::Result<Vec<u32>> {
    read_spv_async!(|b| tokio::io::AsyncReadExt::read(r, &mut b), u32::from_ne_bytes)
}

pub struct UpQueue<T> {
    ptr: NonNull<T>,
    len: AtomicUsize,
    capacity: usize
}

impl<T> UpQueue<T> {
    #[inline]
    pub fn new (capacity: usize) -> Self {
        let layout = Layout::array::<T>(capacity).unwrap();
        let ptr = unsafe { std::alloc::alloc(layout) };
        if ptr.is_null() { std::alloc::handle_alloc_error(layout) }
        
        return Self {
            ptr: unsafe { NonNull::new_unchecked(ptr.cast()) },
            len: AtomicUsize::new(0),
            capacity
        }
    }
    
    #[inline]
    pub fn len (&self) -> usize {
        return self.len.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn capacity (&self) -> usize {
        return self.capacity
    }

    #[inline]
    pub unsafe fn get_unchecked (&self, idx: usize) -> &T {
        return unsafe { &*self.ptr.as_ptr().add(idx) }
    }

    #[inline]
    pub unsafe fn get_unchecked_mut (&mut self, idx: usize) -> &mut T {
        return unsafe { &mut *self.ptr.as_ptr().add(idx) }
    }

    #[inline]
    pub fn get (&self, idx: usize) -> Option<&T> {
        if idx < self.len.load(Ordering::Acquire) {
            return unsafe { Some(self.get_unchecked(idx)) }
        }
        return None
    }

    #[inline]
    pub fn get_mut (&mut self, idx: usize) -> Option<&mut T> {
        if idx < *self.len.get_mut() {
            return unsafe { Some(self.get_unchecked_mut(idx)) }
        }
        return None
    }

    #[inline]
    pub fn try_push<'a> (&'a self, v: T) -> Result<(usize, &'a T), T> {
        let idx = self.len.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        if idx >= self.capacity {
            self.len.store(self.capacity, std::sync::atomic::Ordering::Relaxed);
            return Err(v)
        }

        unsafe {
            let ptr = self.ptr.as_ptr().add(idx);
            ptr.write(v);
            return Ok((idx, &mut *ptr))
        }
    }

    #[inline]
    pub fn try_push_mut<'a> (&'a mut self, v: T) -> Result<(usize, &'a mut T), T> {
        let len = self.len.get_mut();
        let idx = *len;
        if idx >= self.capacity { return Err(v) }
        *len += 1;

        unsafe {
            let ptr = self.ptr.as_ptr().add(idx);
            ptr.write(v);
            return Ok((idx, &mut *ptr))
        }
    }

    #[inline]
    pub fn iter (&self) -> Iter<'_, T> {
        return Iter {
            range: 0..self.len(),
            parent: self,
        }
    }

    #[inline]
    pub fn iter_indexed (&self) -> IterIndexed<'_, T> {
        return IterIndexed {
            range: 0..self.len(),
            parent: self,
        }
    }
}

#[derive(Clone)]
pub struct Iter<'a, T> {
    parent: &'a UpQueue<T>,
    range: Range<usize>
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.range.next()?;
        return unsafe { Some(self.parent.get_unchecked(idx)) }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let idx = self.range.nth(n)?;
        return unsafe { Some(self.parent.get_unchecked(idx)) }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        return self.range.size_hint();
    }
}

impl<T> DoubleEndedIterator for Iter<'_, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let idx = self.range.next_back()?;
        return unsafe { Some(self.parent.get_unchecked(idx)) }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let idx = self.range.nth_back(n)?;
        return unsafe { Some(self.parent.get_unchecked(idx)) }
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {
    #[inline]
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl<'a, T> IntoIterator for &'a UpQueue<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        UpQueue::iter(self)
    }
}

#[derive(Clone)]
pub struct IterIndexed<'a, T> {
    parent: &'a UpQueue<T>,
    range: Range<usize>
}

impl<'a, T> Iterator for IterIndexed<'a, T> {
    type Item = (usize, &'a T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.range.next()?;
        return unsafe { Some((idx, self.parent.get_unchecked(idx))) }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let idx = self.range.nth(n)?;
        return unsafe { Some((idx, self.parent.get_unchecked(idx))) }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        return self.range.size_hint();
    }
}

impl<T> DoubleEndedIterator for IterIndexed<'_, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let idx = self.range.next_back()?;
        return unsafe { Some((idx, self.parent.get_unchecked(idx))) }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let idx = self.range.nth_back(n)?;
        return unsafe { Some((idx, self.parent.get_unchecked(idx))) }
    }
}

impl<T> ExactSizeIterator for IterIndexed<'_, T> {
    #[inline]
    fn len(&self) -> usize {
        self.range.len()
    }
}