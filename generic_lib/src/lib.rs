use std::marker::PhantomData;

use frunk_utils::Func;
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

pub struct AllFieldsPresent<'a>(pub &'a mut bool);
impl<'a, T> Func<&'a Option<T>> for AllFieldsPresent<'_> {
    type Output = ();

    fn call(&mut self, i: &'a Option<T>) -> Self::Output {
        *self.0 &= i.is_some()
    }
}

pub struct UnwrapFields;
impl<T> Func<Option<T>> for UnwrapFields {
    type Output = T;

    fn call(&mut self, i: Option<T>) -> Self::Output {
        i.unwrap()
    }
}

#[derive(Default)]
pub struct FieldViews<'a>(PhantomData<&'a ()>);
impl<'a, A, Idx: Dimension> Func<&'a Array<A, Idx>> for FieldViews<'a> {
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

pub struct Owned;
pub struct Partial;
pub struct Arcd;
pub struct View<'a>(PhantomData<&'a ()>);

pub trait Domain {
    type Array<DType: 'static, Idx>;
}

impl Domain for Owned {
    type Array<DType: 'static, Idx> = ndarray::Array<DType, Idx>;
}

impl Domain for Partial {
    type Array<DType: 'static, Idx> = Option<ndarray::Array<DType, Idx>>;
}

impl Domain for Arcd {
    type Array<DType: 'static, Idx> = ndarray::ArcArray<DType, Idx>;
}

impl<'a> Domain for View<'a> {
    type Array<DType: 'static, Idx> = ndarray::ArrayView<'a, DType, Idx>;
}
