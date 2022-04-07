impl<C, Traits> const AsRef<[C]> for BasicStr<C, Traits> {
    fn as_ref(&self) -> &[C] {
        self.as_chars()
    }
}

#[cfg(not(feature = "const-trait-impl"))]
impl<'a, C, Traits> const Default for &'a BasicStr<C, Traits>
where
    BasicStr<C, Traits>: 'a,
{
    fn default() -> &'a BasicStr<C, Trait> {
        unsafe { BasicStr::from_chars_unchecked(&[]) }
    }
}
