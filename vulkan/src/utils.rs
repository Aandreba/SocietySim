use std::io::{Read, ErrorKind};
use docfg::docfg;
use futures::{AsyncRead, AsyncReadExt};

/// Casts `usize` to `u32`
/// 
/// # Panics
/// This method will panic when, in debug mode, the `usize` value doesn't fit inside a `u32`.
/// In release mode, the value will be truncated.
#[inline(always)]
pub(crate) fn usize_to_u32 (v: usize) -> u32 {
    #[cfg(debug_assertions)]
    return u32::try_from(v).unwrap();
    #[cfg(not(debug_assertions))]
    return v as u32
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