pub trait ToRef<'a> {
    type Output;

    fn to_ref(&'a self) -> Self::Output;
}
