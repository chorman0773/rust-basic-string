impl const CharTraits for UtfCharTraits<u8> {
    type Char = u8;
    type Int = i32;
    type Error = UtfError;

    fn validate_range(buf: &[Self::Char]) -> Result<(), Self::Error> {
        let mut i = 0;
        while i < buf.len() {
            let c = buf[i];

            if c & 0x80 == 0x00 {
                continue;
            } else if c & 0xC0 == 0x80 {
                return Err(UtfError {
                    pos: i,
                    len: Some(1),
                });
            } else if c & 0xE0 == 0xC0 {
                i += 1;
                let c = if i < buf.len() {
                    buf[i]
                } else {
                    return Err(UtfError { pos: i, len: None });
                };

                if c & 0xC0 != 0x80 {
                    return Err(UtfError {
                        pos: i,
                        len: Some(1),
                    });
                }
            } else if c & 0xF0 == 0xE0 {
                i += 1;
                let c = if i < buf.len() {
                    buf[i]
                } else {
                    return Err(UtfError { pos: i, len: None });
                };

                if c & 0xC0 != 0x80 {
                    return Err(UtfError {
                        pos: i,
                        len: Some(1),
                    });
                }
                i += 1;
                let c = if i < buf.len() {
                    buf[i]
                } else {
                    return Err(UtfError { pos: i, len: None });
                };

                if c & 0xC0 != 0x80 {
                    return Err(UtfError {
                        pos: i,
                        len: Some(2),
                    });
                }
            } else if c & 0xF8 == 0xF0 {
                i += 1;
                let c = if i < buf.len() {
                    buf[i]
                } else {
                    return Err(UtfError { pos: i, len: None });
                };

                if c & 0xC0 != 0x80 {
                    return Err(UtfError {
                        pos: i,
                        len: Some(1),
                    });
                }
                i += 1;
                let c = if i < buf.len() {
                    buf[i]
                } else {
                    return Err(UtfError { pos: i, len: None });
                };

                if c & 0xC0 != 0x80 {
                    return Err(UtfError {
                        pos: i,
                        len: Some(2),
                    });
                }
                i += 1;
                let c = if i < buf.len() {
                    buf[i]
                } else {
                    return Err(UtfError { pos: i, len: None });
                };

                if c & 0xC0 != 0x80 {
                    return Err(UtfError {
                        pos: i,
                        len: Some(3),
                    });
                }
            }
        }

        Ok(())
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
            // for (i, &c) in buf.iter().rev().enumerate() {
            //     if c & 0xc0 == 0x80 {
            //         continue;
            //     } else if ((c & 0x80 == 0x00) && i == 0)
            //         || ((c & 0xe0 == 0xc0) && i == 1)
            //         || (i == 2)
            //     {
            //         return Ok(());
            //     }
            // }
            let mut i = 0;
            let mut l = buf.len() - 1;
            while i < 3 {
                if buf[l] & 0xc0 == 0x80 {
                    continue;
                } else if ((buf[l] & 0x80 == 0x00) && i == 0)
                    || ((buf[l] & 0xe0 == 0xc0) && i == 1)
                    || (i == 2)
                {
                    return Ok(());
                }
                if l == 0 {
                    break;
                }
                i += 1;
                l -= 1;
            }
            Err(UtfError {
                pos: buf.len(),
                len: None,
            })
        }
    }

    fn compare(r1: &[Self::Char], r2: &[Self::Char]) -> Result<Ordering, Self::Error> {
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

    fn eof() -> Self::Int {
        -1
    }

    fn is_zero_term(c: Self::Char) -> bool {
        c == 0
    }
}

impl const CharTraits for UtfCharTraits<u16> {
    type Char = u16;

    type Int = i32;

    type Error = UtfError;

    fn validate_range(buf: &[Self::Char]) -> Result<(), Self::Error> {
        let mut i = 0;

        while i < buf.len() {
            let c = buf[i];
            if (0xD800 <= c) && (c <= 0xDBFF) {
                i += 1;
                let c = if i < buf.len() {
                    buf[i]
                } else {
                    return Err(UtfError { pos: i, len: None });
                };
                if !(0xDC00 <= c) && (c <= 0xDFFF) {
                    return Err(UtfError {
                        pos: i,
                        len: Some(2),
                    });
                }
            } else if (0xDC00 <= c) && (c <= 0xDFFF) {
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

    fn eof() -> Self::Int {
        -1
    }

    fn is_zero_term(c: Self::Char) -> bool {
        c == 0
    }
}

impl const CharTraits for UtfCharTraits<char> {
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
        '\0'
    }

    fn eof() -> Self::Int {
        -1
    }

    fn is_zero_term(c: Self::Char) -> bool {
        c == '\0'
    }
}
