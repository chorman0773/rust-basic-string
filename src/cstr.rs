use core::{cmp::Ordering, hash::Hash, marker::PhantomData};

use crate::{
    str::BasicStr,
    traits::{Char, CharTraits, DebugStr, DisplayStr},
};

#[cfg(feature = "utf")]
use crate::utf::UtfCharTraits;

#[repr(transparent)]
pub struct BasicCStr<CharT, Traits>(PhantomData<Traits>, [CharT]);

impl<CharT, Traits> BasicCStr<CharT, Traits> {
    /// Converts from a slice of `CharT` to a [`BasicCStr`] without validating the array, or checking for null terminators
    ///
    /// # Safety
    /// The following preconditions must hold:
    /// * `chars.last()` must refer to the value designated by `Traits::zero_term()`
    /// * `Traits::validate_range` must not return an error for `chars`
    /// * No character before the last may satisfy `Traits::is_zero_term()`
    pub const unsafe fn from_chars_with_null_unchecked(chars: &[CharT]) -> &Self {
        &*(chars as *const [CharT] as *const Self)
    }

    /// Converts from a slice of `CharT` to a [`BasicCStr`] without validating the array, or checking for null terminators
    ///
    /// # Safety
    /// The following preconditions must hold:
    /// * `chars.last()` must refer to the value designated by `Traits::zero_term()`
    /// * `Traits::validate_range` must not return an error for `chars`
    /// * No character before the last may satisfy `Traits::is_zero_term()`
    pub unsafe fn from_chars_with_null_unchecked_mut(chars: &mut [CharT]) -> &mut Self {
        &mut *(chars as *mut [CharT] as *mut Self)
    }

    /// Converts a [`BasicCStr`] into a slice of `CharT` (including the null terminator)
    pub const fn as_chars(&self) -> &[CharT] {
        &self.1
    }

    /// Converts a [`BasicCStr`] into a slice of `CharT` (including the null terminator)
    /// # Safety
    /// The return value must not be used to modify the slice in a way that violates the preconditions on [`BasicCStr::from_chars_with_null_unchecked_mut`].
    pub unsafe fn as_chars_mut(&mut self) -> &mut [CharT] {
        &mut self.1
    }

    /// Returns a pointer to a null-terminated string consisting of the characters of `self`.
    /// The returned pointer is valid for at least as long as the lifetime of `self`.
    ///
    /// # Safety
    /// The returned pointer must not be used to mutate the contents of the string
    pub const fn as_ptr(&self) -> *const CharT {
        &self.1 as *const [CharT] as *const CharT
    }

    /// Returns a mutable pointer to a null-terminated string consisting of the characters of `self`.
    /// The returned pointer is valid for at least as long as the lifetime of `self`
    ///
    /// # Safety
    /// The returned pointer must not be used to mutate the contents of the string in any way that violates the preconditions on [`BasicCStr::from_chars_with_null_unchecked_mut`]..
    pub fn as_mut_ptr(&mut self) -> *mut CharT {
        &mut self.1 as *mut [CharT] as *mut CharT
    }

    pub const fn len(&self) -> usize {
        self.1.len()
    }

    ///
    /// Converts the `CStr` into a `Str` that includes the zero terminator.
    /// This may
    pub const fn as_basic_str_with_nul(&self) -> &BasicStr<CharT, Traits> {
        // SAFETY:
        // We're validiated w/ the zero terminator
        unsafe { BasicStr::from_chars_unchecked(self.as_chars()) }
    }

    pub const fn as_basic_str_without_nul(&self) -> &BasicStr<CharT, Traits> {
        if let Some((_, rest)) = self.as_chars().split_last() {
            unsafe { BasicStr::from_chars_unchecked(rest) }
        } else {
            unsafe { core::hint::unreachable_unchecked() }
        }
    }
}

#[cfg(feature = "const-from-chars")]
include!("cstr_from_chars_const.rs");
#[cfg(not(feature = "const-from-chars"))]
include!("cstr_from_chars.rs");

