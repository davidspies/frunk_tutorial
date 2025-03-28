pub mod reexports {
    pub use frunk_utils;
}

use frunk_utils::Func;

pub trait AllFieldsPresent {
    fn all_fields_present(&self) -> bool;
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
