use frunk_utils::Func;

pub trait AllFieldsPresent {
    fn all_fields_present(&self) -> bool;
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
