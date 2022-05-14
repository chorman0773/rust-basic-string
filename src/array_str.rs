use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

use crate::str::BasicStr;

use crate::traits::CharTraits;
use crate::utf::UtfCharTraits;

pub struct BasicArrayString<CharT, Traits, const N: usize>([CharT; N], PhantomData<Traits>);

impl<CharT: Copy, Traits, const N: usize> Copy for BasicArrayString<CharT, Traits, N> {}

impl<CharT: Clone, Traits, const N: usize> Clone for BasicArrayString<CharT, Traits, N> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<CharT: PartialEq, Traits, const N: usize, const M: usize>
    PartialEq<BasicArrayString<CharT, Traits, M>> for BasicArrayString<CharT, Traits, N>
{
    fn eq(&self, rhs: &BasicArrayString<CharT, Traits, M>) -> bool {
        self.as_chars() == rhs.as_chars()
    }
}

impl<CharT: PartialEq, Traits, const N: usize> PartialEq<BasicStr<CharT, Traits>>
    for BasicArrayString<CharT, Traits, N>
{
    fn eq(&self, rhs: &BasicStr<CharT, Traits>) -> bool {
        &self.0 == rhs.as_chars()
    }
}

impl<CharT: Eq, Traits, const N: usize> Eq for BasicArrayString<CharT, Traits, N> {}

impl<Traits: CharTraits, const N: usize, const M: usize>
    PartialOrd<BasicArrayString<Traits::Char, Traits, M>>
    for BasicArrayString<Traits::Char, Traits, N>
{
    fn partial_cmp(&self, rhs: &BasicArrayString<Traits::Char, Traits, M>) -> Option<Ordering> {
        Some(unsafe { Traits::compare(&self.0, &rhs.0).unwrap_unchecked() })
    }
}

impl<Traits: CharTraits, const N: usize> PartialOrd<BasicStr<Traits::Char, Traits>>
    for BasicArrayString<Traits::Char, Traits, N>
{
    fn partial_cmp(&self, rhs: &BasicStr<Traits::Char, Traits>) -> Option<Ordering> {
        Some(unsafe { Traits::compare(&self.0, rhs.as_chars()).unwrap_unchecked() })
    }
}

impl<Traits: CharTraits, const N: usize> Ord for BasicArrayString<Traits::Char, Traits, N> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        unsafe { Traits::compare(&self.0, &rhs.0).unwrap_unchecked() }
    }
}

impl<CharT, CharTraits, const N: usize> BasicArrayString<CharT, CharTraits, N> {
    #[inline]
    pub const unsafe fn from_chars_unchecked(chars: [CharT; N]) -> Self {
        Self(chars, PhantomData)
    }

    #[inline]
    pub fn as_chars(&self) -> &[CharT] {
        &self.0
    }

    #[inline]
    pub fn into_chars(self) -> [CharT; N] {
        self.0
    }

    #[inline]
    pub unsafe fn as_chars_mut(&mut self) -> &mut [CharT] {
        &mut self.0
    }

    #[inline]
    pub fn as_basic_str(&self) -> &BasicStr<CharT, CharTraits> {
        // SAFETY: We are already valid, by the invariant of BasicArrayString
        unsafe { BasicStr::from_chars_unchecked(&self.0) }
    }

    #[inline]
    pub fn as_basic_str_mut(&mut self) -> &mut BasicStr<CharT, CharTraits> {
        // SAFETY: We are already valid, by the invariant of BasicArrayString
        unsafe { BasicStr::from_chars_unchecked_mut(&mut self.0) }
    }
}

impl<Traits: CharTraits, const N: usize> BasicArrayString<Traits::Char, Traits, N> {
    pub fn from_chars(chars: [Traits::Char; N]) -> Result<Self, Traits::Error> {
        match Traits::validate_range(&chars) {
            Ok(()) => Ok(unsafe { Self::from_chars_unchecked(chars) }),
            Err(e) => Err(e),
        }
    }
}

