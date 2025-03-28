use frunk::{HCons, HNil, hlist::h_cons};

use generic_lib::{AllFieldsPresent, AllFieldsPresentFromOwned};

pub struct Foo {
    field1: Vec<i32>,
    field2: Option<char>,
    field3: Vec<String>,
    field4: Option<String>,
}

type FooHListRepr =
    HCons<Vec<i32>, HCons<Option<char>, HCons<Vec<String>, HCons<Option<String>, HNil>>>>;

impl Foo {
    fn into_hlist_repr(self) -> FooHListRepr {
        h_cons(
            self.field1,
            h_cons(self.field2, h_cons(self.field3, h_cons(self.field4, HNil))),
        )
    }
}

impl AllFieldsPresentFromOwned for Foo {
    fn all_fields_present(self) -> bool {
        self.into_hlist_repr().all_fields_present()
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
