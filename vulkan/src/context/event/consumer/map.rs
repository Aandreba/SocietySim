use super::EventConsumer;

#[derive(Debug, Clone)]
pub struct Map<F, U> {
    f: F,
    u: U
}

impl<T, F: EventConsumer, U: FnOnce(F::Output) -> T> EventConsumer for Map<F, U> {
    type Output = T;

    #[inline]
    fn consume (self) -> Self::Output {
        (self.u)(self.f.consume())
    }
}
