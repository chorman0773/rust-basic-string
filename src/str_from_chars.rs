impl<Traits: CharTraits> BasicStr<Traits::Char, Traits> {
    pub fn from_chars(chars: &[Traits::Char]) -> Result<&Self, Traits::Error> {
        match Traits::validate_range(chars) {
            Ok(()) => Ok(unsafe { Self::from_chars_unchecked(chars) }),
            Err(e) => Err(e),
        }
    }
}
