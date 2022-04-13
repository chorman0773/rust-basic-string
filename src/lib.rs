#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(feature = "allocator-api", feature(allocator_api))]
#![cfg_attr(feature = "const-trait-impl", feature(const_trait_impl))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(not(feature = "allocator-api"))]
pub(crate) mod placeholders;

pub mod cstr;
#[cfg(feature = "alloc")]
pub mod cstring;
pub mod str;
#[cfg(feature = "alloc")]
pub mod string;
pub mod traits;
#[cfg(feature = "utf")]
pub mod utf;
pub mod view;

#[cfg(feature = "pattern")]
pub mod pattern;
