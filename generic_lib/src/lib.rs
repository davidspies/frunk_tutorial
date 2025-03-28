pub trait AllFieldsPresent {
    fn all_fields_present(&self) -> bool;
}

pub trait AllFieldsPresentFromOwned {
    fn all_fields_present(self) -> bool;
}
