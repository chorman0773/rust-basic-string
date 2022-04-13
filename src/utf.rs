use core::{cmp::Ordering, convert::Infallible, marker::PhantomData, str::Utf8Error};

use crate::traits::{
    Char, CharTraits, DebugStr, DecodeRev, DisplayStr, IntoChars, ValidationError,
};

use self::private::UtfIntoChars;

pub struct UtfCharTraits<T>(PhantomData<T>);

mod private {
    use crate::traits::CharTraits;

    use super::UtfCharTraits;

    pub trait UtfIntoChars: CharTraits {
        fn next_code_point<I: Iterator<Item = Self::Char> + ?Sized>(iter: &mut I) -> Option<char>;
    }

    impl UtfIntoChars for UtfCharTraits<u8> {
        fn next_code_point<I: Iterator<Item = u8> + ?Sized>(iter: &mut I) -> Option<char> {
            let v0 = iter.next()?;
            if (v0 & 0x80) == 0x00 {
                Some(v0 as char)
            } else if (v0 & 0xe0) == 0xc0 {
                let v1 = iter.next().expect("Expected valid UTF-8");
                let val = ((v0 & 0x1f) as u32) << 6 | ((v1 & 0x3f) as u32);
                Some(char::from_u32(val).expect("Expected valid UTF-8"))
            } else if (v0 & 0xf0) == 0xe0 {
                let v1 = iter.next().expect("Expected valid UTF-8");
                let v2 = iter.next().expect("Expected valid UTF-8");
                let val =
                    ((v0 & 0xf) as u32) << 12 | ((v1 & 0x3f) as u32) << 6 | ((v2 & 0x3f) as u32);
                Some(char::from_u32(val).expect("Expected valid UTF-8"))
            } else if (v0 & 0xf8) == 0xf0 {
                let v1 = iter.next().expect("Expected valid UTF-8");
                let v2 = iter.next().expect("Expected valid UTF-8");
                let v3 = iter.next().expect("Expected valid UTF-8");
                let val = ((v0 & 0x7) as u32) << 18
                    | ((v1 & 0x3f) as u32) << 12
                    | ((v2 & 0x3f) as u32) << 6
                    | ((v3 & 0x3f) as u32);
                Some(char::from_u32(val).expect("Expected valid UTF-8"))
            } else {
                panic!("Expected valid UTF-8")
            }
        }
    }

    impl UtfIntoChars for UtfCharTraits<u16> {
        fn next_code_point<I: Iterator<Item = u16> + ?Sized>(iter: &mut I) -> Option<char> {
            let v0 = iter.next()?;
            if (0xD800..=0xDBFF).contains(&v0) {
                let v1 = iter.next().expect("Expected valid UTF-16");
                if !(0xDC00..=0xDFFF).contains(&v0) {
                    panic!("Expected valid UTF-16")
                }
                let val = ((v0 - 0xD800) as u32) << 10 | ((v1 - 0xDC00) as u32);
                Some(char::from_u32(val).expect("Expected valid UTF-8"))
            } else {
                Some(char::from_u32(v0 as u32).expect("Expected valid UTF-8"))
            }
        }
    }

    impl UtfIntoChars for UtfCharTraits<char> {
        fn next_code_point<I: Iterator<Item = char> + ?Sized>(iter: &mut I) -> Option<char> {
            iter.next()
        }
    }
}

pub struct Chars<I>(I);

impl<I: Iterator> Iterator for Chars<I>
where
    UtfCharTraits<I::Item>: UtfIntoChars<Char = I::Item>,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        UtfCharTraits::<I::Item>::next_code_point(&mut self.0)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UtfError {
    pos: usize,
    len: Option<usize>,
}

impl From<Utf8Error> for UtfError {
    fn from(x: Utf8Error) -> Self {
        Self {
            pos: x.valid_up_to(),
            len: x.error_len(),
        }
    }
}

impl ValidationError for UtfError {
    fn first_error_pos(&self) -> usize {
        self.pos
    }

    fn first_error_len(&self) -> Option<usize> {
        self.len
    }
}

#[cfg(feature = "const-utf-char-traits")]
include!("utf_const_char_traits.rs");

#[cfg(not(feature = "const-utf-char-traits"))]
include!("utf_char_traits.rs");

