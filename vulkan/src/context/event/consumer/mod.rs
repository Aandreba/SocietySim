mod map;
use std::{marker::{PhantomData, PhantomPinned}};

#[macro_export(local_inner_macros)]
macro_rules! forward_phantom {
    ($ty:ty as $vis:vis $name:ident $($t:tt)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(transparent)]
        $vis struct $name $($t)* (core::marker::PhantomData<$ty>);

        impl $($t)* $name $($t)* {
            #[inline]
            fn new () -> Self {
                Self(core::marker::PhantomData)
            } 
        }

        impl $($t)* $crate::context::event::consumer::EventConsumer for $name $($t)* {
            type Output = ();

            #[inline]
            fn consume (self) -> Self::Output {}
        }
    };
}

pub trait EventConsumer {
    type Output;

    fn consume (self) -> Self::Output;
}

impl<T, F: FnOnce() -> T> EventConsumer for F {
    type Output = T;

    #[inline]
    fn consume (self) -> Self::Output {
        self()
    }
}

impl<T: ?Sized> EventConsumer for PhantomData<T> {
    type Output = ();

    #[inline]
    fn consume (self) -> Self::Output {}
}

impl EventConsumer for PhantomPinned {
    type Output = ();

    #[inline]
    fn consume (self) -> Self::Output {}
}