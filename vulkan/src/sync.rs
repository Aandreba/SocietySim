use futures::Future;
use pin_project::pin_project;
use utils_atomics::FillQueue;

use crate::{context::ContextRef, device::Device, utils::usize_to_u32, Entry, Result};
use std::{
    num::NonZeroU64,
    ptr::{addr_of, addr_of_mut},
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    time::{Duration, Instant}, ops::Deref,
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Fence<C: ContextRef> {
    inner: NonZeroU64,
    context: C,
}

impl<C: ContextRef> Fence<C> {
    /// Kinda arbitrary, may change in the future
    pub const MAX_BUDGET: Duration = Duration::from_millis(5);

    pub fn new(context: C, flags: FenceFlags) -> Result<Self> {
        let info = vk::FenceCreateInfo {
            sType: vk::STRUCTURE_TYPE_FENCE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: flags.bits(),
        };

        let mut result = 0;
        tri! {
            (Entry::get().create_fence)(
                context.device().id(),
                addr_of!(info),
                core::ptr::null(),
                addr_of_mut!(result)
            )
        }

        if let Some(inner) = NonZeroU64::new(result) {
            return Ok(Self { inner, context });
        }
        return Err(vk::ERROR_UNKNOWN.into());
    }

    #[inline]
    pub fn id(&self) -> u64 {
        return self.inner.get();
    }

    #[inline]
    pub fn device(&self) -> &Device {
        return self.context.device();
    }

    #[inline]
    pub fn status(&self) -> Result<bool> {
        return match (Entry::get().get_fence_status)(self.device().id(), self.id()) {
            vk::SUCCESS => Ok(true),
            vk::NOT_READY => Ok(false),
            e => Err(e.into()),
        };
    }

    #[inline]
    pub fn is_signaled(&self) -> bool {
        matches!(self.status(), Ok(true))
    }

    #[inline]
    pub fn is_unsignaled(&self) -> bool {
        matches!(self.status(), Ok(false))
    }

    #[inline]
    pub fn reset(&mut self) -> Result<()> {
        tri! {
            (Entry::get().reset_fences)(
                self.device().id(),
                1,
                addr_of!(self.inner).cast()
            )
        }
        return Ok(());
    }

    #[inline]
    pub fn wait(&self) -> Result<()> {
        loop {
            match self.wait_for_fences(u64::MAX) {
                vk::SUCCESS => return Ok(()),
                vk::TIMEOUT => {}
                e => return Err(e.into()),
            }
        }
    }

    pub fn wait_timeout(&self, timeout: Duration) -> Result<bool> {
        const LIMIT: u128 = u64::MAX as u128;

        let nanos = timeout.as_nanos();
        let div = nanos / LIMIT;
        let rem = (nanos % LIMIT) as u64; // [0, u64::MAX)

        for _ in 0..div {
            match self.wait_for_fences(u64::MAX) {
                vk::SUCCESS => return Ok(true),
                vk::TIMEOUT => {}
                e => return Err(e.into()),
            }
        }

        if rem > 0 {
            return match self.wait_for_fences(rem) {
                vk::SUCCESS => Ok(true),
                vk::TIMEOUT => Ok(false),
                e => Err(e.into()),
            };
        }

        return Ok(false);
    }

    #[inline]
    pub fn wait_async(&self) -> FenceWait<C> {
        self.wait_async_with_budget(Self::MAX_BUDGET)
    }

    #[inline]
    pub fn wait_async_with_budget_by_deref<D: 'static + Send + Clone + Deref<Target = Self>> (this: D, budget: Duration) -> FenceWait<C> {
        let budget = Duration::min(budget, Self::MAX_BUDGET).as_nanos() as u64;
        let flag = send_fence(this, budget);
        todo!()
    }

    #[inline]
    fn wait_for_fences(&self, nanos: u64) -> vk::Result {
        return (Entry::get().wait_for_fences)(
            self.device().id(),
            1,
            addr_of!(self.inner).cast(),
            vk::TRUE,
            nanos,
        );
    }
}

impl<D: ContextRef> Drop for Fence<D> {
    #[inline]
    fn drop(&mut self) {
        (Entry::get().destroy_fence)(self.device().id(), self.id(), core::ptr::null())
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct FenceFlags: vk::FenceCreateFlagBits {
        const SIGNALED = vk::FENCE_CREATE_SIGNALED_BIT;
    }
}

const UNINIT: u8 = 0;
const WAITING: u8 = 1;
const ABORTING: u8 = 2;

#[derive(Debug, Clone)]
#[pin_project]
pub struct FenceWait<F> {
    #[pin]
    flag: utils_atomics::flag::mpmc::AsyncSubscribe,
    err: Arc<vk::Result>,
    fence: F,
    aborting: Arc<AtomicU8>,
}

impl<F: Deref<Target = Fence<C>>, C: ContextRef> Future for FenceWait<F> {
    type Output = Result<()>;

    #[inline]
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.project().flag.poll(cx).is_ready() {
            todo!()
        }
        todo!()
    }
}

// impl<C: ContextRef> Drop for FenceWait<'_, C> {
//     #[inline]
//     fn drop(&mut self) {
//         let mut guard = match self.abort.lock() {
//             Ok(x) => x,
//             Err(e) => e.into_inner()
//         };
//         *guard = true
//     }
// }

