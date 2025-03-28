use frunk::Generic;
use frunk_utils_derives::ToRef;

use generic_lib::{AllFieldsPresent, PrefixPresent};

#[derive(Generic, ToRef)]
pub struct Foo {
    field1: Vec<i32>,
    field2: Option<char>,
    field3: Vec<String>,
    field4: Option<String>,
}

impl AllFieldsPresent for Foo {
    fn all_fields_present(&self) -> bool {
        use frunk::ToRef;
        use frunk_utils::ForEach;

        let mut all_fields_present = true;
        frunk::into_generic(self.to_ref()).for_each(PrefixPresent(&mut all_fields_present));
        all_fields_present
    }
}
