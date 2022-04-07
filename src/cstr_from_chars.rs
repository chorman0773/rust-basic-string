impl<Traits: CharTraits> BasicCStr<Traits::Char, Traits> {
    /// Converts a slice of `CharT` to a [`BasicCStr`] if the following hold:
    /// * The last character of the slice is a zero terminator, according to [`CharTraits::is_zero_term`]
    /// * No character other than the last is a zero terminator, according to [`CharTraits::is_zero_term`]
    /// * The array (including the zero terminator) is valid according to [`CharTraits::validate_range`]
    ///
    /// Otherwise, returns `None`
    pub fn from_chars_with_null(chars: &[Traits::Char]) -> Option<&Self> {
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
            Ok(()) => Some(unsafe { Self::from_chars_with_null_unchecked(chars) }),
            Err(e) => {
                core::mem::forget(e);
                None
            }
        }
    }
}
