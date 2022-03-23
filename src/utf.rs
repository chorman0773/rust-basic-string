use core::{cmp::Ordering, convert::Infallible, marker::PhantomData, str::Utf8Error};

use crate::traits::{Char, CharTraits, DebugStr, DisplayStr, IntoChars, ValidationError};

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

impl CharTraits for UtfCharTraits<u8> {
    type Char = u8;
    type Int = i32;
    type Error = UtfError;

    fn validate_range(buf: &[Self::Char]) -> Result<(), Self::Error> {
        core::str::from_utf8(buf).map(drop).map_err(From::from)
    }

    unsafe fn validate_subrange(buf: &[Self::Char]) -> Result<(), Self::Error> {
        if buf.is_empty() {
            Ok(())
        } else if buf[0] & 0xc0 == 0x80 {
            Err(UtfError {
                pos: 0,
                len: Some(1),
            })
        } else if buf.len() == 1 {
            Ok(())
        } else {
            for (i, &c) in buf.iter().rev().enumerate() {
                if c & 0xc0 == 0x80 {
                    continue;
                } else if ((c & 0x80 == 0x00) && i == 0)
                    || ((c & 0xe0 == 0xc0) && i == 1)
                    || (i == 2)
                {
                    return Ok(());
                }
            }
            Err(UtfError {
                pos: buf.len(),
                len: None,
            })
        }
    }

    fn compare(r1: &[Self::Char], r2: &[Self::Char]) -> Result<Ordering, Self::Error> {
        Ok(r1.cmp(r2))
    }

    fn zero_term() -> Self::Char {
        0
    }

    fn eof() -> Self::Int {
        -1
    }
}

impl CharTraits for UtfCharTraits<u16> {
    type Char = u16;

    type Int = i32;

    type Error = UtfError;

    fn validate_range(buf: &[Self::Char]) -> Result<(), Self::Error> {
        let mut iter = buf.iter().enumerate();

        while let Some((i, &c)) = iter.next() {
            if (0xD800..=0xDBFF).contains(&c) {
                let (_, &c) = iter.next().ok_or(UtfError { pos: i, len: None })?;
                if !(0xDC00..=0xDFFF).contains(&c) {
                    return Err(UtfError {
                        pos: i,
                        len: Some(2),
                    });
                }
            } else if (0xDC00..=0xDFFF).contains(&c) {
                return Err(UtfError {
                    pos: i,
                    len: Some(1),
                });
            }
        }

        Ok(())
    }

    unsafe fn validate_subrange(buf: &[Self::Char]) -> Result<(), Self::Error> {
        if let Some(0xDC00..=0xDFFF) = buf.first() {
            Err(UtfError {
                pos: 0,
                len: Some(1),
            })
        } else if let Some(0xD800..=0xDBFF) = buf.last() {
            Err(UtfError {
                pos: buf.len() - 1,
                len: None,
            })
        } else {
            Ok(())
        }
    }

    fn compare(r1: &[Self::Char], r2: &[Self::Char]) -> Result<Ordering, Self::Error> {
        Ok(r1.cmp(r2))
    }

    fn zero_term() -> Self::Char {
        0
    }

    fn eof() -> Self::Int {
        -1
    }
}

impl CharTraits for UtfCharTraits<char> {
    type Char = char;

    type Int = i32;

    type Error = Infallible;

    fn validate_range(_: &[Self::Char]) -> Result<(), Self::Error> {
        Ok(())
    }

