use core::cmp::Ordering;
use core::hash::Hash;
use core::ops::{Index, IndexMut};
use core::{marker::PhantomData, slice::SliceIndex};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

#[cfg(feature = "pattern")]
use crate::pattern::{BidirectionalPattern, Pattern, RevPattern};
use crate::traits::IntoChars;
use crate::traits::{Char, CharTraits, DebugStr, DisplayStr};

#[cfg(feature = "utf")]
use crate::utf::UtfCharTraits;

#[repr(transparent)]
pub struct BasicStr<CharT, CharTraits>(PhantomData<CharTraits>, [CharT]);

impl<CharT, Traits> BasicStr<CharT, Traits> {
    /// Returns the underlying array for the [`BasicStr`] as a borrowed slice of `CharT`
    pub const fn as_chars(&self) -> &[CharT] {
        &self.1
    }

    /// Returns the underlying array for the [`BasicStr`] as a mutably borrowed slice of `CharT`
    ///
    /// # Safety
    /// The result slice shall not be modified to be invalid according to [`CharTraits::validate_range`]
    ///
    pub unsafe fn as_chars_mut(&mut self) -> &mut [CharT] {
        &mut self.1
    }

    /// Converts a slice of `CharT` into a [`BasicStr`] without validating the slice
    ///
    /// # Safety
    ///
    /// The `chars` shall be valid according to [`CharTraits::validate_range`]
    pub const unsafe fn from_chars_unchecked(chars: &[CharT]) -> &Self {
        &*(chars as *const [CharT] as *const Self)
    }

    /// Converts a mutable slice of `CharT` into a [`BasicStr`] without validating the slice
    ///
    /// # Safety
    ///
    /// The `chars` shall be valid according to [`CharTraits::validate_range`]
    pub unsafe fn from_chars_unchecked_mut(chars: &mut [CharT]) -> &mut Self {
        &mut *(chars as *mut [CharT] as *mut Self)
    }

    /// Converts a boxed slice of `CharT` into a [`BasicStr`] without validating the slice
    ///
    /// # Safety
    ///
    /// The `chars` shall be valid according to [`CharTraits::validate_range`]
    #[cfg(all(feature = "alloc", not(feature = "allocator-api")))]
    pub unsafe fn from_boxed_chars_unchecked(chars: Box<[CharT]>) -> Box<Self> {
        let ptr = Box::into_raw(chars);
        Box::from_raw(ptr as *mut Self)
    }

    /// Converts a boxed slice of `CharT` into a [`BasicStr`] without validating the slice
    ///
    /// # Safety
    ///
    /// The `chars` shall be valid according to [`CharTraits::validate_range`]
    #[cfg(all(feature = "alloc", feature = "allocator-api"))]
    pub unsafe fn from_boxed_chars_unchecked<A: alloc::alloc::Allocator>(
        chars: Box<[CharT], A>,
    ) -> Box<Self, A> {
        let (ptr, alloc) = Box::into_raw_with_allocator(chars);
        Box::from_raw_in(ptr as *mut Self, alloc)
    }

    pub const fn len(&self) -> usize {
        self.1.len()
    }

    pub const fn as_ptr(&self) -> *const CharT {
        self.1.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut CharT {
        self.1.as_mut_ptr()
    }
}

#[cfg(feature = "pattern")]
impl<CharT, Traits> BasicStr<CharT, Traits> {
    pub fn split_once<'a, P: Pattern<CharT, Traits>>(
        &'a self,
        delimiter: P,
    ) -> Option<(&'a Self, &'a Self)> {
        // Safety:
        // `self` is valid by invariant
        let pat = unsafe { delimiter.first_match_unchecked(&self.1) }?;

        // Safety:
        // Guaranteed by the `Pattern` impl
        let begin = unsafe { pat.as_ptr().offset_from(self.as_ptr()) } as usize;

        let end = begin + pat.len();

        // Safety:
        // Guaranteed by the `Pattern` impl
        Some(unsafe {
            (
                Self::from_chars_unchecked(self.1.get_unchecked(..begin)),
                Self::from_chars_unchecked(self.1.get_unchecked(end..)),
            )
        })
    }

    pub fn find<P: Pattern<CharT, Traits>>(&self, pat: P) -> Option<usize> {
        // Safety:
        // `self` is valid by invariant
        let pat = unsafe { pat.first_match_unchecked(&self.1) }?;

        // Safety:
        // Guaranteed by the `Pattern` impl
        let begin = unsafe { pat.as_ptr().offset_from(self.as_ptr()) } as usize;

        Some(begin)
    }