impl<Traits: CharTraits> BasicCStr<Traits::Char, Traits> {
    /// Converts a mutable slice of `CharT` to a [`BasicCStr`] if the following hold:
    /// * The last character of the slice is a zero terminator, according to [`CharTraits::is_zero_term`]
    /// * No character other than the last is a zero terminator, according to [`CharTraits::is_zero_term`]
    /// * The array (including the zero terminator) is valid according to [`CharTraits::validate_range`]
    ///
    /// Otherwise, returns `None`
    pub fn from_chars_with_null_mut(chars: &mut [Traits::Char]) -> Option<&mut Self> {
        match chars.last() {
            Some(c) if Traits::is_zero_term(*c) => return None,
            None => return None,
            Some(_) => {}
        }

        let mut i = 0;
        while i < (chars.len() - 1) {
            if Traits::is_zero_term(chars[i]) {
                return None;
            }
            i += 1;
        }

        match Traits::validate_range(chars) {
            Ok(()) => Some(unsafe { Self::from_chars_with_null_unchecked_mut(chars) }),
            Err(e) => {
                core::mem::forget(e);
                None
            }
        }
    }

    ///
    /// Converts the shortest null terminated subrange of `chars` into a [`BasicCStr`].
    ///
    /// Returns that converted cstr and the remainder of the range if it is valid (according to [`CharTraits::validate_range`])
    pub fn split_from_chars(chars: &[Traits::Char]) -> Option<(&Self, &[Traits::Char])> {
        for (i, &c) in chars.iter().enumerate() {
            if Traits::is_zero_term(c) {
                let (left, right) = chars.split_at(i + 1);

                return Traits::validate_range(left)
                    .map(|_| (unsafe { Self::from_chars_with_null_unchecked(left) }, right))
                    .ok();
            }
        }

        None
    }

    ///
    /// Converts the shortest null terminated subrange of `chars` into a [`BasicCStr`].
    ///
    /// Returns that converted cstr and the remainder of the range if it is valid (according to [`CharTraits::validate_range`])
    pub fn split_from_chars_mut(
        chars: &mut [Traits::Char],
    ) -> Option<(&mut Self, &mut [Traits::Char])> {
        for (i, &c) in chars.iter().enumerate() {
            if Traits::is_zero_term(c) {
                let (left, right) = chars.split_at_mut(i + 1);

                match Traits::validate_range(left) {
                    Ok(()) => {
                        return Some((
                            unsafe { Self::from_chars_with_null_unchecked_mut(left) },
                            right,
                        ))
                    }
                    Err(_) => return None,
                }
            }
        }

        None
    }

    /// Obtains a [`BasicCStr`] slice over the null terminated string starting at `begin`.
    ///
    /// # Safety
    /// The following preconditions must hold:
    /// * There exists some `i` such that `begin.offset(i).read()` is a zero terminator, according to [`CharTraits::zero_term`],
    /// * `[begin,begin.offset(i+1))` shall be a range which is valid and not modified for the duration of `'a`
    /// * The characters in that range form a valid string according to [`CharTraits::validate_range`]
    pub unsafe fn from_raw<'a>(begin: *const Traits::Char) -> &'a Self {
        let mut end = begin;
        while *end != Traits::zero_term() {
            end = end.offset(1);
        }
        end = end.offset(1);

        Self::from_chars_with_null_unchecked(core::slice::from_raw_parts(
            begin,
            end.offset_from(begin) as usize,
        ))
    }

    /// Obtains a mut [`BasicCStr`] slice over the null terminated string starting at `begin`.
    ///
    /// # Safety
    /// The following preconditions must hold:
    /// * There exists some `i` such that `begin.offset(i).read()` is a zero terminator, according to [`CharTraits::zero_term`],
    /// * `[begin,begin.offset(i+1))` shall be a range which is valid and not aliased for the duration of `'a``
    /// * The characters in that range form a valid string according to [`CharTraits::validate_range`]
    pub unsafe fn from_raw_mut<'a>(begin: *mut Traits::Char) -> &'a mut Self {
        let mut end = begin;
        while *end != Traits::zero_term() {
            end = end.offset(1);
        }
        end = end.offset(1);

        Self::from_chars_with_null_unchecked_mut(core::slice::from_raw_parts_mut(
            begin,
            end.offset_from(begin) as usize,
        ))
    }

