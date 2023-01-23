use std::{time::Duration};
use crate::{context::ContextRef, r#async::{EventRuntimeHandle, EventWait}, Result};
use super::{consumer::EventConsumer, Event};

impl<C: ContextRef, F: EventConsumer> Event<C, F> {
    /// Kinda arbitrary, may change in the future
    pub const MAX_BUDGET: Duration = EventRuntimeHandle::<C>::MAX_BUDGET;

    #[inline]
    pub fn wait_async<'a> (self, handle: &EventRuntimeHandle<C>) -> Result<EventWait<F>> {
        return self.wait_async_with_budget(Self::MAX_BUDGET, handle)
    }
    
    #[inline]
    pub fn wait_async_with_budget<'a> (self, budget: Duration, handle: &EventRuntimeHandle<C>) -> Result<EventWait<F>> {
        return handle.push(self, budget)
    }
}