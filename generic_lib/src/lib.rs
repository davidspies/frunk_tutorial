use std::marker::PhantomData;

use frunk::{HCons, HNil};
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

pub trait Func<Input> {
    type Output;

    fn call(&mut self, i: Input) -> Self::Output;
}

pub trait AllFieldsPresent {
    fn all_fields_present(self) -> bool;
}
impl AllFieldsPresent for HNil {
    fn all_fields_present(self) -> bool {
        true
    }
}
impl<'a, H, T: AllFieldsPresent> AllFieldsPresent for HCons<&'a Option<H>, T> {
    fn all_fields_present(self) -> bool {
        let HCons { head, tail } = self;
        head.is_some() && tail.all_fields_present()
    }
}

pub trait HMappable<Mapper> {
    type Output;

    fn map(self, mapper: Mapper) -> Self::Output;
}

impl<F> HMappable<F> for HNil {
    type Output = HNil;

    fn map(self, _mapper: F) -> Self::Output {
        HNil
    }
}

impl<F: Func<Head>, Head, Tail: HMappable<F>> HMappable<F> for HCons<Head, Tail> {
    type Output = HCons<F::Output, Tail::Output>;

    fn map(self, mut mapper: F) -> Self::Output {
        let HCons { head, tail } = self;
        HCons {
            head: mapper.call(head),
            tail: tail.map(mapper),
        }
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