    pub fn rsplit_once<'a, P: RevPattern<CharT, Traits>>(
        &'a self,
        delimiter: P,
    ) -> Option<(&'a Self, &'a Self)> {
        // Safety:
        // `self` is valid by invariant
        let pat = unsafe { delimiter.last_match_unchecked(&self.1) }?;

        // Safety:
        // Guaranteed by the `Pattern` impl
        let begin = unsafe { pat.as_ptr().offset_from(self.as_ptr()) } as usize;

        let end = begin + pat.len();

        // Safety:
        // Guaranteed by the `Pattern` impl
        Some(unsafe {
            (
                Self::from_chars_unchecked(self.1.get_unchecked(..begin)),
                Self::from_chars_unchecked(self.1.get_unchecked(end..)),
            )
        })
    }

    pub fn rfind<P: RevPattern<CharT, Traits>>(&self, pat: P) -> Option<usize> {
        // Safety:
        // `self` is valid by invariant
        let pat = unsafe { pat.last_match_unchecked(&self.1) }?;

        // Safety:
        // Guaranteed by the `Pattern` impl
        let begin = unsafe { pat.as_ptr().offset_from(self.as_ptr()) } as usize;

        Some(begin)
    }

    pub fn split<P: Pattern<CharT, Traits>>(&self, pat: P) -> Split<P, CharT, Traits> {
        Split(Some(self), pat)
    }

    pub fn rsplit<P: RevPattern<CharT, Traits>>(&self, pat: P) -> RSplit<P, CharT, Traits> {
        RSplit(Some(self), pat)
    }
}

#[cfg(feature = "pattern")]
pub struct Split<'a, P, CharT, Traits>(Option<&'a BasicStr<CharT, Traits>>, P);

#[cfg(feature = "pattern")]
pub struct RSplit<'a, P, CharT, Traits>(Option<&'a BasicStr<CharT, Traits>>, P);

#[cfg(feature = "pattern")]
impl<'a, P, CharT, Traits> Iterator for Split<'a, P, CharT, Traits>
where
    P: Pattern<CharT, Traits>,
{
    type Item = &'a BasicStr<CharT, Traits>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(test) = self.0.take() {
            let pat = unsafe { self.1.first_match_unchecked(test.as_chars()) };

            if let Some(pat) = pat {
                // Safety:
                // Guaranteed by the `Pattern` impl
                let begin = unsafe { pat.as_ptr().offset_from(test.as_ptr()) } as usize;

                let end = begin + pat.len();
                self.0 = Some(unsafe { BasicStr::from_chars_unchecked(&test.as_chars()[end..]) });
                Some(unsafe { BasicStr::from_chars_unchecked(&test.as_chars()[..begin]) })
            } else {
                Some(test)
            }
        } else {
            None
        }
    }
}

#[cfg(feature = "pattern")]
impl<'a, P, CharT, Traits> Iterator for RSplit<'a, P, CharT, Traits>
where
    P: RevPattern<CharT, Traits>,
{
    type Item = &'a BasicStr<CharT, Traits>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(test) = self.0.take() {
            let pat = unsafe { self.1.last_match_unchecked(test.as_chars()) };

            if let Some(pat) = pat {
                // Safety:
                // Guaranteed by the `Pattern` impl
                let begin = unsafe { pat.as_ptr().offset_from(test.as_ptr()) } as usize;

                let end = begin + pat.len();
                self.0 = Some(unsafe { BasicStr::from_chars_unchecked(&test.as_chars()[..begin]) });

                Some(unsafe { BasicStr::from_chars_unchecked(&test.as_chars()[end..]) })
            } else {
                Some(test)
            }
        } else {
            None
        }
    }
}

#[cfg(feature = "pattern")]
impl<'a, P, CharT, Traits> DoubleEndedIterator for Split<'a, P, CharT, Traits>
where
    P: BidirectionalPattern<CharT, Traits>,
{
    fn next_back(&mut self) -> Option<<Self as Iterator>::Item> {
        if let Some(test) = self.0.take() {
            let pat = unsafe { self.1.last_match_unchecked(test.as_chars()) };

            if let Some(pat) = pat {
                // Safety:
                // Guaranteed by the `Pattern` impl
                let begin = unsafe { pat.as_ptr().offset_from(test.as_ptr()) } as usize;

                let end = begin + pat.len();
                self.0 = Some(unsafe { BasicStr::from_chars_unchecked(&test.as_chars()[..begin]) });

                Some(unsafe { BasicStr::from_chars_unchecked(&test.as_chars()[end..]) })
            } else {
                Some(test)
            }
        } else {
            None
        }
    }
}

