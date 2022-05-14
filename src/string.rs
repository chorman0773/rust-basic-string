use core::borrow::Borrow;
use core::borrow::BorrowMut;
use core::cmp::Ordering;
use core::hash::Hash;
use core::hash::Hasher;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ops::DerefMut;

#[cfg(feature = "allocator-api")]
use alloc::alloc::{Allocator, Global};

#[cfg(not(feature = "allocator-api"))]
use crate::placeholders::*;

use crate::str::BasicStr;
use crate::traits::Char;
use crate::traits::CharTraits;
use crate::traits::IntoChars;

use alloc::boxed::Box;
use alloc::vec::Vec;

#[cfg(feature = "allocator-api")]
pub struct BasicString<CharT, Traits, A: Allocator = Global> {
    inner: Vec<CharT, A>,
    _traits: PhantomData<Traits>,
    _allocator: PhantomData<A>,
}

#[cfg(not(feature = "allocator-api"))]
pub struct BasicString<CharT, Traits, A: Allocator = Global> {
    inner: Vec<CharT>,
    _traits: PhantomData<Traits>,
    _allocator: PhantomData<A>,
}

impl<CharT, Traits> BasicString<CharT, Traits, Global> {
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            _traits: PhantomData,
            _allocator: PhantomData,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Vec::with_capacity(cap),
            _traits: PhantomData,
            _allocator: PhantomData,
        }
    }
}

impl<CharT, Traits, A: Allocator> BasicString<CharT, Traits, A> {
    #[cfg(feature = "allocator-api")]
    pub const fn new_in(alloc: A) -> Self {
        Self {
            inner: Vec::new_in(alloc),
            _traits: PhantomData,
            _allocator: PhantomData,
        }
    }
    #[cfg(feature = "allocator-api")]
    pub fn with_capacity_in(cap: usize, alloc: A) -> Self {
        Self {
            inner: Vec::with_capacity_in(cap, alloc),
            _traits: PhantomData,
            _allocator: PhantomData,
        }
    }

    #[cfg(feature = "allocator-api")]
    pub const unsafe fn from_chars_unchecked(chars: Vec<CharT, A>) -> Self {
        Self {
            inner: chars,
            _traits: PhantomData,
            _allocator: PhantomData,
        }
    }

    #[cfg(not(feature = "allocator-api"))]
    pub const unsafe fn from_chars_unchecked(chars: Vec<CharT>) -> Self {
        Self {
            inner: chars,
            _traits: PhantomData,
            _allocator: PhantomData,
        }
    }

    #[cfg(feature = "allocator-api")]
    pub fn into_chars(self) -> Vec<CharT, A> {
        self.inner
    }

    #[cfg(not(feature = "allocator-api"))]
    pub fn into_chars(self) -> Vec<CharT> {
        self.inner
    }

    #[cfg(feature = "allocator-api")]
    pub fn from_boxed_str(str: Box<BasicStr<CharT, Traits>, A>) -> Self {
        unsafe { Self::from_chars_unchecked(str.into_boxed_chars().into()) }
    }

    #[cfg(not(feature = "allocator-api"))]
    pub fn from_boxed_str(str: Box<BasicStr<CharT, Traits>>) -> Self {
        unsafe { Self::from_chars_unchecked(str.into_boxed_chars().into()) }
    }

    pub fn push_str(&mut self, s: &BasicStr<CharT, Traits>)
    where
        CharT: Char,
    {
        self.inner.extend_from_slice(s.as_chars());
    }
}

#[cfg(feature = "allocator-api")]
pub struct FromCharsError<CharT, FromCharsErr, A: Allocator = Global> {
    err: FromCharsErr,
    chars: Vec<CharT, A>,
    _allocator: PhantomData<A>,
}

#[cfg(not(feature = "allocator-api"))]
pub struct FromCharsError<CharT, CharsErr, A: Allocator = Global> {
    err: CharsErr,
    chars: Vec<CharT>,
    _allocator: PhantomData<A>,
}

impl<CharT, CharsErr, A: Allocator> FromCharsError<CharT, CharsErr, A> {
    #[cfg(feature = "allocator-api")]
    pub fn into_bytes(self) -> Vec<CharT, A> {
        self.chars
    }

    #[cfg(not(feature = "allocator-api"))]
    pub fn into_bytes(self) -> Vec<CharT> {
        self.chars
    }

    pub fn as_bytes(&self) -> &[CharT] {
        &self.chars
    }

    pub fn chars_error(&self) -> CharsErr
    where
        CharsErr: Clone,
    {
        self.err.clone()
    }
}

impl<Traits: CharTraits, A: Allocator> BasicString<Traits::Char, Traits, A> {
    #[cfg(feature = "allocator-api")]
    pub fn from_chars(
        chars: Vec<Traits::Char, A>,
    ) -> Result<Self, FromCharsError<Traits::Char, Traits::Error, A>> {
        match Traits::validate_range(&chars) {
            Ok(()) => Ok(Self {
                inner: chars,
                _traits: PhantomData,
                _allocator: PhantomData,
            }),
            Err(err) => Err(FromCharsError {
                err,
                chars,
                _allocator: PhantomData,
            }),
        }
    }

