use crate::{
    str::BasicStr,
    traits::{CharTraits, DecodeRev, IntoChars},
};

///
/// Trait for types that can search for matches within a string.
///
/// # Safety
/// The requirements of [`Pattern::first_match`] and [`Pattern::first_match_unchecked`] shall be upheld.
/// Additionally, if a slice of characters that is valid according to [`CharTraits::validate_range`], is passed to either `first_match` or `first_match_unchecked`,
///  the resulting slice, if any, shall be valid/
pub unsafe trait Pattern<CharT, CharTraits> {
    /// Finds the first match of `self` in `slice` and returns a slice over that pattern, or None if no such match exists
    /// This function may (but is not required to) return `None` if `slice` is not valid, according to [`CharTraits::validate_range`]
    ///
    /// This function shall be implemented such that if the return value is some, then it is a subslice of `slice`.
    fn first_match<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>;

    /// Finds the first match of `self` in `slice`, or None if no such match exists.
    ///  
    /// This function shall be implemented such that if the return value is some, then it is a subslice of `slice`.
    ///
    /// # Safety
    /// The behaviour may be undefined or an implementation-defined result may result if `slice` is not valid, according to [`CharTraits::validate_range`]
    unsafe fn first_match_unchecked<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]> {
        self.first_match(slice)
    }
}

///
/// Trait for types that can search for matches within a string.
///
/// # Safety
/// The requirements of [`RevPattern::last_match`] and [`RevPattern::last_match_unchecked`] shall be upheld.
/// Additionally, if a slice of characters that is valid according to [`CharTraits::validate_range`], is passed to either `last_match` or `last_match_unchecked`,
///  the resulting slice, if any, shall be valid/
pub unsafe trait RevPattern<CharT, CharTraits> {
    /// Finds the last match of `self` in `slice` and returns a slice over that pattern, or None if no such match exists
    /// This function may (but is not required to) return `None` if `slice` is not valid, according to [`CharTraits::validate_range`]
    ///
    /// This function shall be implemented such that if the return value is some, then it is a subslice of `slice`.
    fn last_match<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>;

    /// Finds the last match of `self` in `slice`  and returns a slice over that pattern, or None if no such match exists.
    ///  
    /// This function shall be implemented such that if the return value is some, then it is a subslice of `slice`.
    ///
    /// # Safety
    /// The behaviour may be undefined or an implementation-defined result may result if `slice` is not valid, according to [`CharTraits::validate_range`]
    unsafe fn last_match_unchecked<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]> {
        self.last_match(slice)
    }
}

/// A trait for pattern types that can be matched both forwards and in reverse
///
pub unsafe trait BidirectionalPattern<CharT, CharTraits>:
    Pattern<CharT, CharTraits> + RevPattern<CharT, CharTraits>
{
}

unsafe impl<Traits: CharTraits> Pattern<Traits::Char, Traits> for BasicStr<Traits::Char, Traits> {
    fn first_match<'a>(&self, slice: &'a [Traits::Char]) -> Option<&'a [Traits::Char]> {
        if slice.len() < self.len() {
            None
        } else {
            for i in 0..(slice.len() - self.len()) {
                let sliced = &slice[i..][..self.len()];
                if self.as_chars() == sliced {
                    return Some(sliced);
                }
            }
            None
        }
    }
}

unsafe impl<Traits: CharTraits> RevPattern<Traits::Char, Traits>
    for BasicStr<Traits::Char, Traits>
{
    fn last_match<'a>(&self, slice: &'a [Traits::Char]) -> Option<&'a [Traits::Char]> {
        if slice.len() < self.len() {
            None
        } else {
            for i in (0..(slice.len() - self.len())).rev() {
                let sliced = &slice[..i][..self.len()];
                if self.as_chars() == sliced {
                    return Some(sliced);
                }
            }
            None
        }
    }
}

unsafe impl<Traits: CharTraits> BidirectionalPattern<Traits::Char, Traits>
    for BasicStr<Traits::Char, Traits>
{
}

