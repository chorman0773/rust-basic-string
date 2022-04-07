impl const CharTraits for RawCharTraits {
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
