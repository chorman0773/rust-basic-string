use core::marker::PhantomData;
use core::ops::Deref;
use core::ops::DerefMut;
use std::mem::MaybeUninit;

use alloc::alloc::Global;

#[cfg(feature = "allocator-api")]
use alloc::alloc::Allocator;

use crate::str::BasicStr;
use crate::traits::CharTraits;
use crate::traits::IntoChars;

#[cfg(feature = "allocator-api")]
pub struct BasicString<CharT, Traits, A: Allocator = Global> {
    inner: Vec<CharT, A>,
    _traits: PhantomData<Traits>,
}

#[cfg(not(feature = "allocator-api"))]
pub struct BasicString<CharT, Traits> {
    inner: Vec<CharT>,
    _traits: PhantomData<Traits>,
}

impl<CharT, Traits> BasicString<CharT, Traits> {
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            _traits: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            _traits: PhantomData,
        }
    }
}

#[cfg(feature = "allocator-api")]
impl<CharT, Traits, A: Allocator> BasicString<CharT, Traits, A> {
    pub fn from_boxed_str(b: Box<BasicStr<CharT, Traits>, A>) -> Self {
        Self {
            inner: Vec::from(BasicStr::into_boxed_chars(b)),
            _traits: PhantomData,
        }
    }

    pub fn into_boxed_str(self) -> Box<BasicStr<CharT, Traits>, A> {
        unsafe { BasicStr::from_boxed_chars_unchecked(self.inner.into_boxed_slice()) }
    }

    pub fn into_chars(self) -> Vec<CharT, A> {
        self.inner
    }

    pub unsafe fn from_chars_unchecked(chars: Vec<CharT, A>) -> Self {
        Self {
            inner: chars,
            _traits: PhantomData,
        }
    }

    pub fn push_str<S: AsRef<BasicStr<CharT, Traits>>>(&mut self, s: &S)
    where
        CharT: Clone,
    {
        let str = s.as_ref();
        self.inner.extend_from_slice(str.as_chars());
    }
}

#[cfg(not(feature = "allocator-api"))]
impl<CharT, Traits> BasicString<CharT, Traits> {
    pub fn from_boxed_str(b: Box<BasicStr<CharT, Traits>>) -> Self {
        Self {
            inner: Vec::from(BasicStr::into_boxed_chars(b)),
            _traits: PhantomData,
        }
    }

    pub fn into_boxed_str(self) -> Box<BasicStr<CharT, Traits>> {
        unsafe { BasicStr::from_boxed_chars_unchecked(self.inner.into_boxed_slice()) }
    }

    pub fn into_chars(self) -> Vec<CharT, A> {
        self.inner
    }

    pub fn push_str<S: AsRef<BasicStr<CharT, Traits>>>(&mut self, s: &S)
    where
        CharT: Clone,
    {
        let str = s.as_ref();
        self.inner.extend_from_slice(str.as_chars());
    }
}

#[cfg(feature = "allocator-api")]
impl<CharT, Traits, A: Allocator> Deref for BasicString<CharT, Traits, A> {
    type Target = BasicStr<CharT, Traits>;

    fn deref(&self) -> &BasicStr<CharT, Traits> {
        unsafe { BasicStr::from_chars_unchecked(&self.inner) }
    }
}

#[cfg(not(feature = "allocator-api"))]
impl<CharT, Traits> Deref for BasicString<CharT, Traits> {
    type Target = BasicStr<CharT, Traits>;

    fn deref(&self) -> &BasicStr<CharT, Traits> {
        unsafe { BasicStr::from_chars_unchecked(&self.inner) }
    }
}

#[cfg(feature = "allocator-api")]
impl<CharT, Traits, A: Allocator> DerefMut for BasicString<CharT, Traits, A> {
    fn deref_mut(&mut self) -> &mut BasicStr<CharT, Traits> {
        unsafe { BasicStr::from_chars_unchecked_mut(&mut self.inner) }
    }
}

#[cfg(not(feature = "allocator-api"))]
impl<CharT, Traits> DerefMut for BasicString<CharT, Traits> {
    fn deref_mut(&mut self) -> &mut BasicStr<CharT, Traits> {
        unsafe { BasicStr::from_chars_unchecked_mut(&mut self.inner) }
    }
}

