flat_mod! { map }
use std::marker::{PhantomData, PhantomPinned};

#[macro_export]
macro_rules! forward_phantom {
    ($ty:ty as $vis:vis $name:ident $( < $($lt:lifetime,)* $($t:ident $(: $bound:path)?,)* > )? ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(transparent)]
        $vis struct $name $(<$($lt,)* $($t $(: $bound)?,)*>)? (core::marker::PhantomData<$ty>);

        impl $(<$($lt,)* $($t $(: $bound)?,)*>)? $name $(<$($lt,)* $($t,)*>)? {
            #[inline]
            fn new () -> Self {
                Self(core::marker::PhantomData)
            }
        }

        unsafe impl $(<$($lt,)* $($t $(: $bound)?,)*>)? $crate::context::event::consumer::EventConsumer for $name $(<$($lt,)* $($t,)*>)? {
            type Output = ();

            #[inline]
            fn consume (self) -> Self::Output {}
        }
    };
}

pub unsafe trait EventConsumer {
    type Output;

    fn consume(self) -> Self::Output;
}

unsafe impl<T, F: FnOnce() -> T> EventConsumer for F {
    type Output = T;

    #[inline]
    fn consume(self) -> Self::Output {
        self()
    }
}

unsafe impl<T: ?Sized> EventConsumer for PhantomData<T> {
    type Output = ();

    #[inline]
    fn consume(self) -> Self::Output {}
}

unsafe impl EventConsumer for PhantomPinned {
    type Output = ();

    #[inline]
    fn consume(self) -> Self::Output {}
}
