use frunk::{HList, hlist};

use generic_lib::{AllFieldsPresent, AllFieldsPresentFromOwned};

pub struct Foo {
    field1: Vec<i32>,
    field2: Option<char>,
    field3: Vec<String>,
    field4: Option<String>,
}

type FooHListRepr = HList![Vec<i32>, Option<char>, Vec<String>, Option<String>];

impl Foo {
    fn into_hlist_repr(self) -> FooHListRepr {
        hlist![self.field1, self.field2, self.field3, self.field4]
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