    /// Obtains a [`BasicCStr`] slice over the null terminated string starting at `begin`, if it is valid according to [`CharTraits::validate_range`]
    ///
    /// # Errors
    /// Returns an error if the string starting from `begin` is not valid according to [`CharTraits::validate_range`]
    ///
    /// # Safety
    /// The following preconditions must hold:
    /// * There exists some `i` such that `begin.offset(i).read()` is a zero terminator, according to [`CharTraits::zero_term`],
    /// * `[begin,begin.offset(i+1))` shall be a range which is valid and not modified for the duration of `'a`
    pub unsafe fn validate_from_raw<'a>(
        begin: *const Traits::Char,
    ) -> Result<&'a Self, Traits::Error> {
        let mut end = begin;
        while *end != Traits::zero_term() {
            end = end.offset(1);
        }
        end = end.offset(1);
        let slice = core::slice::from_raw_parts(begin, end.offset_from(begin) as usize);
        Traits::validate_range(slice)?;
        Ok(Self::from_chars_with_null_unchecked(slice))
    }

    /// Obtains a [`BasicCStr`] slice over the null terminated string starting at `begin`, if it is valid according to [`CharTraits::validate_range`]
    ///
    /// # Errors
    /// Returns an error if the string starting from `begin` is not valid according to [`CharTraits::validate_range`]
    ///
    /// # Safety
    /// The following preconditions must hold:
    /// * There exists some `i` such that `begin.offset(i).read()` is a zero terminator, according to [`CharTraits::zero_term`],
    /// * `[begin,begin.offset(i+1))` shall be a range which is valid and not aliased for the duration of `'a`
    pub unsafe fn validate_from_raw_mut<'a>(
        begin: *mut Traits::Char,
    ) -> Result<&'a mut Self, Traits::Error> {
        let mut end = begin;
        while *end != Traits::zero_term() {
            end = end.offset(1);
        }
        end = end.offset(1);
        let slice = core::slice::from_raw_parts_mut(begin, end.offset_from(begin) as usize);
        Traits::validate_range(slice)?;
        Ok(Self::from_chars_with_null_unchecked_mut(slice))
    }
}

pub struct RawCharTraits;

#[cfg(not(feature = "const-raw-char-traits"))]
impl CharTraits for RawCharTraits {
    type Char = u8;
    type Int = i32;
    type Error = core::convert::Infallible;

    fn validate_range(_: &[Self::Char]) -> Result<(), Self::Error> {
        Ok(())
    }

    unsafe fn validate_subrange(_: &[Self::Char]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn compare(r1: &[Self::Char], r2: &[Self::Char]) -> Result<core::cmp::Ordering, Self::Error> {
        let mut i = 0;
        while i < r1.len() && i < r2.len() {
            if r1[i] < r2[i] {
                return Ok(Ordering::Less);
            } else if r1[i] == r2[i] {
                i += 1;
                continue;
            } else {
                return Ok(Ordering::Greater);
            }
        }

        if r1.len() < r2.len() {
            Ok(Ordering::Less)
        } else if r1.len() == r2.len() {
            Ok(Ordering::Equal)
        } else {
            Ok(Ordering::Greater)
        }
    }

    fn zero_term() -> Self::Char {
        0
    }

    fn is_zero_term(c: Self::Char) -> bool {
        c == 0
    }

    fn eof() -> Self::Int {
        -1
    }
}

#[cfg(feature = "const-raw-char-traits")]
include!("cstr_raw_traits_const.rs");

pub type CStr = BasicCStr<u8, RawCharTraits>;

impl<CharT: Char, Traits: CharTraits<Char = CharT> + DebugStr> core::fmt::Debug
    for BasicCStr<CharT, Traits>
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        unsafe { Traits::debug_range_unchecked(self.as_chars().split_last().unwrap().1, fmt) }
    }
}

impl<CharT: Char, Traits: CharTraits<Char = CharT> + DisplayStr> core::fmt::Display
    for BasicCStr<CharT, Traits>
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        unsafe { Traits::display_range_unchecked(self.as_chars().split_last().unwrap().1, fmt) }
    }
}

#[cfg(feature = "utf")]
pub type UtfCStr<CharT> = BasicCStr<CharT, UtfCharTraits<CharT>>;

#[cfg(feature = "utf")]
pub type Utf8CStr = UtfCStr<u8>;

#[cfg(feature = "utf")]
pub type Utf16CStr = UtfCStr<u16>;

#[cfg(feature = "utf")]
pub type Utf32CStr = UtfCStr<char>;

impl<Traits: CharTraits> Ord for BasicCStr<Traits::Char, Traits> {
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe { Traits::compare(self.as_chars(), other.as_chars()).unwrap_unchecked() }
    }
}

impl<CharT: Eq, Traits> PartialEq for BasicCStr<CharT, Traits> {
    fn eq(&self, other: &Self) -> bool {
        self.as_chars() == other.as_chars()
    }
}

impl<Traits: CharTraits> PartialOrd for BasicCStr<Traits::Char, Traits> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<CharT: Eq, Traits> Eq for BasicCStr<CharT, Traits> {}

impl<CharT: Hash, Traits> Hash for BasicCStr<CharT, Traits> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.1.hash(state);
    }
}
