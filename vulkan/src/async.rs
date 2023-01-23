use crate::{
    context::{
        event::{consumer::EventConsumer, Event},
        ContextRef,
    },
    sync::Fence,
    utils::usize_to_u32,
    Entry, Result,
};
use futures::Future;
use pin_project::pin_project;
use std::{
    mem::ManuallyDrop,
    sync::{Arc, Weak},
    task::Poll,
    time::{Duration, Instant},
};
use utils_atomics::FillQueue;

struct Node<C: ContextRef> {
    fence: Fence<C>,
    budget: u64, // in nanos
    flag: utils_atomics::flag::mpmc::AsyncFlag,
    result: Weak<vk::Result>,
}

pub struct EventRuntime<C: ContextRef> {
    handle: EventRuntimeHandle<C>,
}

impl<C: ContextRef> EventRuntime<C> {
    #[inline]
    pub fn new () -> Self {
        return Self {
            handle: EventRuntimeHandle {
                queue: Arc::new(FillQueue::new())
            }
        }
    }

    #[inline]
    pub fn handle(&self) -> EventRuntimeHandle<C> {
        self.handle.clone()
    }

    pub fn run_to_end (self) {
        struct Metadata {
            budget: u64, // in nanos
            flag: utils_atomics::flag::mpmc::AsyncFlag,
            result: Weak<vk::Result>, // `Weak` enables abortion detection
        }

        struct InnerEntry {
            fences: Vec<vk::Fence>,
            data: Vec<Metadata>,
        }

        struct Map<C: ContextRef>(Vec<(C, InnerEntry)>);
        impl<C: ContextRef> Drop for Map<C> {
            #[inline]
            fn drop(&mut self) {
                for (context, mut entry) in self.0.drain(..) {
                    for fence in entry.fences.drain(..) {
                        (Entry::get().destroy_fence)(
                            context.device().id(),
                            fence,
                            core::ptr::null(),
                        );
                    }
                }
            }
        }

        let mut recv = Some(Arc::downgrade(&self.handle.queue));
        let mut contexts = Map(Vec::<(C, InnerEntry)>::new());
        drop(self.handle);

        loop {
            // Check if receiver is still open
            if let Some(ref this_recv) = recv {
                // Check if receiver is still open
                if let Some(this_recv) = this_recv.upgrade() {
                    for Node {
                        fence,
                        budget,
                        flag,
                        result,
                    } in this_recv.chop()
                    {
                        let fence = ManuallyDrop::new(fence);
                        let context = unsafe { core::ptr::read(&fence.context) };
                        let fence = fence.id();

                        // Add to device-fences map
                        let mut meta = Some(Metadata {
                            budget,
                            flag,
                            result,
                        });

                        for (key, entry) in contexts.0.iter_mut() {
                            if key.device() == context.device() {
                                entry.fences.push(fence);
                                unsafe {
                                    entry
                                        .data
                                        .push(core::mem::take(&mut meta).unwrap_unchecked())
                                };
                                break;
                            }
                        }

                        // If the device wasn't found, add new entry
                        if let Some(meta) = meta {
                            contexts.0.push((
                                context,
                                InnerEntry {
                                    fences: vec![fence],
                                    data: vec![meta],
                                },
                            ));
                        }
                    }
                } else if contexts.0.iter().all(|(_, entry)| entry.fences.is_empty()) {
                    break;
                } else {
                    recv = None
                }
            } else if contexts.0.iter().all(|(_, entry)| entry.fences.is_empty()) {
                break;
            }

            for (context, entry) in contexts.0.iter_mut() {
                if let Some(mut budget) = entry.data.iter().map(|x| x.budget).max() {
                    // Run until budget is consumed
                    loop {
                        let now = Instant::now();
                        let wait = (Entry::get().wait_for_fences)(
                            context.device().id(),
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

                                    // Check if someone is still listening for the result
                                    if let Some(mut result) =
                                        unsafe { entry.data.get_unchecked(i).result.upgrade() }
                                    {
                                        match (Entry::get().get_fence_status)(
                                            context.device().id(),
                                            fence,
                                        ) {
                                            vk::NOT_READY => i += 1,
                                            r => unsafe {
                                                // IMPL: Although the order is not mainteined, the correlation between the
                                                // indices of both vectors is, so this is sound
                                                let fence = entry.fences.swap_remove(i);
                                                let data = entry.data.swap_remove(i);

                                                // Destroy fence
                                                (Entry::get().destroy_fence)(
                                                    context.device().id(),
                                                    fence,
                                                    core::ptr::null(),
                                                );

                                                // SAFETY: Until the flag is marked, we are the only ones with access to err
                                                *Arc::get_mut_unchecked(&mut result) = r;
                                                data.flag.mark();
                                            },
                                        }
                                    } else {
                                        // If no one is listening for this fence's result, remove it from the queue.
                                        //
                                        // IMPL: Although the order is not mainteined, the correlation between the
                                        // indices of both vectors is, so this is sound
                                        let fence = entry.fences.swap_remove(i);
                                        let _ = entry.data.swap_remove(i);

                                        // Destroy fence
                                        (Entry::get().destroy_fence)(
                                            context.device().id(),
                                            fence,
                                            core::ptr::null(),
                                        );
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
    }
}

/// Handle to interect with a running [`FenceRuntime`]
pub struct EventRuntimeHandle<C: ContextRef> {
    queue: Arc<FillQueue<Node<C>>>
}

impl<C: ContextRef> EventRuntimeHandle<C> {
    /// Kinda arbitrary, might change
    pub const MAX_BUDGET: Duration = Duration::from_millis(5);

    pub(crate) fn push<F: EventConsumer>(
        &self,
        event: Event<C, F>,
        budget: Duration,
    ) -> EventWait<F> {
        let budget = Duration::min(budget, Self::MAX_BUDGET);
        let result = Arc::new(vk::ERROR_UNKNOWN); // protect against unexpected panics by returning ERROR_UNKNOWN if we check the result before it's set
        let (flag, sub) = utils_atomics::flag::mpmc::async_flag();
        self.queue.push(Node {
            fence: event.fence,
            budget: budget.as_nanos() as u64,
            result: Arc::downgrade(&result),
            flag,
        });

        return EventWait {
            f: Some(event.c),
            flag: sub,
            result,
        };
    }
}

impl<C: ContextRef> Clone for EventRuntimeHandle<C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
        }
    }
}

#[derive(Debug, Clone)]
#[pin_project]
pub struct EventWait<F> {
    #[pin]
    flag: utils_atomics::flag::mpmc::AsyncSubscribe,
    result: Arc<vk::Result>,
    f: Option<F>,
}

impl<F: EventConsumer> Future for EventWait<F> {
    type Output = Result<F::Output>;

    #[inline]
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();

        if this.flag.poll(cx).is_ready() {
            let result: vk::Result = *(this.result as &i32);
            return Poll::Ready(match result {
                vk::SUCCESS => Ok(core::mem::take(this.f).unwrap().consume()),
                e => Err(e.into()),
            });
        }

        return Poll::Pending;
    }
}
