use crate::{
    context::{
        event::{consumer::EventConsumer, Event},
        ContextRef, Context,
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
    time::{Duration, Instant}, pin::Pin, ops::Deref,
};
use utils_atomics::FillQueue;

struct Node {
    fence: Fence<NodeContext>,
    budget: u64, // in nanos
    flag: utils_atomics::flag::mpmc::AsyncFlag,
    result: Weak<vk::Result>,
}

pub struct EventRuntime<C: ContextRef> {
    handle: EventRuntimeHandle<C>,
    entries: Vec<InnerEntry>
}

impl<C: Clone + ContextRef> EventRuntime<C> {
    #[inline]
    pub fn new (context: C) -> Self where C: Unpin {
        return Self::new_pinned(Pin::new(context))
    }

    #[inline]
    pub unsafe fn new_unchecked (context: C) -> Self  {
        return Self::new_pinned(Pin::new_unchecked(context))
    }
    
    #[inline]
    pub fn new_pinned (context: Pin<C>) -> Self {
        return Self {
            handle: EventRuntimeHandle {
                queue: Arc::new(FillQueue::new()),
                context
            },
            entries: Vec::new()
        }
    }

    #[inline]
    pub fn handle(&self) -> EventRuntimeHandle<C> where C: Clone {
        self.handle.clone()
    }

    pub fn run_to_end (&mut self) {
        self.entries.clear();
        loop {
            // Check if receiver is still open
            for Node {
                fence,
                budget,
                flag,
                result,
            } in self.queue.chop()
            {
                let fence = ManuallyDrop::new(fence);
                let fence = fence.id();

                self.entries.push(InnerEntry {
                    fences: vec![fence],
                    data: vec![Metadata {
                        budget,
                        flag,
                        result,
                        #[cfg(debug_assertions)] checks: 0
                    }],
                });
            }

            // Check if receiver is still open
            if Arc::strong_count(&self.queue) == 1 && self.entries.iter().all(|entry| entry.fences.is_empty()) {
                break
            }

            for entry in self.entries.iter_mut() {
                if let Some(mut budget) = entry.data.iter().map(|x| x.budget).max() {
                    // Run until budget is consumed
                    loop {
                        let now = Instant::now();
                        let wait = (Entry::get().wait_for_fences)(
                            self.handle.context.device().id(),
                            usize_to_u32(entry.fences.len()),
                            entry.fences.as_ptr(),
                            vk::FALSE,
                            budget,
                        );
                        let elapsed = now.elapsed();

                        match wait {
                            vk::TIMEOUT => {
                                #[cfg(debug_assertions)]
                                for data in entry.data.iter_mut() {
                                    data.checks += 1
                                }
                                break
                            },
                            _ => {
                                // Find and treat with all completed/errored fences
                                let mut i = 0;
                                while i < entry.fences.len() {
                                    let fence = unsafe { *entry.fences.get_unchecked(i) };

                                    // Check if someone is still listening for the result
                                    if let Some(mut result) =
                                        unsafe { entry.data.get_unchecked(i).result.upgrade() }
                                    {
                                        { entry.data[i].checks += 1 };
                                        match (Entry::get().get_fence_status)(
                                            self.handle.context.device().id(),
                                            fence,
                                        ) {
                                            vk::NOT_READY => i += 1,

                                            r => unsafe {
                                                // IMPL: Although the order is not mainteined, the correlation between the
                                                // indices of both vectors is, so this is sound
                                                let fence = entry.fences.swap_remove(i);
                                                let data = entry.data.swap_remove(i);

                                                // Print number of checks
                                                #[cfg(debug_assertions)]
                                                println!("fence {fence} has been checked {} times", data.checks);

                                                // Destroy fence
                                                (Entry::get().destroy_fence)(
                                                    self.handle.context.device().id(),
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
                                            self.handle.context.device().id(),
                                            fence,
                                            core::ptr::null(),
                                        );
                                    }
                                }

                                // Check if there is enough remaining budget
                                if entry.fences.len() > 0 {
                                    if let Some(remaining) =
                                        budget.checked_sub(elapsed.as_nanos() as u64)
                                    {
                                        budget = remaining;
                                        continue;
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<C: ContextRef> Deref for EventRuntime<C> {
    type Target = EventRuntimeHandle<C>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

/// Handle to interect with a running [`FenceRuntime`]
pub struct EventRuntimeHandle<C: ContextRef> {
    queue: Arc<FillQueue<Node>>,
    context: Pin<C>
}

impl<C: ContextRef> EventRuntimeHandle<C> {
    /// Kinda arbitrary, might change
    pub const MAX_BUDGET: Duration = Duration::from_millis(5);

    pub(crate) fn push<F: EventConsumer>(
        &self,
        event: Event<C, F>,
        budget: Duration,
    ) -> Result<EventWait<F>> {
        if event.device() != self.context.device() {
            #[cfg(debug_assertions)]
            eprintln!("{:#?} is should be the same device as {:#?}", event.device(), self.context.device());
            return Err(vk::ERROR_DEVICE_LOST.into())
        }

        let fence = ManuallyDrop::new(event.fence);
        let budget = Duration::min(budget, Self::MAX_BUDGET);
        let result = Arc::new(vk::ERROR_UNKNOWN); // protect against unexpected panics by returning ERROR_UNKNOWN if we check the result before it's set
        let (flag, sub) = utils_atomics::flag::mpmc::async_flag();

        self.queue.push(Node {
            fence: Fence {
                inner: fence.inner,
                context: NodeContext(&self.context as &Context),
            },
            budget: budget.as_nanos() as u64,
            result: Arc::downgrade(&result),
            flag,
        });

        return Ok(EventWait {
            f: Some(event.c),
            flag: sub,
            result,
        });
    }
}

impl<C: Clone + ContextRef> Clone for EventRuntimeHandle<C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
            context: self.context.clone()
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

struct NodeContext (*const Context);

impl Deref for NodeContext {
    type Target = Context;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

unsafe impl Send for NodeContext where for<'a> &'a Context: Send {}
unsafe impl Sync for NodeContext where for<'a> &'a Context: Sync {}

struct Metadata {
    budget: u64, // in nanos
    flag: utils_atomics::flag::mpmc::AsyncFlag,
    result: Weak<vk::Result>, // `Weak` enables abortion detection
    #[cfg(debug_assertions)]
    checks: u64
}

struct InnerEntry {
    fences: Vec<vk::Fence>,
    data: Vec<Metadata>,
}