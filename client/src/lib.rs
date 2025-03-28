use frunk::{Generic, ToRef};
use frunk_utils_derives::ToRef;

use generic_lib::{
    AllFieldsPresent, AllFieldsPresentFromOwned, derive_all_fields_present_from_owned,
};

#[derive(Generic, ToRef)]
pub struct Foo {
    field1: Vec<i32>,
    field2: Option<char>,
    field3: Vec<String>,
    field4: Option<String>,
}
derive_all_fields_present_from_owned!(Foo);

impl AllFieldsPresent for Foo {
    fn all_fields_present(&self) -> bool {
        AllFieldsPresentFromOwned::all_fields_present(frunk::into_generic(self.to_ref()))
    }
}
