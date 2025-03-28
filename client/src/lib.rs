use frunk::Generic;

use generic_lib::{AllFieldsPresent, derive_all_fields_present_from_owned};

#[derive(Generic)]
pub struct Foo {
    field1: Vec<i32>,
    field2: Option<char>,
    field3: Vec<String>,
    field4: Option<String>,
}
derive_all_fields_present_from_owned!(Foo);

#[derive(Generic)]
pub struct FooRef<'a> {
    field1: &'a Vec<i32>,
    field2: &'a Option<char>,
    field3: &'a Vec<String>,
    field4: &'a Option<String>,
}

impl Foo {
    fn to_ref<'a>(&'a self) -> FooRef<'a> {
        FooRef {
            field1: &self.field1,
            field2: &self.field2,
            field3: &self.field3,
            field4: &self.field4,
        }
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
