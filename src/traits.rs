use core::{cmp::Ordering, convert::Infallible};

///
/// Represents a type that can be used as a char in a string
/// [`Char`] provides primitive operations that are valid on values of the type reguardless f encoding.
///
/// # Safety
/// [`Char::into_int`] shall return a value of type [`Char::Int`],
///     such that passing it into [`Char::from_int`], [`Char::from_int_lossy`], or [`Char::from_int_unchecked`] yields the same value, as observed by `Eq`.
///
/// The [`Eq`] impl shall be consistent with the [`Eq`] impl of [`Char::Int`], and the [`Ord`] impl of [`Char::Int`] shall be c
pub unsafe trait Char: Copy + Sized + Eq + PartialOrd {
    /// The integer type that represent each valid value of `Self` (and usually at least one other value, EOF)
    type Int: Copy + Sized + Ord;

    /// The Maximum value of `Self`, such that `MAX.into_int()` compares greater than or equal to the result of converting any other value of `Self` to `Self::Int`
    const MAX: Self;
    /// The Minimum value of `Self`, such that `MIN.into_int()` compares less than or equal to the result of converting any other value of `Self` to `Self::Int`
    const MIN: Self;

    /// Converts self into `Self::Int`
    fn into_int(self) -> Self::Int;
    /// Attempts to convert a `Self::Int` into a `Self`.
    /// if `val` is out of range of `Self`, returns [`None`]
    fn from_int(val: Self::Int) -> Option<Self>;
    /// Convert a `Self::Int` into a `Self`, truncating it in an implementation-specific manner if it is out of range
    fn from_int_lossy(val: Self::Int) -> Self;
    /// Convert a `Self::Int` into a `Self` without checking the bounds of `val`
    ///
    /// # Safety
    /// val shall be in range for `Self`
    unsafe fn from_int_unchecked(val: Self::Int) -> Self;
}

unsafe impl Char for u8 {
    type Int = i32;

    const MAX: Self = u8::MAX;
    const MIN: Self = u8::MIN;

    fn into_int(self) -> Self::Int {
        self as i32
    }

    fn from_int(val: Self::Int) -> Option<Self> {
        val.try_into().ok()
    }

    fn from_int_lossy(val: Self::Int) -> Self {
        val as u8
    }

    unsafe fn from_int_unchecked(val: Self::Int) -> Self {
        val as u8 // We could make this actually cause UB when out of range, but eh...
    }
}

unsafe impl Char for u16 {
    type Int = i32;

    const MAX: Self = u16::MAX;
    const MIN: Self = u16::MIN;

    fn into_int(self) -> Self::Int {
        self as i32
    }

    fn from_int(val: Self::Int) -> Option<Self> {
        val.try_into().ok()
    }

    fn from_int_lossy(val: Self::Int) -> Self {
        val as u16
    }

    unsafe fn from_int_unchecked(val: Self::Int) -> Self {
        val as u16 // We could make this actually cause UB when out of range, but eh...
    }
}

unsafe impl Char for char {
    type Int = i32;

    const MAX: Self = char::MAX;
    const MIN: Self = '\0';

    fn into_int(self) -> Self::Int {
        self as i32
    }

    fn from_int(val: Self::Int) -> Option<Self> {
        char::from_u32(val as u32)
    }

    fn from_int_lossy(val: Self::Int) -> Self {
        char::from_u32(val as u32).unwrap_or('\0')
    }

    unsafe fn from_int_unchecked(val: Self::Int) -> Self {
        char::from_u32_unchecked(val as u32)
    }
}

unsafe impl Char for u32 {
    type Int = i32;

    const MAX: Self = u32::MAX;
    const MIN: Self = u32::MIN;

    fn into_int(self) -> Self::Int {
        self as i32
    }

    fn from_int(val: Self::Int) -> Option<Self> {
        Some(val as u32)
    }

    fn from_int_lossy(val: Self::Int) -> Self {
        val as u32
    }

    unsafe fn from_int_unchecked(val: Self::Int) -> Self {
        val as u32
    }
}

pub trait ValidationError: core::fmt::Debug {
    fn first_error_pos(&self) -> usize;
    fn first_error_len(&self) -> Option<usize>;
}

/// A trait for types that can be used in String Types and String Views/String Slices
pub trait CharTraits {
    /// The Character Type for these traits
    type Char: Char<Int = Self::Int>;
    /// The Integer type
    type Int: Copy + Sized + Ord;
    /// The Type that Is returned when validation of a range fails
    type Error: ValidationError;

    /// Performs a validiation of the input range
    ///
    /// # Errors
    ///
    /// Returns an error if the range is not valid text
    fn validate_range(buf: &[Self::Char]) -> Result<(), Self::Error>;

