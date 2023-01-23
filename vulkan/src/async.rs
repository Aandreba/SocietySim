use crate::{context::ContextRef, sync::Fence, Entry, utils::usize_to_u32};
use std::{marker::PhantomData, sync::{Arc, Weak}, time::{Duration, Instant}};
use utils_atomics::FillQueue;

struct Node<'a> {
    device: vk::Device,
    fence: vk::Fence,
    budget: u64, // in nanos
    flag: utils_atomics::flag::mpmc::AsyncFlag,
    result: Arc<vk::Result>,
    _phtm: PhantomData<&'a Fence<dyn 'a + Sync + ContextRef>>,
}

pub struct FenceRuntime<'a> {
    handle: FenceRuntimeHandle<'a>,
}

impl<'a> FenceRuntime<'a> {
    #[inline]
    pub fn handle(&self) -> FenceRuntimeHandle<'a> {
        self.handle.clone()
    }

    #[inline]
    pub fn run(self) {
        struct Metadata {
            budget: u64, // in nanos
            flag: utils_atomics::flag::mpmc::AsyncFlag,
            result: Arc<vk::Result>
        }

        struct InnerEntry {
            fences: Vec<vk::Fence>,
            data: Vec<Metadata>,
        }

        let recv = unsafe {
            core::mem::transmute::<_, Weak<FillQueue<Node<'static>>>>(Arc::downgrade(&self.handle.queue))
        };

        std::thread::spawn(move || {
            let mut recv = Some(recv);
            let mut devices = Vec::<(_, InnerEntry)>::new();

            loop {
                // Check if receiver is still open
                if let Some(ref this_recv) = recv {
                    // Check if receiver is still open
                    if let Some(this_recv) = this_recv.upgrade() {
                        for Node {
                            device,
                            fence,
                            budget,
                            flag,
                            result,
                            _phtm,
                        } in this_recv.chop()
                        {
                            // Add to device-fences map
                            let mut meta = Some(Metadata {
                                budget,
                                flag,
                                result,
                            });

                            for (key, entry) in devices.iter_mut() {
                                if key == &device {
                                    entry.fences.push(fence);
                                    unsafe { entry.data.push(core::mem::take(&mut meta).unwrap_unchecked()) };
                                    break;
                                }
                            }

                            // If the device wasn't found, add new entry
                            if let Some(meta) = meta {
                                devices.push((
                                    device,
                                    InnerEntry {
                                        fences: vec![fence],
                                        data: vec![meta],
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

                for (device, entry) in devices.iter_mut() {
                    if let Some(mut budget) = entry.data.iter().map(|x| x.budget).max() {
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
                                                let mut data = entry.data.swap_remove(i);

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
    }
}

/// Handle to interect with a running [`FenceRuntime`]
#[derive(Clone)]
pub struct FenceRuntimeHandle<'a> {
    queue: Arc<FillQueue<Node<'a>>>,
}

impl<'a> FenceRuntimeHandle<'a> {
    /// Kinda arbitrary, might change
    pub const MAX_BUDGET: Duration = Duration::from_millis(5);

    pub(crate) fn push<C: 'a + Sync + ContextRef>(
        &self,
        fence: &'a Fence<C>,
        budget: Duration,
    ) -> (Arc<vk::Result>, utils_atomics::flag::mpmc::AsyncSubscribe) {
        let budget = Duration::min(budget, Self::MAX_BUDGET);
        let result = Arc::new(vk::ERROR_UNKNOWN); // protect against unexpected panics by returning ERROR_UNKNOWN if we check the result before it's set
        let (flag, sub) = utils_atomics::flag::mpmc::async_flag();

        self.queue.push(Node {
            device: fence.device().id(),
            fence: fence.id(),
            budget: budget.as_nanos() as u64,
            result: result.clone(),
            _phtm: PhantomData,
            flag,
        });

        return (result, sub)
    }
}