#[cfg(feature = "pattern")]
impl<'a, P, CharT, Traits> DoubleEndedIterator for RSplit<'a, P, CharT, Traits>
where
    P: BidirectionalPattern<CharT, Traits>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(test) = self.0.take() {
            let pat = unsafe { self.1.first_match_unchecked(test.as_chars()) };

            if let Some(pat) = pat {
                // Safety:
                // Guaranteed by the `Pattern` impl
                let begin = unsafe { pat.as_ptr().offset_from(test.as_ptr()) } as usize;

                let end = begin + pat.len();
                self.0 = Some(unsafe { BasicStr::from_chars_unchecked(&test.as_chars()[end..]) });
                Some(unsafe { BasicStr::from_chars_unchecked(&test.as_chars()[..begin]) })
            } else {
                Some(test)
            }
        } else {
            None
        }
    }
}

#[cfg(feature = "alloc")]
impl<CharT, Traits> From<Box<BasicStr<CharT, Traits>>> for Box<[CharT]> {
    fn from(b: Box<BasicStr<CharT, Traits>>) -> Self {
        let ptr = Box::into_raw(b);
        unsafe { Box::from_raw(ptr as *mut [CharT]) }
    }
}

#[cfg(all(feature = "alloc"))]
impl<CharT, Traits> BasicStr<CharT, Traits> {
    #[cfg(feature = "allocator-api")]
    pub fn into_boxed_chars<A: alloc::alloc::Allocator>(
        self: Box<BasicStr<CharT, Traits>, A>,
    ) -> Box<[CharT], A> {
        let (ptr, alloc) = Box::into_raw_with_allocator(self);
        unsafe { Box::from_raw_in(ptr as *mut [CharT], alloc) }
    }

    #[cfg(not(feature = "allocator-api"))]
    pub fn into_boxed_chars(self: Box<BasicStr<CharT, Traits>>) -> Box<[CharT]> {
        let ptr = Box::into_raw(self);
        unsafe { Box::from_raw(ptr as *mut [CharT]) }
    }
}

#[cfg(feature = "const-from-chars")]
include!("str_from_chars_const.rs");

#[cfg(not(feature = "const-from-chars"))]
include!("str_from_chars.rs");

impl<CharT, Traits: CharTraits<Char = CharT>> BasicStr<CharT, Traits> {
    pub fn from_chars_mut(chars: &mut [CharT]) -> Result<&mut Self, Traits::Error> {
        Traits::validate_range(chars)?;
        // SAFETY:
        // [`CharTraits::validate_range`] has already validated the range
        Ok(unsafe { Self::from_chars_unchecked_mut(chars) })
    }

    pub fn get<I: SliceIndex<[CharT], Output = [CharT]>>(&self, range: I) -> Option<&Self> {
        let range = self.1.get(range)?;

        // SAFETY:
        // self.1 is a valid range of `[CharT]` according to `Traits`.
        // Thus, validate_subrange is sufficient to prove we've successfully sliced the BasicStr in a way that is valid for Self
        unsafe {
            Traits::validate_subrange(range).ok()?;
            Some(Self::from_chars_unchecked(range))
        }
    }

    pub fn get_mut<I: SliceIndex<[CharT], Output = [CharT]>>(
        &mut self,
        range: I,
    ) -> Option<&mut Self> {
        let range = self.1.get_mut(range)?;

        // SAFETY:
        // self.1 is a valid range of `[CharT]` according to `Traits`.
        // Thus, validate_subrange is sufficient to prove we've successfully sliced the BasicStr in a way that is valid for Self
        unsafe {
            Traits::validate_subrange(range).ok()?;
            Some(Self::from_chars_unchecked_mut(range))
        }
    }

