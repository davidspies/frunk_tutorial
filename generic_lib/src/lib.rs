use std::marker::PhantomData;

use ndarray::{ArcArray, Array, ArrayView};

pub trait Domain {
    type Array<DType: 'static, Idx>;
}

pub struct Owned;
impl Domain for Owned {
    type Array<DType: 'static, Idx> = Array<DType, Idx>;
}

pub struct Partial;
impl Domain for Partial {
    type Array<DType: 'static, Idx> = Option<Array<DType, Idx>>;
}

pub struct Arcd;
impl Domain for Arcd {
    type Array<DType: 'static, Idx> = ArcArray<DType, Idx>;
}

pub struct View<'a>(PhantomData<&'a ()>);
impl<'a> Domain for View<'a> {
    type Array<DType: 'static, Idx> = ArrayView<'a, DType, Idx>;
}

pub trait ArrayCarrier {
    type Partial;
    type Arcd;
    type View<'a>
    where
        Self: 'a;

    fn build(partial: Self::Partial) -> Result<Self, Self::Partial>
    where
        Self: Sized;
    fn views(&self) -> Self::View<'_>;
    fn arcs(self) -> Self::Arcd;
}
