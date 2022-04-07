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

    fn is_zero_term(c: Self::Char) -> bool {
        c == 0
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

    fn is_zero_term(c: Self::Char) -> bool {
        c == 0
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

    fn is_zero_term(c: Self::Char) -> bool {
        c == '\0'
    }
}
