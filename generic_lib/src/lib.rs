use std::marker::PhantomData;

use frunk_utils::Func;
use ndarray::{ArcArray, Array, ArrayView, Dimension};

pub mod reexports {
    pub use ::frunk_utils;
}

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

#[macro_export]
macro_rules! impl_array_fields {
    ($name:ident) => {
        impl $crate::ArrayFields for $name<$crate::Owned> {
            type Partial = $name<$crate::Partial>;
            type Arcs = $name<$crate::Arcd>;
            type Views<'a> = $name<$crate::View<'a>>;

            fn build(partial: Self::Partial) -> Result<Self, Self::Partial> {
                use ::frunk::ToRef;
                use $crate::reexports::frunk_utils::WithGeneric;

                let mut all_fields_present = true;
                partial
                    .to_ref()
                    .for_each($crate::AllFieldsPresent(&mut all_fields_present));
                if !all_fields_present {
                    return Err(partial);
                }
                Ok(partial.hmap($crate::UnwrapFields))
            }

            fn views(&self) -> Self::Views<'_> {
                use ::frunk::ToRef;
                use $crate::reexports::frunk_utils::WithGeneric;

                self.to_ref().hmap($crate::FieldViews::default())
            }

            fn arcs(self) -> Self::Arcs {
                use $crate::reexports::frunk_utils::WithGeneric;

                self.hmap($crate::FieldArcs)
            }
        }
    };
}
