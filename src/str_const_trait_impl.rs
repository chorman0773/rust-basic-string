impl<C: Char, Traits: CharTraits> const AsRef<[C]> for BasicStr<C, Traits> {
    fn as_ref(&self) -> &[C] {
        self.as_chars()
    }
}
