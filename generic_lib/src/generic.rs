pub trait Generic {
    /// This should be an HList which is "isomorphic" to `Self`
    type Repr;

    fn into(self) -> Self::Repr;
}

pub fn into_generic<T: Generic>(t: T) -> T::Repr {
    t.into()
}
