use frunk::Generic;
use frunk_utils_derives::ToRef;

use generic_lib::{AllFieldsPresent, Present};

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
        use generic_lib::reexports::frunk_utils::MapToList;

        let bool_list = frunk::into_generic(self.to_ref()).map_to_list(Present);
        bool_list.into_iter().all(|x| x)
    }
}
