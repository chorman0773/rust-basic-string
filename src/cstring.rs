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

impl<Traits: CharTraits> BasicCString<Traits::Char, Traits, Global> {
    pub fn new() -> Self {
        let mut arr = Vec::with_capacity(1);
        arr.push(Traits::zero_term());

        Self {
            inner: arr,
            _traits: PhantomData,
            _allocator: PhantomData,
        }
    }

    pub fn with_capacity(n: usize) -> Self {
        let mut arr = if n == 0 {
            Vec::with_capacity(1)
        } else {
            Vec::with_capacity(n)
        };

        arr.push(Traits::zero_term());

        Self {
            inner: arr,
            _traits: PhantomData,
            _allocator: PhantomData,
        }
    }
}

impl<Traits: CharTraits, A: Allocator> BasicCString<Traits::Char, Traits, A> {
    #[cfg(feature = "allocator-api")]
    pub fn new_in(alloc: A) -> Self {
        let mut arr = Vec::with_capacity_in(1, alloc);
        arr.push(Traits::zero_term());

        Self {
            inner: arr,
            _traits: PhantomData,
            _allocator: PhantomData,
        }
    }

    #[cfg(feature = "allocator-api")]
    pub fn with_capacity_in(n: usize, alloc: A) -> Self {
        let mut arr = if n == 0 {
            Vec::with_capacity_in(1, alloc)
        } else {
            Vec::with_capacity_in(n, alloc)
        };

        arr.push(Traits::zero_term());

        Self {
            inner: arr,
            _traits: PhantomData,
            _allocator: PhantomData,
        }
    }
}
