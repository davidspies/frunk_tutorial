use frunk::HList;

use generic_lib::AllFieldsPresentFromOwned;

fn check_all_fields_present_from_owned<T: AllFieldsPresentFromOwned>() {}

type MyHList = HList![Vec<i32>, Vec<String>, Option<usize>];

#[test]
fn check_my_hlist() {
    check_all_fields_present_from_owned::<MyHList>()
}