unsafe impl<Traits: CharTraits + IntoChars> Pattern<Traits::Char, Traits> for char {
    unsafe fn first_match_unchecked<'a>(
        &self,
        mut slice: &'a [Traits::Char],
    ) -> Option<&'a [Traits::Char]> {
        while !slice.is_empty() {
            let (c, rest) = Traits::decode_buf_unchecked(slice);
            if c == *self {
                return Some(&slice[..Traits::encoding_len(c)]);
            }
            slice = rest;
        }
        None
    }

    fn first_match<'a>(&self, mut slice: &'a [Traits::Char]) -> Option<&'a [Traits::Char]> {
        while let Some((c, rest)) = Traits::decode_buf(slice) {
            if c == *self {
                return Some(&slice[..Traits::encoding_len(c)]);
            }
            slice = rest;
        }
        None
    }
}

unsafe impl<Traits: CharTraits + DecodeRev> RevPattern<Traits::Char, Traits> for char {
    unsafe fn last_match_unchecked<'a>(
        &self,
        mut slice: &'a [Traits::Char],
    ) -> Option<&'a [Traits::Char]> {
        while !slice.is_empty() {
            let (c, rest) = Traits::decode_back_unchecked(slice);
            if c == *self {
                let pos = slice.as_ptr().offset_from(rest.as_ptr()) as usize;
                return Some(&slice[pos..]);
            }
            slice = rest;
        }
        None
    }

    fn last_match<'a>(&self, mut slice: &'a [Traits::Char]) -> Option<&'a [Traits::Char]> {
        while let Some((c, rest)) = Traits::decode_back(slice) {
            if c == *self {
                let pos = unsafe { slice.as_ptr().offset_from(rest.as_ptr()) } as usize;
                return Some(&slice[pos..]);
            }
            slice = rest;
        }
        None
    }
}

unsafe impl<Traits: CharTraits + DecodeRev> BidirectionalPattern<Traits::Char, Traits> for char {}

unsafe impl<Traits: CharTraits + IntoChars, F: Fn(char) -> bool> Pattern<Traits::Char, Traits>
    for F
{
    fn first_match<'a>(&self, mut slice: &'a [Traits::Char]) -> Option<&'a [Traits::Char]> {
        while let Some((c, rest)) = Traits::decode_buf(slice) {
            if (*self)(c) {
                return Some(&slice[..Traits::encoding_len(c)]);
            }
            slice = rest;
        }
        None
    }

    unsafe fn first_match_unchecked<'a>(
        &self,
        mut slice: &'a [Traits::Char],
    ) -> Option<&'a [Traits::Char]> {
        while !slice.is_empty() {
            let (c, rest) = Traits::decode_buf_unchecked(slice);
            if (*self)(c) {
                return Some(&slice[..Traits::encoding_len(c)]);
            }
            slice = rest;
        }
        None
    }
}

unsafe impl<Traits: CharTraits + DecodeRev, F: Fn(char) -> bool> RevPattern<Traits::Char, Traits>
    for F
{
    unsafe fn last_match_unchecked<'a>(
        &self,
        mut slice: &'a [Traits::Char],
    ) -> Option<&'a [Traits::Char]> {
        while !slice.is_empty() {
            let (c, rest) = Traits::decode_back_unchecked(slice);
            if (*self)(c) {
                let pos = slice.as_ptr().offset_from(rest.as_ptr()) as usize;
                return Some(&slice[pos..]);
            }
            slice = rest;
        }
        None
    }

    fn last_match<'a>(&self, mut slice: &'a [Traits::Char]) -> Option<&'a [Traits::Char]> {
        while let Some((c, rest)) = Traits::decode_back(slice) {
            if (*self)(c) {
                let pos = unsafe { slice.as_ptr().offset_from(rest.as_ptr()) } as usize;
                return Some(&slice[pos..]);
            }
            slice = rest;
        }
        None
    }
}

