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
