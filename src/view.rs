use core::ops::Deref;
use core::{marker::PhantomData, ptr::NonNull};

use crate::str::BasicStr;

#[repr(C)]
pub struct BasicStringView<'a, CharT, Traits> {
    begin: *const CharT,
    end: *const CharT,
    _phantom: PhantomData<&'a BasicStr<CharT, Traits>>,
}

impl<'a, CharT, Traits> BasicStringView<'a, CharT, Traits> {
    pub const fn new(str: &'a BasicStr<CharT, Traits>) -> Self {
        let slice = str.as_chars();

        #[repr(C)]
        union AsArray<'a, T> {
            reff: &'a T,
            arr: &'a [T; 1],
        }

        let begin = slice.as_ptr();
        let end = if let [.., reff] = slice {
            let [_, end @ ..] = unsafe { AsArray { reff }.arr };
            end.as_ptr()
        } else {
            slice.as_ptr()
        };

        unsafe { Self::from_raw_parts(begin, end) }
    }

    pub const fn empty() -> Self {
        let dangling = NonNull::dangling().as_ptr();
        unsafe { Self::from_raw_parts(dangling, dangling) }
    }

    pub const unsafe fn from_raw_parts(begin: *const CharT, end: *const CharT) -> Self {
        Self {
            begin,
            end,
            _phantom: PhantomData,
        }
    }
}

impl<CharT, Traits> Deref for BasicStringView<'_, CharT, Traits> {
    type Target = BasicStr<CharT, Traits>;

    fn deref(&self) -> &BasicStr<CharT, Traits> {
        let slice = unsafe {
            core::slice::from_raw_parts(self.begin, self.end.offset_from(self.begin) as usize)
        };
        unsafe { BasicStr::from_chars_unchecked(slice) }
    }
}