unsafe impl<Traits: CharTraits + DecodeRev, F: Fn(char) -> bool>
    BidirectionalPattern<Traits::Char, Traits> for F
{
}

unsafe impl<Traits: CharTraits + IntoChars> Pattern<Traits::Char, Traits> for [char] {
    fn first_match<'a>(&self, mut slice: &'a [Traits::Char]) -> Option<&'a [Traits::Char]> {
        while let Some((c, rest)) = Traits::decode_buf(slice) {
            if self.contains(&c) {
                return Some(&slice[..Traits::encoding_len(c)]);
            }
            slice = rest;
        }
        None
    }

    unsafe fn first_match_unchecked<'a>(
        &self,
        mut slice: &'a [Traits::Char],
    ) -> Option<&'a [Traits::Char]> {
        while !slice.is_empty() {
            let (c, rest) = Traits::decode_buf_unchecked(slice);
            if self.contains(&c) {
                return Some(&slice[..Traits::encoding_len(c)]);
            }
            slice = rest;
        }
        None
    }
}

unsafe impl<Traits: CharTraits + DecodeRev> RevPattern<Traits::Char, Traits> for [char] {
    unsafe fn last_match_unchecked<'a>(
        &self,
        mut slice: &'a [Traits::Char],
    ) -> Option<&'a [Traits::Char]> {
        while !slice.is_empty() {
            let (c, rest) = Traits::decode_back_unchecked(slice);
            if self.contains(&c) {
                let pos = slice.as_ptr().offset_from(rest.as_ptr()) as usize;
                return Some(&slice[pos..]);
            }
            slice = rest;
        }
        None
    }

    fn last_match<'a>(&self, mut slice: &'a [Traits::Char]) -> Option<&'a [Traits::Char]> {
        while let Some((c, rest)) = Traits::decode_back(slice) {
            if self.contains(&c) {
                let pos = unsafe { slice.as_ptr().offset_from(rest.as_ptr()) } as usize;
                return Some(&slice[pos..]);
            }
            slice = rest;
        }
        None
    }
}

unsafe impl<Traits: CharTraits + DecodeRev> BidirectionalPattern<Traits::Char, Traits>
    for [char]
{
}

unsafe impl<CharT, Traits, const N: usize> Pattern<CharT, Traits> for [char; N]
where
    [char]: Pattern<CharT, Traits>,
{
    fn first_match<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]> {
        <[char]>::first_match(self, slice)
    }

    unsafe fn first_match_unchecked<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]> {
        <[char]>::first_match_unchecked(self, slice)
    }
}

unsafe impl<CharT, Traits, const N: usize> RevPattern<CharT, Traits> for [char; N]
where
    [char]: RevPattern<CharT, Traits>,
{
    fn last_match<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]> {
        <[char]>::last_match(self, slice)
    }

    unsafe fn last_match_unchecked<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]> {
        <[char]>::last_match_unchecked(self, slice)
    }
}

unsafe impl<CharT, Traits, const N: usize> BidirectionalPattern<CharT, Traits> for [char; N] where
    [char]: BidirectionalPattern<CharT, Traits>
{
}

