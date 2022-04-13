use core::marker::PhantomData;

#[cfg(feature = "allocator-api")]
use alloc::alloc::{Allocator, Global};

#[cfg(not(feature = "allocator-api"))]
use crate::placeholders::*;

use crate::cstr::BasicCStr;
use crate::traits::CharTraits;

use alloc::boxed::Box;
use alloc::vec::Vec;

#[cfg(feature = "allocator-api")]
pub struct BasicCString<CharT, Traits, A: Allocator = Global> {
    inner: Vec<CharT, A>,
    _traits: PhantomData<Traits>,
    _allocator: PhantomData<A>,
}

#[cfg(not(feature = "allocator-api"))]
pub struct BasicCString<CharT, Traits, A: Allocator = Global> {
    inner: Vec<CharT>,
    _traits: PhantomData<Traits>,
    _allocator: PhantomData<A>,
}