    unsafe fn validate_subrange(_: &[Self::Char]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn compare(r1: &[Self::Char], r2: &[Self::Char]) -> Result<Ordering, Self::Error> {
        Ok(r1.cmp(r2))
    }

    fn zero_term() -> Self::Char {
        '\0'
    }

    fn eof() -> Self::Int {
        -1
    }
}

unsafe impl IntoChars for UtfCharTraits<u8> {
    unsafe fn decode_buf_unchecked(buf: &[Self::Char]) -> (char, &[Self::Char]) {
        if buf[0] & 0x80 == 0x00 {
            (buf[0] as char, buf.get_unchecked(1..))
        } else if buf[0] & 0xe0 == 0xc0 {
            let val = ((buf[0] & 0x1f) as u32) << 6 | ((buf[1] & 0x3f) as u32);
            (char::from_u32_unchecked(val), buf.get_unchecked(2..))
        } else if buf[0] & 0xf0 == 0xe0 {
            let val = ((buf[0] & 0x1f) as u32) << 12
                | ((*buf.get_unchecked(1) & 0x3f) as u32) << 6
                | ((*buf.get_unchecked(2) & 0x3f) as u32);
            (char::from_u32_unchecked(val), buf.get_unchecked(3..))
        } else {
            let val = ((buf[0] & 0x7) as u32) << 18
                | ((*buf.get_unchecked(1) & 0x3f) as u32) << 12
                | ((*buf.get_unchecked(2) & 0x3f) as u32) << 6
                | ((*buf.get_unchecked(3) & 0x3f) as u32);
            (char::from_u32_unchecked(val), buf.get_unchecked(4..))
        }
    }

    fn decode_buf(buf: &[Self::Char]) -> Option<(char, &[Self::Char])> {
        let c0 = *buf.get(0)?;
        if c0 & 0x80 == 0x00 {
            Some((c0 as char, &buf[1..]))
        } else if c0 & 0xe0 == 0xc0 {
            let c1 = *buf.get(1)?;
            let val = ((c0 & 0x1f) as u32) << 6 | ((c1 & 0x3f) as u32);
            Some((char::from_u32(val)?, &buf[2..]))
        } else if c0 & 0xf0 == 0xe0 {
            let c1 = *buf.get(1)?;
            let c2 = *buf.get(2)?;
            let val = ((c0 & 0xf) as u32) << 12 | ((c1 & 0x3f) as u32) << 6 | ((c2 & 0x3f) as u32);
            Some((char::from_u32(val)?, &buf[2..]))
        } else if c0 & 0xf8 == 0xf0 {
            let c1 = *buf.get(1)?;
            let c2 = *buf.get(2)?;
            let c3 = *buf.get(3)?;
            let val = ((c0 & 0x7) as u32) << 18
                | ((c1 & 0x3f) as u32) << 12
                | ((c2 & 0x3f) as u32) << 6
                | ((c3 & 0x3f) as u32);
            Some((char::from_u32(val)?, &buf[2..]))
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
}

unsafe impl IntoChars for UtfCharTraits<u16> {
    unsafe fn decode_buf_unchecked(buf: &[Self::Char]) -> (char, &[Self::Char]) {
        let v0 = *buf.get_unchecked(0);
        if (0xD800..=0xDBFF).contains(&v0) {
            let v1 = *buf.get_unchecked(1);
            let val = ((v0 - 0xD800) as u32) << 10 | ((v1 - 0xDC00) as u32);
            (char::from_u32_unchecked(val), buf.get_unchecked(2..))
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
            Some((char::from_u32(val)?, &buf[2..]))
        } else {
            Some((char::from_u32(v0 as u32)?, &buf[1..]))
        }
    }

    fn max_encoding_len() -> usize {
        2
    }

    fn encode(c: char, buf: &mut [Self::Char]) -> &mut [Self::Char] {
        c.encode_utf16(buf)
    }
}

unsafe impl IntoChars for UtfCharTraits<char> {
    unsafe fn decode_buf_unchecked(buf: &[Self::Char]) -> (char, &[Self::Char]) {
        (*buf.get_unchecked(0), buf.get_unchecked(1..))
    }

    fn decode_buf(buf: &[Self::Char]) -> Option<(char, &[Self::Char])> {
        Some((*buf.get(0)?, buf.get(1..)?))
    }

    fn max_encoding_len() -> usize {
        1
    }

    fn encode(c: char, buf: &mut [Self::Char]) -> &mut [Self::Char] {
        buf[0] = c;
        &mut buf[0..1]
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