    ///
    /// Slices `self` without checking the range bounds or validity
    ///
    /// # Safety
    ///
    /// Let `start` be the beginning of the `range`, and `end` be the end of that `range`:
    /// * `start` shall be less than or equal to `end`
    /// * `end` shall be less than `self.len()`
    /// * The resulting range of `self.as_chars()` slice by `range` shall be valid according to [`CharTraits::validate_range`]
    pub unsafe fn get_unchecked<I: SliceIndex<[CharT], Output = [CharT]>>(
        &self,
        range: I,
    ) -> &Self {
        Self::from_chars_unchecked(self.1.get_unchecked(range))
    }
    ///
    /// Slices `self` mutably without checking the range bounds or validity
    ///
    /// # Safety
    ///
    /// Let `start` be the beginning of the `range`, and `end` be the end of that `range`:
    /// * `start` shall be less than or equal to `end`
    /// * `end` shall be less than `self.len()`
    /// * The resulting range of `self.as_mut_chars()` slice by `range` shall be valid according to [`CharTraits::validate_range`]
    pub unsafe fn get_unchecked_mut<I: SliceIndex<[CharT], Output = [CharT]>>(
        &mut self,
        range: I,
    ) -> &mut Self {
        Self::from_chars_unchecked_mut(self.1.get_unchecked_mut(range))
    }

    pub fn split_at(&self, mid: usize) -> (&Self, &Self) {
        let (left, right) = self.1.split_at(mid);

        unsafe { Traits::validate_subrange(left) }.unwrap();
        unsafe { Traits::validate_subrange(right) }.unwrap();

        unsafe {
            (
                Self::from_chars_unchecked(left),
                Self::from_chars_unchecked(right),
            )
        }
    }

    pub fn split_at_mut(&mut self, mid: usize) -> (&mut Self, &mut Self) {
        let (left, right) = self.1.split_at_mut(mid);

        unsafe { Traits::validate_subrange(left) }.unwrap();
        unsafe { Traits::validate_subrange(right) }.unwrap();

        unsafe {
            (
                Self::from_chars_unchecked_mut(left),
                Self::from_chars_unchecked_mut(right),
            )
        }
    }
}

#[cfg(not(feature = "const-trait-impl"))]
impl<'a, C, Traits> Default for &'a BasicStr<C, Traits>
where
    BasicStr<C, Traits>: 'a,
{
    fn default() -> &'a BasicStr<C, Traits> {
        unsafe { BasicStr::from_chars_unchecked(&[]) }
    }
}

#[cfg(not(feature = "const-trait-impl"))]
impl<C, Traits> AsRef<[C]> for BasicStr<C, Traits> {
    fn as_ref(&self) -> &[C] {
        self.as_chars()
    }
}

#[cfg(feature = "const-trait-impl")]
include!("str_const_trait_impl.rs");

impl<C: Char, Traits: CharTraits<Char = C>, I: SliceIndex<[C], Output = [C]>> Index<I>
    for BasicStr<C, Traits>
{
    type Output = BasicStr<C, Traits>;
    fn index(&self, idx: I) -> &BasicStr<C, Traits> {
        let chars = &self.1[idx];

        unsafe {
            Traits::validate_subrange(chars).expect("Attempt to index str to produce invalid range")
        }

        unsafe { Self::from_chars_unchecked(chars) }
    }
}

impl<C: Char, Traits: CharTraits<Char = C>, I: SliceIndex<[C], Output = [C]>> IndexMut<I>
    for BasicStr<C, Traits>
{
    fn index_mut(&mut self, idx: I) -> &mut BasicStr<C, Traits> {
        let chars = &mut self.1[idx];

        unsafe {
            Traits::validate_subrange(chars).expect("Attempt to index str to produce invalid range")
        }

        unsafe { Self::from_chars_unchecked_mut(chars) }
    }
}

#[cfg(feature = "utf")]
impl Str {
    pub const fn from_str(x: &str) -> &Self {
        // SAFETY:
        // `Str` and `str` have the same invariant, thus `UtfCharTraits<u8>::validate_range` is trivially satisfied for the bytes of `str
        unsafe { Self::from_chars_unchecked(x.as_bytes()) }
    }

    pub fn from_str_mut(x: &mut str) -> &mut Self {
        // SAFETY:
        // `Str` and `str` have the same invariant, thus `UtfCharTraits<u8>::validate_range` is trivially satisfied for the bytes of `str`
        // Further, no mutation via `&mut Self` can break this invariant
        unsafe { Self::from_chars_unchecked_mut(x.as_bytes_mut()) }
    }

    pub const fn as_str(&self) -> &str {
        // SAFETY:
        // `Str` and `str` have the same invariant, thus self.as_chars() is valid utf8
        unsafe { core::str::from_utf8_unchecked(self.as_chars()) }
    }

    pub fn as_str_mut(&mut self) -> &mut str {
        // SAFETY:
        // `Str` and `str` have the same invariant, thus self.as_chars() is valid utf8
        // Further, no mutation via `&mut str` can break this invariant
        unsafe { core::str::from_utf8_unchecked_mut(self.as_chars_mut()) }
    }
}

