use frunk::{HCons, HNil};

use generic_lib::AllFieldsPresentFromOwned;

fn check_all_fields_present_from_owned<T: AllFieldsPresentFromOwned>() {}

type MyHList = HCons<Vec<i32>, HCons<Vec<String>, HCons<Option<usize>, HNil>>>;

#[test]
fn check_my_hlist() {
    check_all_fields_present_from_owned::<MyHList>()
}