    #[cfg(not(feature = "allocator-api"))]
    pub fn from_chars(
        chars: Vec<Traits::Char>,
    ) -> Result<Self, FromCharsError<Traits::Char, Traits::Error, A>> {
        match Traits::validate_range(&chars) {
            Ok(()) => Ok(Self {
                inner: chars,
                _traits: PhantomData,
                _allocator: PhantomData,
            }),
            Err(err) => Err(FromCharsError {
                err,
                chars,
                _allocator: PhantomData,
            }),
        }
    }
}

impl<Traits: CharTraits + IntoChars, A: Allocator> BasicString<Traits::Char, Traits, A> {
    pub fn push(&mut self, c: char) {
        let base_len = self.len();
        let len = base_len.saturating_add(Traits::encoding_len(c));

        self.inner.resize_with(len, Traits::zero_term); // Use Zero-term as a default-init state

        let right = &mut self.inner[base_len..];
        Traits::encode(c, right);
    }
}

impl<CharT, Traits, A: Allocator> Deref for BasicString<CharT, Traits, A> {
    type Target = BasicStr<CharT, Traits>;

    fn deref(&self) -> &BasicStr<CharT, Traits> {
        unsafe { BasicStr::from_chars_unchecked(&self.inner) }
    }
}
impl<CharT, Traits, A: Allocator> DerefMut for BasicString<CharT, Traits, A> {
    fn deref_mut(&mut self) -> &mut BasicStr<CharT, Traits> {
        unsafe { BasicStr::from_chars_unchecked_mut(&mut self.inner) }
    }
}

impl<CharT, Traits, A: Allocator> Borrow<BasicStr<CharT, Traits>>
    for BasicString<CharT, Traits, A>
{
    fn borrow(&self) -> &BasicStr<CharT, Traits> {
        self
    }
}

impl<CharT, Traits, A: Allocator> BorrowMut<BasicStr<CharT, Traits>>
    for BasicString<CharT, Traits, A>
{
    fn borrow_mut(&mut self) -> &mut BasicStr<CharT, Traits> {
        self
    }
}

impl<S: ?Sized, CharT, Traits, A: Allocator> AsRef<S> for BasicString<CharT, Traits, A>
where
    BasicStr<CharT, Traits>: AsRef<S>,
{
    fn as_ref(&self) -> &S {
        BasicStr::as_ref(self)
    }
}

impl<S: ?Sized, CharT, Traits, A: Allocator> AsMut<S> for BasicString<CharT, Traits, A>
where
    BasicStr<CharT, Traits>: AsMut<S>,
{
    fn as_mut(&mut self) -> &mut S {
        BasicStr::as_mut(self)
    }
}

impl<CharT: Eq, Traits, A: Allocator> PartialEq for BasicString<CharT, Traits, A> {
    fn eq(&self, other: &Self) -> bool {
        BasicStr::eq(&**self, &**other)
    }
}

impl<CharT: Eq, Traits, A: Allocator> PartialEq<BasicStr<CharT, Traits>>
    for BasicString<CharT, Traits, A>
{
    fn eq(&self, other: &BasicStr<CharT, Traits>) -> bool {
        BasicStr::eq(&**self, other)
    }
}

impl<CharT: Eq, Traits, A: Allocator> PartialEq<BasicString<CharT, Traits, A>>
    for BasicStr<CharT, Traits>
{
    fn eq(&self, other: &BasicString<CharT, Traits, A>) -> bool {
        BasicStr::eq(self, &**other)
    }
}

impl<CharT: Eq, Traits, A: Allocator> Eq for BasicString<CharT, Traits, A> {}

impl<Traits: CharTraits, A: Allocator> PartialOrd for BasicString<Traits::Char, Traits, A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        BasicStr::partial_cmp(&**self, &**other)
    }
}

impl<Traits: CharTraits, A: Allocator> PartialOrd<BasicStr<Traits::Char, Traits>>
    for BasicString<Traits::Char, Traits, A>
{
    fn partial_cmp(&self, other: &BasicStr<Traits::Char, Traits>) -> Option<Ordering> {
        BasicStr::partial_cmp(&**self, other)
    }
}

impl<Traits: CharTraits, A: Allocator> PartialOrd<BasicString<Traits::Char, Traits, A>>
    for BasicStr<Traits::Char, Traits>
{
    fn partial_cmp(&self, other: &BasicString<Traits::Char, Traits, A>) -> Option<Ordering> {
        BasicStr::partial_cmp(self, &**other)
    }
}

impl<Traits: CharTraits, A: Allocator> Ord for BasicString<Traits::Char, Traits, A> {
    fn cmp(&self, other: &Self) -> Ordering {
        BasicStr::cmp(self, other)
    }
}

impl<CharT: Hash, Traits, A: Allocator> Hash for BasicString<CharT, Traits, A> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        BasicStr::hash(self, state);
    }
}

#[cfg(feature = "utf")]
pub type UtfString<CharT, A = Global> = BasicString<CharT, crate::utf::UtfCharTraits<CharT>, A>;

#[cfg(feature = "utf")]
pub type String = UtfString<u8>;
#[cfg(feature = "utf")]
pub type U16String = UtfString<u16>;
#[cfg(feature = "utf")]
pub type U32String = UtfString<char>;

#[cfg(feature = "utf")]
impl String {
    pub fn from_utf8(st: alloc::string::String) -> Self {
        unsafe { Self::from_chars_unchecked(st.into_bytes()) }
    }

    pub fn into_utf8(self) -> alloc::string::String {
        unsafe { alloc::string::String::from_utf8_unchecked(self.into_chars()) }
    }
}