impl<CharT, Traits, const N: usize> Deref for BasicArrayString<CharT, Traits, N> {
    type Target = BasicStr<CharT, Traits>;

    fn deref(&self) -> &Self::Target {
        self.as_basic_str()
    }
}

impl<CharT, Traits, const N: usize> DerefMut for BasicArrayString<CharT, Traits, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_basic_str_mut()
    }
}

impl<CharT, Traits, const N: usize> AsRef<BasicStr<CharT, Traits>>
    for BasicArrayString<CharT, Traits, N>
{
    fn as_ref(&self) -> &BasicStr<CharT, Traits> {
        self
    }
}

impl<CharT, Traits, const N: usize> AsRef<[CharT]> for BasicArrayString<CharT, Traits, N> {
    fn as_ref(&self) -> &[CharT] {
        self.as_chars()
    }
}

impl<CharT, Traits, const N: usize> Borrow<BasicStr<CharT, Traits>>
    for BasicArrayString<CharT, Traits, N>
{
    fn borrow(&self) -> &BasicStr<CharT, Traits> {
        self
    }
}

impl<CharT, Traits, const N: usize> AsMut<BasicStr<CharT, Traits>>
    for BasicArrayString<CharT, Traits, N>
{
    fn as_mut(&mut self) -> &mut BasicStr<CharT, Traits> {
        self
    }
}

impl<CharT, Traits, const N: usize> BorrowMut<BasicStr<CharT, Traits>>
    for BasicArrayString<CharT, Traits, N>
{
    fn borrow_mut(&mut self) -> &mut BasicStr<CharT, Traits> {
        self
    }
}

#[cfg(feature = "utf")]
pub type UtfArrayString<CharT, const N: usize> = BasicArrayString<CharT, UtfCharTraits<CharT>, N>;

#[cfg(feature = "utf")]
pub type ArrayString<const N: usize> = UtfArrayString<u8, N>;

#[cfg(feature = "utf")]
impl<const N: usize> AsRef<str> for ArrayString<N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct LengthError<const N: usize>(usize);

impl<const N: usize> core::fmt::Display for LengthError<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "Unexpected length of array {}, required length is {}",
            self.0, N
        ))
    }
}

#[cfg(feature = "utf")]
impl<const N: usize> TryFrom<&str> for ArrayString<N> {
    type Error = LengthError<N>;
    fn try_from(x: &str) -> Result<Self, Self::Error> {
        if x.len() != N {
            Err(LengthError(x.len()))
        } else {
            Ok(unsafe {
                Self::from_chars_unchecked(*(x.as_bytes() as *const [u8] as *const [u8; N]))
            })
        }
    }
}

#[cfg(feature = "utf")]
impl<const N: usize> From<[char; N]> for U32ArrayString<N> {
    fn from(x: [char; N]) -> Self {
        // SAFETY: `UtfCharTraits<char>::validate_range` is infallible
        unsafe { Self::from_chars_unchecked(x) }
    }
}

impl<const N: usize> AsMut<[char]> for U32ArrayString<N> {
    fn as_mut(&mut self) -> &mut [char] {
        &mut self.0
    }
}

#[cfg(feature = "utf")]
pub type U16ArrayString<const N: usize> = UtfArrayString<u16, N>;
#[cfg(feature = "utf")]
pub type U32ArrayString<const N: usize> = UtfArrayString<char, N>;

#[macro_export]
#[cfg(feature = "utf")]
macro_rules! const_array_str {
    ($lit:literal) => {{
        const __STR: &::core::primitive::str = $lit;
        type ArrayType = [u8; __STR.len()];
        const __RET: $crate::array_str::ArrayString<{ __STR.len() }> = {
            let bytes = __STR.as_bytes();

            unsafe {
                $crate::array_str::ArrayString::from_chars_unchecked(
                    *(bytes as *const [u8] as *const ArrayType),
                )
            }
        };

        __RET
    }};
}
