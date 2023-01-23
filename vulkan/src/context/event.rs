use std::time::Duration;
use crate::{sync::Fence, Result};
use super::ContextRef;

pub struct Event<C: ContextRef> {
    fence: Fence<C>
}

impl<C: ContextRef> Event<C> {
    #[inline]
    pub fn wait (&self) -> Result<()> {
        self.fence.wait()
    }

    #[inline]
    pub fn wait_timeout (&self, timeout: Duration) -> Result<bool> {
        self.fence.wait_timeout(timeout)
    }
}