use frunk::Generic;

use generic_lib::{AllFieldsPresent, AllFieldsPresentFromOwned};

#[derive(Generic)]
pub struct Foo {
    field1: Vec<i32>,
    field2: Option<char>,
    field3: Vec<String>,
    field4: Option<String>,
}

impl AllFieldsPresentFromOwned for Foo {
    fn all_fields_present(self) -> bool {
        frunk::into_generic(self).all_fields_present()
    }
}

impl AllFieldsPresent for Foo {
    fn all_fields_present(&self) -> bool {
        !self.field1.is_empty()
            && self.field2.is_some()
            && !self.field3.is_empty()
            && self.field4.is_some()
    }
}
