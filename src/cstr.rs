use std::marker::PhantomData;

pub struct BasicCStr<CharT, Traits>(PhantomData<Traits>, [CharT]);
