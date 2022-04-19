impl<Traits: CharTraits> BasicStr<Traits::Char, Traits> {
    pub const fn from_chars(chars: &[Traits::Char]) -> Result<&Self, Traits::Error>
    where
        Traits: ~const CharTraits,
        Traits::Error: ~const core::marker::Destruct,
    {
        match Traits::validate_range(chars) {
            Ok(()) => Ok(unsafe { Self::from_chars_unchecked(chars) }),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod test {

    use super::BasicStr;
    use crate::traits::*;
    use core::cmp::Ordering;
    struct TestCharTraits;

    #[derive(Debug)]
    struct TestCharTraitsError(usize);

    impl ValidationError for TestCharTraitsError {
        fn first_error_pos(&self) -> usize {
            self.0
        }

        fn first_error_len(&self) -> Option<usize> {
            Some(1)
        }
    }

    impl const CharTraits for TestCharTraits {
        type Char = u8;
        type Int = i32;
        type Error = TestCharTraitsError;

        fn validate_range(buf: &[Self::Char]) -> Result<(), Self::Error> {
            let mut i = 0;
            while i < buf.len() {
                if buf[i] == 0xff {
                    return Err(TestCharTraitsError(i));
                }
                i += 1;
            }

            Ok(())
        }

        unsafe fn validate_subrange(buf: &[Self::Char]) -> Result<(), Self::Error> {
            match (buf.first(), buf.last()) {
                (Some(0xff), _) => Err(TestCharTraitsError(0)),
                (_, Some(0xff)) => Err(TestCharTraitsError(buf.len() - 1)),
                _ => Ok(()),
            }
        }

        fn compare(left: &[Self::Char], right: &[Self::Char]) -> Result<Ordering, Self::Error> {
            let mut i = 0;

            while i < left.len() && i < right.len() {
                if left[i] < right[i] {
                    return Ok(Ordering::Less);
                } else if left[i] == right[i] {
                } else if left[i] > right[i] {
                    return Ok(Ordering::Greater);
                }
            }

            if left.len() < right.len() {
                Ok(Ordering::Less)
            } else if left.len() == right.len() {
                Ok(Ordering::Equal)
            } else {
                Ok(Ordering::Greater)
            }
        }

        fn zero_term() -> u8 {
            0
        }

        fn is_zero_term(c: Self::Char) -> bool {
            c == 0
        }

        fn eof() -> i32 {
            -1
        }
    }

    #[test]
    fn test_from_chars_const() {
        const VALID: &BasicStr<u8, TestCharTraits> = {
            match BasicStr::from_chars(&[0x00, 0x01, 0x02]) {
                Ok(x) => x,
                Err(_) => unreachable!(),
            }
        };

        assert_eq!(VALID.as_chars(), &[0x00, 0x01, 0x02])
    }

    #[test]
    fn test_from_chars_err() {
        const INVALID: TestCharTraitsError = {
            match BasicStr::<u8, TestCharTraits>::from_chars(&[0x00, 0xff, 0x02]) {
                Ok(_) => unreachable!(),
                Err(e) => e,
            }
        };

        assert_eq!(INVALID.0, 1)
    }
}
