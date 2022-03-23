#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "allocator-api", feature(allocator_api))]
#![cfg_attr(feature = "const-trait-impl", feature(const_trait_impl))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod cstr;
pub mod str;
#[cfg(feature = "alloc")]
pub mod string;
pub mod traits;
pub mod utf;
pub mod view;
