use std::{time::Duration};
use crate::{context::ContextRef, r#async::{EventRuntimeHandle, EventWait}};
use super::{consumer::EventConsumer, Event};

impl<C: ContextRef, F: EventConsumer> Event<C, F> {
    /// Kinda arbitrary, may change in the future
    pub const MAX_BUDGET: Duration = Duration::from_millis(5);

    #[inline]
    pub fn wait_async<'a> (self, handle: EventRuntimeHandle<C>) -> EventWait<F> where Self: 'a, C: 'a + Sync {
        return self.wait_async_with_budget(Self::MAX_BUDGET, handle)
    }
    
    #[inline]
    pub fn wait_async_with_budget<'a> (self, budget: Duration, handle: EventRuntimeHandle<C>) -> EventWait<F> where Self: 'a, C: 'a + Sync {
        return handle.push(self, budget)
    }
}