unsafe impl IntoChars for UtfCharTraits<u8> {
    unsafe fn decode_buf_unchecked(buf: &[Self::Char]) -> (char, &[Self::Char]) {
        if buf[0] & 0x80 == 0x00 {
            (buf[0] as char, buf.get_unchecked(1..))
        } else if buf[0] & 0xe0 == 0xc0 {
            let val = ((buf[0] & 0x1f) as u32) << 6 | ((buf[1] & 0x3f) as u32);
            (char::from_u32_unchecked(val), buf.get(2..).unwrap_or(&[]))
        } else if buf[0] & 0xf0 == 0xe0 {
            let val = ((buf[0] & 0x1f) as u32) << 12
                | ((*buf.get_unchecked(1) & 0x3f) as u32) << 6
                | ((*buf.get_unchecked(2) & 0x3f) as u32);
            (char::from_u32_unchecked(val), buf.get(3..).unwrap_or(&[]))
        } else {
            let val = ((buf[0] & 0x7) as u32) << 18
                | ((*buf.get_unchecked(1) & 0x3f) as u32) << 12
                | ((*buf.get_unchecked(2) & 0x3f) as u32) << 6
                | ((*buf.get_unchecked(3) & 0x3f) as u32);
            (char::from_u32_unchecked(val), buf.get(4..).unwrap_or(&[]))
        }
    }

    fn decode_buf(buf: &[Self::Char]) -> Option<(char, &[Self::Char])> {
        let c0 = *buf.get(0)?;
        if c0 & 0x80 == 0x00 {
            Some((c0 as char, buf.get(1..).unwrap_or(&[])))
        } else if c0 & 0xe0 == 0xc0 {
            let c1 = *buf.get(1)?;
            let val = ((c0 & 0x1f) as u32) << 6 | ((c1 & 0x3f) as u32);
            Some((char::from_u32(val)?, buf.get(2..).unwrap_or(&[])))
        } else if c0 & 0xf0 == 0xe0 {
            let c1 = *buf.get(1)?;
            let c2 = *buf.get(2)?;
            let val = ((c0 & 0xf) as u32) << 12 | ((c1 & 0x3f) as u32) << 6 | ((c2 & 0x3f) as u32);
            Some((char::from_u32(val)?, buf.get(3..).unwrap_or(&[])))
        } else if c0 & 0xf8 == 0xf0 {
            let c1 = *buf.get(1)?;
            let c2 = *buf.get(2)?;
            let c3 = *buf.get(3)?;
            let val = ((c0 & 0x7) as u32) << 18
                | ((c1 & 0x3f) as u32) << 12
                | ((c2 & 0x3f) as u32) << 6
                | ((c3 & 0x3f) as u32);
            Some((char::from_u32(val)?, buf.get(4..).unwrap_or(&[])))
        } else {
            None
        }
    }

    fn max_encoding_len() -> usize {
        4
    }

    fn encode(c: char, buf: &mut [Self::Char]) -> &mut [Self::Char] {
        // SAFETY:
        // the intermediate &mut str is never used, and only exists because that's what [`char::encode_utf8`] returns apparently
        unsafe { c.encode_utf8(buf).as_bytes_mut() }
    }

    fn encoding_len(c: char) -> usize {
        c.len_utf8()
    }
}

unsafe impl DecodeRev for UtfCharTraits<u8> {
    unsafe fn decode_back_unchecked(mut buf: &[Self::Char]) -> (char, &[Self::Char]) {
        let mut val = 0;
        for i in 0.. {
            let (&b, rest) = buf.split_last().unwrap_unchecked();
            buf = rest;
            if b & 0xC0 != 0x80 {
                val |= (b as u32 & ((0x100 >> i) - 1)) << (6 * i);
                break;
            } else {
                val |= (b as u32 & 0x3f) << (6 * i);
            }
        }

        (char::from_u32_unchecked(val), buf)
    }

    fn decode_back(mut buf: &[Self::Char]) -> Option<(char, &[Self::Char])> {
        let mut val = 0;
        for i in 0.. {
            if i == 4 {
                return None;
            }
            let (&b, rest) = buf.split_last()?;
            buf = rest;
            if b & 0xC0 != 0x80 {
                if (i == 0 && b.leading_ones() != 1) || (b.leading_ones() != (i + 1)) {
                    return None;
                }
                val |= (b as u32 & ((0x100 >> i) - 1)) << (6 * i);
                break;
            } else {
                val |= (b as u32 & 0x3f) << (6 * i);
            }
        }

        Some((char::from_u32(val)?, buf))
    }
}

unsafe impl IntoChars for UtfCharTraits<u16> {
    unsafe fn decode_buf_unchecked(buf: &[Self::Char]) -> (char, &[Self::Char]) {
        let v0 = *buf.get_unchecked(0);
        if (0xD800..=0xDBFF).contains(&v0) {
            let v1 = *buf.get_unchecked(1);
            let val = ((v0 - 0xD800) as u32) << 10 | ((v1 - 0xDC00) as u32);
            (char::from_u32_unchecked(val), buf.get(2..).unwrap_or(&[]))
        } else {
            (char::from_u32_unchecked(v0 as u32), buf.get_unchecked(1..))
        }
    }

