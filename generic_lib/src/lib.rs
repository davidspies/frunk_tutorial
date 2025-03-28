pub mod reexports {
    pub use frunk_utils;
}

use frunk_utils::Func;

pub trait AllFieldsPresent {
    fn all_fields_present(&self) -> bool;
}

#[macro_export]
macro_rules! derive_all_fields_present {
    ($t:ty) => {
        impl $crate::AllFieldsPresent for $t {
            fn all_fields_present(&self) -> bool {
                use frunk::ToRef;
                use $crate::reexports::frunk_utils::WithGeneric;

                let mut all_fields_present = true;
                self.to_ref()
                    .for_each($crate::PrefixPresent(&mut all_fields_present));
                all_fields_present
            }
        }
    };
}

pub struct PrefixPresent<'a>(pub &'a mut bool);

impl<T> Func<&'_ Option<T>> for PrefixPresent<'_> {
    type Output = ();

    fn call(&mut self, i: &Option<T>) -> Self::Output {
        *self.0 &= i.is_some()
    }
}

impl<T> Func<&'_ Vec<T>> for PrefixPresent<'_> {
    type Output = ();

    fn call(&mut self, i: &Vec<T>) -> Self::Output {
        *self.0 &= !i.is_empty()
    }
}
