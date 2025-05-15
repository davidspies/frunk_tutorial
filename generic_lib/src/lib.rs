use frunk_utils::{ForEach, Func, WithGeneric};
use ndarray::{ArcArray, Array, ArrayView, Dimension};

pub trait ArrayFields: Sized {
    type Partial;
    type Arcs;
    type Views<'a>
    where
        Self: 'a;

    fn build(partial: Self::Partial) -> Result<Self, Self::Partial>;
    fn views(&self) -> Self::Views<'_>;
    fn arcs(self) -> Self::Arcs;
}

pub struct AllFieldsPresent<'a>(&'a mut bool);
impl<'a, T> Func<&'a Option<T>> for AllFieldsPresent<'_> {
    type Output = ();

    fn call(&mut self, i: &'a Option<T>) -> Self::Output {
        *self.0 &= i.is_some()
    }
}

pub fn all_fields_present<R: for<'a> ForEach<AllFieldsPresent<'a>>>(
    data: impl WithGeneric<Repr = R>,
) -> bool {
    let mut all_present = true;
    data.for_each(AllFieldsPresent(&mut all_present));
    all_present
}

pub struct UnwrapFields;
impl<T> Func<Option<T>> for UnwrapFields {
    type Output = T;

    fn call(&mut self, i: Option<T>) -> Self::Output {
        i.unwrap()
    }
}

pub struct FieldViews;
impl<'a, A, Idx: Dimension> Func<&'a Array<A, Idx>> for FieldViews {
    type Output = ArrayView<'a, A, Idx>;

    fn call(&mut self, i: &'a Array<A, Idx>) -> Self::Output {
        i.view()
    }
}

pub struct FieldArcs;
impl<A, Idx: Dimension> Func<Array<A, Idx>> for FieldArcs {
    type Output = ArcArray<A, Idx>;

    fn call(&mut self, i: Array<A, Idx>) -> Self::Output {
        ArcArray::from(i)
    }
}
