use generic_lib::AllFieldsPresentFromOwned;

fn check_all_fields_present_from_owned<T: AllFieldsPresentFromOwned>() {}

type MyConsList = (Vec<i32>, (Vec<String>, (Option<usize>, ())));

#[test]
fn check_my_cons_list() {
    check_all_fields_present_from_owned::<MyConsList>()
}
