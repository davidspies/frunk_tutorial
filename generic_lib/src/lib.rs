use frunk::{HCons, HNil};

pub mod generic;

pub use self::generic::Generic;

pub trait AllFieldsPresent {
    fn all_fields_present(&self) -> bool;
}

pub trait AllFieldsPresentFromOwned {
    fn all_fields_present(self) -> bool;
}

trait Present {
    fn present(&self) -> bool;
}

impl<T> Present for Option<T> {
    fn present(&self) -> bool {
        self.is_some()
    }
}

impl<T> Present for Vec<T> {
    fn present(&self) -> bool {
        !self.is_empty()
    }
}

impl AllFieldsPresentFromOwned for HNil {
    fn all_fields_present(self) -> bool {
        true
    }
}

impl<H: Present, T: AllFieldsPresentFromOwned> AllFieldsPresentFromOwned for HCons<H, T> {
    fn all_fields_present(self) -> bool {
        let HCons { head, tail } = self;
        head.present() && tail.all_fields_present()
    }
}
