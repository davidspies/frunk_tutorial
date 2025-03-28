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
                use $crate::reexports::frunk_utils::MapToList;

                let bool_list = frunk::into_generic(self.to_ref()).map_to_list($crate::Present);
                bool_list.into_iter().all(|x| x)
            }
        }
    };
}

pub struct Present;

impl<T> Func<&'_ Option<T>> for Present {
    type Output = bool;

    fn call(&mut self, i: &Option<T>) -> Self::Output {
        i.is_some()
    }
}

impl<T> Func<&'_ Vec<T>> for Present {
    type Output = bool;

    fn call(&mut self, i: &Vec<T>) -> Self::Output {
        !i.is_empty()
    }
}
