use frunk::{Generic, ToRef};
use frunk_utils_derives::ToRef;

use generic_lib::derive_all_fields_present;

#[derive(Generic, ToRef)]
pub struct Foo {
    field1: Vec<i32>,
    field2: Option<char>,
    field3: Vec<String>,
    field4: Option<String>,
}
derive_all_fields_present!(Foo);