    /// Performs validation of a subrange of known valid range.
    /// The implementation is allowed to assume that the buffer belongs to a previously validated buffer
    /// (This allows, e.g. a Utf8 implementation to only check the first and last few characters).
    ///
    /// This function succeeds if [`CharTraits::validate_range`] would succeed on the same buffer
    ///
    /// # Safety
    /// No undefined behaviour is observed from the implementation of this function.
    /// However, in validating the range, the implementation is permitted to return an incorrect result
    /// if `buf`.
    ///
    /// The function is `unsafe` to ensure that the caller is capable of handling spurious successes.
    ///
    /// # Errors
    ///
    /// Returns an Error if the subrange isn't valid.
    ///
    /// The function may spuriously succeed if the range was not a subrange of a previously validated range.
    unsafe fn validate_subrange(buf: &[Self::Char]) -> Result<(), Self::Error>;

    /// Compares two strings lexicographically
    /// This does not need to be consistent with the [`Ord`] impl of `Char`.
    ///
    /// # Errors
    ///
    /// Returns an error if the subrange isn't valid
    ///
    fn compare(r1: &[Self::Char], r2: &[Self::Char]) -> Result<Ordering, Self::Error>;

    /// The character to append to/search for in null terminated strings
    fn zero_term() -> Self::Char;

    /// The "End of File" Sentinel.
    /// This is typically not a possible value of `Char`
    fn eof() -> Self::Int;
}

/// Methods for [`CharTraits`] implementations that can be encoded/decoded losslessly through the Rust [`char`] type.
///
/// # Safety
/// The behaviour of the `encode` and `max_encoding_len` methods must be as-defined.
pub unsafe trait IntoChars: CharTraits {
    /// Decodes the given buf into a char, and returns it and the remainder of the buffer.
    ///
    /// # Safety
    /// `buf` shall be valid according to [`CharTraits::validate_range`]
    unsafe fn decode_buf_unchecked(buf: &[Self::Char]) -> (char, &[Self::Char]);

    /// Decodes the given buf into a char if possible, and returns it and the remainder of the buffer.
    ///
    /// May return `None` or an implementation-defined `char` if `buf` is invalid according to [`CharTraits::validate_range`]
    fn decode_buf(buf: &[Self::Char]) -> Option<(char, &[Self::Char])>;

    /// Determines the maximum encoding length.
    ///
    /// The Round trip of `Self::decode_unchecked(Self::encode(c,b)).0` shall have defined behaviour
    ///  and yield `c` exactly, if `b` is a `[Self::Char]` that has a `len` at least this value
    fn max_encoding_len() -> usize;

    /// Encodes `c` into the beginning of `buf` and returns the slice of `buf` that entrely contains the encoded characters
    ///
    /// The returned buffer shall be valid according to [`CharTraits::validate_range`]
    ///
    /// # Panics
    /// This function panics if `buf` is not sufficiently sized to encode `c`.
    /// The necessary size is implementation-defined, but is at most [`IntoChars::max_encoding_len`]
    fn encode(c: char, buf: &mut [Self::Char]) -> &mut [Self::Char];
}

pub trait DebugStr: CharTraits {
    /// Writes a [`CharTraits`]-specific Debug representation of the given character range
    ///
    /// Implementations should not wrap the output in double quotes, but should escape non-printing characters, non-unicode characters (if applicable), and control characters
    ///  in an implementation-specific manner
    ///
    /// If the given range is not valid, according to [`CharTraits::validate_range`], the result is implementation-defined, or the function may panic.
    ///
    /// # Panics
    /// The function may panic if validation fails (according to [`CharTraits::validate_range`])
    fn debug_range(range: &[Self::Char], fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;

    /// Writes a Trait-specific debug representation of the given character range
    ///
    /// The output should be consistent with the one produced by the safe [`DebugStr::debug_range`] function.
    ///
    /// If the given range is not valid, according to [`CharTraits::validate_range`], the result is implementation-defined, or the behaviour is undefined.
    ///
    /// # Safety
    ///
    /// This function may have undefined behaviour if validation would fail (according to [`CharTraits::validate_range`])
    unsafe fn debug_range_unchecked(
        range: &[Self::Char],
        fmt: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result;
}

pub trait DisplayStr: CharTraits {
    /// Writes a [`CharTraits`]-specific Debug representation of the given character range
    ///
    /// Implementations should not wrap the output, or escape non-printing or control characters, but may escape or translate non-unicode characters in an implementation-specific manner.
    ///
    /// If the given range is not valid, according to [`CharTraits::validate_range`], the result is implementation-defined, or the function may panic.
    ///
    /// # Panics
    /// The function may panic if validation fails (according to [`CharTraits::validate_range`]).
    fn display_range(range: &[Self::Char], fmt: &mut core::fmt::Formatter<'_>)
        -> core::fmt::Result;

    /// Writes a Trait-specific debug representation of the given character range
    ///
    /// The output should be consistent with the one produced by the safe [`DebugStr::debug_range`] function.
    ///
    /// If the given range is not valid, according to [`CharTraits::validate_range`], the result is implementation-defined, or the behaviour is undefined.
    ///
    /// # Safety
    ///
    /// This function may have undefined behaviour if validation would fail (according to [`CharTraits::validate_range`])
    unsafe fn display_range_unchecked(
        range: &[Self::Char],
        fmt: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result;
}

impl ValidationError for Infallible {
    fn first_error_pos(&self) -> usize {
        match *self {}
    }

    fn first_error_len(&self) -> Option<usize> {
        match *self {}
    }
}