#[cfg(feature = "allocator-api")]
impl<CharT, Traits: CharTraits<Char = CharT> + IntoChars, A: Allocator>
    BasicString<CharT, Traits, A>
{
    pub fn push(&mut self, c: char) {
        let max_len = Traits::max_encoding_len();
        self.inner.reserve(max_len);
        let chars = unsafe { self.inner.as_mut_ptr().add(self.inner.len()) };
        let uninit_chars =
            unsafe { core::slice::from_raw_parts_mut(chars as *mut MaybeUninit<CharT>, max_len) };
        for char in uninit_chars {
            char.write(Traits::zero_term()); // We can insert anything, but `Traits::zero_term()` is a known-valid bitpattern
        }
        let nlen = self.inner.len()
            + Traits::encode(c, unsafe {
                core::slice::from_raw_parts_mut(chars, max_len)
            })
            .len();

        unsafe {
            self.inner.set_len(nlen);
        }
    }
}

#[cfg(not(feature = "allocator-api"))]
impl<CharT, Traits: CharTraits<Char = CharT> + IntoChars> BasicString<CharT, Traits> {
    pub fn push(&mut self, c: char) {
        let max_len = Traits::max_encoding_len();
        self.inner.reserve(max_len);
        let chars = unsafe { self.inner.as_mut_ptr().add(self.inner.len()) };
        let uninit_chars =
            unsafe { core::slice::from_raw_parts_mut(chars as *mut MaybeUninit<CharT>, max_len) };
        for char in uninit_chars {
            char.write(Traits::zero_term()); // We can insert anything, but `Traits::zero_term()` is a known-valid bitpattern
        }
        let nlen = self.inner.len()
            + Traits::encode(c, unsafe {
                core::slice::from_raw_parts_mut(chars, max_len)
            })
            .len();

        unsafe {
            self.inner.set_len(nlen);
        }
    }
}

#[cfg(feature = "allocator-api")]
pub struct FromCharsError<Traits: CharTraits, A: Allocator = alloc::alloc::Global> {
    chars: Vec<Traits::Char, A>,
    error: Traits::Error,
}

#[cfg(feature = "allocator-api")]
impl<Traits: CharTraits, A: Allocator> FromCharsError<Traits, A> {
    pub fn as_chars(&self) -> &[Traits::Char] {
        &self.chars
    }

    pub fn into_chars(self) -> Vec<Traits::Char, A> {
        self.chars
    }

    pub fn error(&self) -> &Traits::Error {
        &self.error
    }
}

#[cfg(feature = "allocator-api")]
impl<Traits: CharTraits, A: Allocator> core::fmt::Debug for FromCharsError<Traits, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FromCharsError")
            .field("error", &self.error)
            .finish()
    }
}

#[cfg(feature = "allocator-api")]
impl<Traits: CharTraits, A: Allocator> core::fmt::Display for FromCharsError<Traits, A>
where
    Traits::Error: core::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

#[cfg(not(feature = "allocator-api"))]
pub struct FromCharsError<Traits: CharTraits> {
    chars: Vec<Traits::Char>,
    error: Traits::Error,
}

#[cfg(not(feature = "allocator-api"))]
impl<Traits: CharTraits> FromCharsError<Traits> {
    pub fn as_chars(&self) -> &[Traits::Char] {
        &self.chars
    }

    pub fn into_chars(self) -> Vec<Traits::Char> {
        self.chars
    }

    pub fn error(&self) -> &Traits::Error {
        &self.error
    }
}

#[cfg(not(feature = "allocator-api"))]
impl<Traits: CharTraits> core::fmt::Debug for FromCharsError<Traits> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FromCharsError")
            .field("error", &self.error)
            .finish()
    }
}

#[cfg(not(feature = "allocator-api"))]
impl<Traits: CharTraits> core::fmt::Display for FromCharsError<Traits>
where
    Traits::Error: core::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

#[cfg(feature = "allocator-api")]
impl<CharT, Traits: CharTraits<Char = CharT>, A: Allocator> BasicString<CharT, Traits, A> {
    pub fn from_chars(chars: Vec<CharT, A>) -> Result<Self, FromCharsError<Traits, A>> {
        match Traits::validate_range(&chars) {
            Ok(()) => Ok(Self {
                inner: chars,
                _traits: PhantomData,
            }),
            Err(error) => Err(FromCharsError { chars, error }),
        }
    }
}

#[cfg(not(feature = "allocator-api"))]
impl<CharT, Traits: CharTraits<Char = CharT>> BasicString<CharT, Traits> {
    pub fn from_chars(chars: Vec<CharT>) -> Result<Self, FromCharsError<Traits>> {
        match Traits::validate_range(&chars) {
            Ok(()) => Ok(Self {
                inner: chars,
                _traits: PhantomData,
            }),
            Err(error) => Err(FromCharsError { chars, error }),
        }
    }
}