macro_rules! impl_ref_ref_mut{
    ($($ty:ty),*) => {
        $(
            unsafe impl<CharT, Traits> Pattern<CharT, Traits> for &$ty where $ty: Pattern<CharT, Traits>{
                fn first_match<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>{
                    <$ty as Pattern<CharT, Traits>>::first_match(self,slice)
                }

                unsafe fn first_match_unchecked<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>{
                    <$ty as Pattern<CharT, Traits>>::first_match_unchecked(self,slice)
                }
            }

            unsafe impl<CharT, Traits> Pattern<CharT, Traits> for &mut $ty where $ty: Pattern<CharT, Traits>{
                fn first_match<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>{
                    <$ty as Pattern<CharT, Traits>>::first_match(self,slice)
                }

                unsafe fn first_match_unchecked<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>{
                    <$ty as Pattern<CharT, Traits>>::first_match_unchecked(self,slice)
                }
            }

            unsafe impl<CharT, Traits> RevPattern<CharT, Traits> for &$ty where $ty: RevPattern<CharT, Traits>{
                fn last_match<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>{
                    <$ty as RevPattern<CharT, Traits>>::last_match(self,slice)
                }

                unsafe fn last_match_unchecked<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>{
                    <$ty as RevPattern<CharT, Traits>>::last_match_unchecked(self,slice)
                }
            }

            unsafe impl<CharT, Traits> RevPattern<CharT, Traits> for &mut $ty where $ty: RevPattern<CharT, Traits>{
                fn last_match<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>{
                    <$ty as RevPattern<CharT, Traits>>::last_match(self,slice)
                }

                unsafe fn last_match_unchecked<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>{
                    <$ty as RevPattern<CharT, Traits>>::last_match_unchecked(self,slice)
                }
            }

            unsafe impl<CharT, Traits> BidirectionalPattern<CharT, Traits> for &$ty where $ty: BidirectionalPattern<CharT, Traits>{}

            unsafe impl<CharT, Traits> BidirectionalPattern<CharT, Traits> for &mut $ty where $ty: BidirectionalPattern<CharT, Traits>{}

        )*
    }
}

impl_ref_ref_mut!(char, [char], BasicStr<CharT,Traits>);

macro_rules! impl_ref_ref_mut_array{
    ($([$ty:ty ; _]),*) => {
        $(
            unsafe impl<CharT, Traits,const N: usize> Pattern<CharT, Traits> for &[$ty;N] where [$ty;N]: Pattern<CharT, Traits>{
                fn first_match<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>{
                    <[$ty;N] as Pattern<CharT, Traits>>::first_match(self,slice)
                }

                unsafe fn first_match_unchecked<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>{
                    <[$ty;N] as Pattern<CharT, Traits>>::first_match_unchecked(self,slice)
                }
            }

            unsafe impl<CharT, Traits,const N: usize> Pattern<CharT, Traits> for &mut [$ty;N] where [$ty;N]: Pattern<CharT, Traits>{
                fn first_match<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>{
                    <[$ty;N] as Pattern<CharT, Traits>>::first_match(self,slice)
                }

                unsafe fn first_match_unchecked<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>{
                    <[$ty;N] as Pattern<CharT, Traits>>::first_match_unchecked(self,slice)
                }
            }

            unsafe impl<CharT, Traits, const N: usize> RevPattern<CharT, Traits> for &[$ty;N] where [$ty;N]: RevPattern<CharT, Traits>{
                fn last_match<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>{
                    <[$ty;N] as RevPattern<CharT, Traits>>::last_match(self,slice)
                }

                unsafe fn last_match_unchecked<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>{
                    <[$ty;N] as RevPattern<CharT, Traits>>::last_match_unchecked(self,slice)
                }
            }

            unsafe impl<CharT, Traits, const N: usize> RevPattern<CharT, Traits> for &mut [$ty;N] where [$ty;N]: RevPattern<CharT, Traits>{
                fn last_match<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>{
                    <[$ty;N] as RevPattern<CharT, Traits>>::last_match(self,slice)
                }

                unsafe fn last_match_unchecked<'a>(&self, slice: &'a [CharT]) -> Option<&'a [CharT]>{
                    <[$ty;N] as RevPattern<CharT, Traits>>::last_match_unchecked(self,slice)
                }
            }

            unsafe impl<CharT, Traits,const N: usize> BidirectionalPattern<CharT, Traits> for &[$ty;N] where [$ty;N]: BidirectionalPattern<CharT, Traits>{}

            unsafe impl<CharT, Traits,const N: usize> BidirectionalPattern<CharT, Traits> for &mut [$ty;N] where [$ty;N]: BidirectionalPattern<CharT, Traits>{}

        )*
    }
}

impl_ref_ref_mut_array!([char; _]);
