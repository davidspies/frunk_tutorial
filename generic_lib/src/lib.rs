use frunk::{Generic, ToRef};
use frunk_utils::{Func, MapToList, WithGeneric};

pub trait AllFieldsPresent {
    fn all_fields_present(&self) -> bool;
}

#[macro_export]
macro_rules! derive_all_fields_present {
    ($t:ty) => {
        impl $crate::AllFieldsPresent for $t {
            fn all_fields_present(&self) -> bool {
                $crate::all_fields_present_helper(self)
            }
        }
    };
}

pub fn all_fields_present_helper<'a, T: ToRef<'a>>(this: &'a T) -> bool
where
    <T as ToRef<'a>>::Output: WithGeneric,
    <<T as ToRef<'a>>::Output as Generic>::Repr: MapToList<Present, bool>,
{
    let bool_list = this.to_ref().map_to_list(Present);
    bool_list.into_iter().all(|x| x)
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