struct AssertSend<T> (pub T);
unsafe impl<T> Send for AssertSend<T> {}
unsafe impl<T> Sync for AssertSend<T> {}

struct Signal {
    device: u64,
    fence: u64,
    budget: u64, // in nanos
    flag: utils_atomics::flag::mpmc::AsyncFlag,
    result: Arc<vk::Result>,
    drop: AssertSend<(*mut (), unsafe fn(*mut ()))>, 
}

thread_local! {
    static FENCE_THREAD: once_cell::unsync::OnceCell<Arc<FillQueue<Signal>>> = once_cell::unsync::OnceCell::new();
}

fn send_fence<F: 'static + Send + Deref<Target = Fence<C>>, C: ContextRef>(
    fence: F,
    budget: u64
) -> (utils_atomics::flag::mpmc::AsyncSubscribe, Arc<vk::Result>) {
    let (flag, sub) = utils_atomics::flag::mpmc::async_flag();
    let result = Arc::new(vk::ERROR_UNKNOWN);
    let drop: unsafe fn(*mut ()) = |ptr| unsafe {
        core::ptr::drop_in_place(ptr.cast::<F>())
    };
    
    let queue = FENCE_THREAD.with(|f| f.get_or_init(init_thread));
    queue.push(Signal {
        device: fence.device().id(),
        fence: fence.id(),
        result: result.clone(),
        budget,
        flag,
        drop: AssertSend((Box::into_raw(Box::new(fence)).cast(), drop))
    });

    return (sub, result);
}

#[inline]
fn init_thread() -> Arc<FillQueue<Signal>> {
    let queue = Arc::new(FillQueue::new());
    let recv = Arc::downgrade(&queue);

    std::thread::spawn(move || {
        struct Metadata {
            budget: u64, // in nanos
            flag: utils_atomics::flag::mpmc::AsyncFlag,
            result: Arc<vk::Result>,
            drop: (*mut (), unsafe fn(*mut ()))
        }

        impl Drop for Metadata {
            #[inline]
            fn drop(&mut self) {
                unsafe { self.drop.1(self.drop.0) }
            }
        }

        struct InnerEntry {
            fences: Vec<vk::Fence>,
            data: Vec<Metadata>,
        }

        let mut recv = Some(recv);
        let mut devices = Vec::<(_, InnerEntry)>::new();

        loop {
            // Check if receiver is still open
            if let Some(ref this_recv) = recv {
                // Check if receiver is still open
                if let Some(this_recv) = this_recv.upgrade() {
                    for Signal {
                        device,
                        fence,
                        budget,
                        flag,
                        result,
                        drop
                    } in this_recv.chop()
                    {
                        // Add to device-fences map
                        let mut added = false;
                        for (key, entry) in devices.iter_mut() {
                            if key == &device {
                                entry.fences.push(fence);
                                entry.data.push(Metadata {
                                    budget,
                                    flag,
                                    result,
                                    drop: drop.0
                                });
                                added = true;
                                break;
                            }
                        }

                        // If the device wasn't found, add new entry
                        if !added {
                            devices.push((
                                device,
                                InnerEntry {
                                    fences: vec![fence],
                                    data: vec![Metadata {
                                        budget,
                                        flag,
                                        result,
                                        drop: drop.0
                                    }],
                                },
                            ));
                        }
                    }
                } else if devices.iter().all(|(_, entry)| entry.fences.is_empty()) {
                    break;
                } else {
                    recv = None
                }
            } else if devices.iter().all(|(_, entry)| entry.fences.is_empty()) {
                break;
            }

            for (device, entry) in devices.iter() {
                if let Some(budget) = entry.data.iter().map(|x| x.budget).max() {
                    // Run until budget is consumed
                    loop {
                        let now = Instant::now();
                        let wait = (Entry::get().wait_for_fences)(
                            *device,
                            usize_to_u32(entry.fences.len()),
                            entry.fences.as_ptr(),
                            vk::FALSE,
                            budget,
                        );
                        let elapsed = now.elapsed();

                        match wait {
                            vk::TIMEOUT => break,
                            _ => {
                                // Find and treat with all completed/errored fences
                                let mut i = 0;
                                while i < entry.fences.len() {
                                    let fence = unsafe { *entry.fences.get_unchecked(i) };
                                    match (Entry::get().get_fence_status)(*device, fence) {
                                        vk::NOT_READY => i += 1,
                                        r => unsafe {
                                            // IMPL: Although the order is not mainteined, the correlation between the
                                            // indices of both vectors is, so this is sound
                                            entry.fences.swap_remove(i);
                                            let data = entry.data.swap_remove(i);

                                            // SAFETY: Until the flag is marked, we are the only ones with access to err
                                            *Arc::get_mut_unchecked(&mut data.result) = r;
                                            data.flag.mark();
                                        },
                                    }
                                }

                                // Check if there is enough remaining budget
                                if let Some(remaining) =
                                    budget.checked_sub(elapsed.as_nanos() as u64)
                                {
                                    budget = remaining
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    return queue;
}
