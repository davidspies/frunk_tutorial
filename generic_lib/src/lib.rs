use frunk::{HCons, HNil};

pub trait AllFieldsPresent {
    fn all_fields_present(&self) -> bool;
}

pub trait AllFieldsPresentFromOwned {
    fn all_fields_present(self) -> bool;
}

#[macro_export]
macro_rules! derive_all_fields_present {
    ($t:ty) => {
        impl $crate::AllFieldsPresent for Foo {
            fn all_fields_present(&self) -> bool {
                $crate::AllFieldsPresentFromOwned::all_fields_present(frunk::into_generic(self.to_ref()))
            }
        }
    };
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

impl<'a, T: Present> Present for &'a T {
    fn present(&self) -> bool {
        T::present(self)
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