    fn decode_buf(buf: &[Self::Char]) -> Option<(char, &[Self::Char])> {
        let v0 = *buf.get(0)?;
        if (0xD800..=0xDBFF).contains(&v0) {
            let v1 = *buf.get(1)?;
            if (0xDC00..=0xDFFF).contains(&v1) {
                return None;
            }
            let val = ((v0 - 0xD800) as u32) << 10 | ((v1 - 0xDC00) as u32);
            Some((char::from_u32(val)?, buf.get(2..).unwrap_or(&[])))
        } else {
            Some((char::from_u32(v0 as u32)?, buf.get(2..).unwrap_or(&[])))
        }
    }

    fn max_encoding_len() -> usize {
        2
    }

    fn encode(c: char, buf: &mut [Self::Char]) -> &mut [Self::Char] {
        c.encode_utf16(buf)
    }

    fn encoding_len(c: char) -> usize {
        if (c as u32) < 0xFFFF {
            1
        } else {
            2
        }
    }
}

unsafe impl DecodeRev for UtfCharTraits<u16> {
    unsafe fn decode_back_unchecked(buf: &[Self::Char]) -> (char, &[Self::Char]) {
        let (v1, rest) = buf.split_last().unwrap_unchecked();

        if (0xDC00..=0xDFFF).contains(v1) {
            let (v0, rest) = rest.split_last().unwrap_unchecked();
            let val = ((*v0 - 0xD800) as u32) << 10 | ((*v1 - 0xDC00) as u32);
            (char::from_u32_unchecked(val), rest)
        } else {
            (char::from_u32_unchecked(*v1 as u32), rest)
        }
    }

    fn decode_back(buf: &[Self::Char]) -> Option<(char, &[Self::Char])> {
        let (v1, rest) = buf.split_last()?;

        if (0xDC00..=0xDFFF).contains(v1) {
            let (v0, rest) = rest.split_last()?;
            if !(0xD800..=0xDBFF).contains(v0) {
                return None;
            }
            let val = ((*v0 - 0xD800) as u32) << 10 | ((*v1 - 0xDC00) as u32);
            Some((unsafe { char::from_u32_unchecked(val) }, rest))
        } else {
            Some((unsafe { char::from_u32_unchecked(*v1 as u32) }, rest))
        }
    }
}

unsafe impl IntoChars for UtfCharTraits<char> {
    unsafe fn decode_buf_unchecked(buf: &[Self::Char]) -> (char, &[Self::Char]) {
        (*buf.get_unchecked(0), buf.get(1..).unwrap_or(&[]))
    }

    fn decode_buf(buf: &[Self::Char]) -> Option<(char, &[Self::Char])> {
        Some((*buf.get(0)?, buf.get(1..).unwrap_or(&[])))
    }

    fn max_encoding_len() -> usize {
        1
    }

    fn encode(c: char, buf: &mut [Self::Char]) -> &mut [Self::Char] {
        buf[0] = c;
        &mut buf[0..1]
    }

    fn encoding_len(_: char) -> usize {
        1
    }
}

unsafe impl DecodeRev for UtfCharTraits<char> {
    #[inline(always)]
    unsafe fn decode_back_unchecked(buf: &[Self::Char]) -> (char, &[Self::Char]) {
        Self::decode_back(buf).unwrap_unchecked()
    }

    #[inline]
    fn decode_back(buf: &[Self::Char]) -> Option<(char, &[Self::Char])> {
        buf.split_last().map(|(c, rest)| (*c, rest))
    }
}

impl<T: Char> DebugStr for UtfCharTraits<T>
where
    Self: UtfIntoChars + CharTraits<Char = T>,
{
    fn debug_range(range: &[T], fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Self::validate_range(range).unwrap();
        for c in Chars(range.iter().copied()) {
            fmt.write_fmt(format_args!("{}", c.escape_debug()))?;
        }
        Ok(())
    }

    unsafe fn debug_range_unchecked(
        range: &[Self::Char],
        fmt: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        for c in Chars(range.iter().copied()) {
            fmt.write_fmt(format_args!("{}", c.escape_debug()))?;
        }
        Ok(())
    }
}

impl<T: Char> DisplayStr for UtfCharTraits<T>
where
    Self: UtfIntoChars + CharTraits<Char = T>,
{
    fn display_range(range: &[T], fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Self::validate_range(range).unwrap();
        for c in Chars(range.iter().copied()) {
            fmt.write_str(c.encode_utf8(&mut [0u8; 4]))?;
        }
        Ok(())
    }

    unsafe fn display_range_unchecked(
        range: &[Self::Char],
        fmt: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        for c in Chars(range.iter().copied()) {
            fmt.write_str(c.encode_utf8(&mut [0u8; 4]))?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! u8str {
    ($lit:literal) => {
        $crate::str::Str::from_str($lit)
    };
}