#[cfg(feature = "utf")]
impl U32Str {
    pub const fn new(chars: &[char]) -> &Self {
        // SAFETY:
        // Validity of `[char]` for `UtfCharTraits<char>` is trivial
        unsafe { Self::from_chars_unchecked(chars) }
    }
}

#[cfg(feature = "utf")]
pub type UtfStr<CharT> = BasicStr<CharT, UtfCharTraits<CharT>>;
#[cfg(feature = "utf")]
pub type Str = BasicStr<u8, UtfCharTraits<u8>>;
#[cfg(feature = "utf")]
pub type U16Str = BasicStr<u16, UtfCharTraits<u16>>;
#[cfg(feature = "utf")]
pub type U32Str = BasicStr<char, UtfCharTraits<char>>;

#[cfg(feature = "utf")]
impl AsRef<U32Str> for [char] {
    fn as_ref(&self) -> &U32Str {
        // SAFETY:
        // All arrays of `char` are valid `U32Str`
        unsafe { U32Str::from_chars_unchecked(self) }
    }
}

#[cfg(feature = "utf")]
impl AsRef<Str> for str {
    fn as_ref(&self) -> &Str {
        // SAFETY:
        // self is a `str`, which guarantees UTF-8 bytes.
        unsafe { Str::from_chars_unchecked(self.as_bytes()) }
    }
}

#[cfg(feature = "utf")]
impl AsRef<str> for Str {
    fn as_ref(&self) -> &str {
        // SAFETY:
        // self is `Str` which guarantees UTF-8 bytes
        unsafe { core::str::from_utf8_unchecked(self.as_chars()) }
    }
}

#[cfg(all(feature = "std", feature = "utf"))]
impl AsRef<std::ffi::OsStr> for Str {
    fn as_ref(&self) -> &std::ffi::OsStr {
        std::ffi::OsStr::new(<Self as AsRef<str>>::as_ref(self))
    }
}

impl<CharT: Char, Traits: CharTraits<Char = CharT> + DebugStr> core::fmt::Debug
    for BasicStr<CharT, Traits>
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        unsafe { Traits::debug_range_unchecked(self.as_chars(), fmt) }
    }
}

impl<CharT: Char, Traits: CharTraits<Char = CharT> + DisplayStr> core::fmt::Display
    for BasicStr<CharT, Traits>
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        unsafe { Traits::display_range_unchecked(self.as_chars(), fmt) }
    }
}

impl<Traits: CharTraits> Ord for BasicStr<Traits::Char, Traits> {
    fn cmp(&self, other: &Self) -> Ordering {
        Traits::compare(self.as_chars(), other.as_chars()).unwrap()
    }
}

impl<CharT: Eq, Traits> PartialEq for BasicStr<CharT, Traits> {
    fn eq(&self, other: &Self) -> bool {
        self.as_chars() == other.as_chars()
    }
}

impl<Traits: CharTraits> PartialOrd for BasicStr<Traits::Char, Traits> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<CharT: Eq, Traits> Eq for BasicStr<CharT, Traits> {}

impl<CharT: Hash, Traits> Hash for BasicStr<CharT, Traits> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.1.hash(state);
    }
}

pub struct Chars<'a, CharT>(core::slice::Iter<'a, CharT>);

impl<CharT: Copy> Iterator for Chars<'_, CharT> {
    type Item = CharT;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<CharT: Copy> ExactSizeIterator for Chars<'_, CharT> {}

impl<CharT, Traits> BasicStr<CharT, Traits> {
    pub fn chars(&self) -> Chars<CharT> {
        Chars(self.1.iter())
    }
}

pub struct UnicodeIter<'a, CharT, Traits>(&'a [CharT], PhantomData<Traits>);

impl<Traits: IntoChars> Iterator for UnicodeIter<'_, Traits::Char, Traits> {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        if self.0.is_empty() {
            None
        } else {
            // SAFETY:
            // We've still got some buffer left, and we know it to be valid, since it came from `BasicStr`
            let (ch, rest) = unsafe { Traits::decode_buf_unchecked(self.0) };

            self.0 = rest;
            Some(ch)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.0.len() / Traits::max_encoding_len(),
            Some(self.0.len()),
        )
    }
}

impl<Traits: IntoChars> BasicStr<Traits::Char, Traits> {
    pub fn unicode_iter(&self) -> UnicodeIter<Traits::Char, Traits> {
        UnicodeIter(&self.1, PhantomData)
    }
}
