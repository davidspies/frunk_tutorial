use std::marker::PhantomData;

use frunk_utils::Func;
use ndarray::{ArcArray, Array, ArrayView, Dimension};

pub mod reexport {
    pub use frunk;
    pub use frunk_utils;
}

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

pub struct IsSome;
impl<'a, T> Func<&'a Option<T>> for IsSome {
    type Output = bool;

    fn call(&mut self, i: &'a Option<T>) -> Self::Output {
        i.is_some()
    }
}

pub struct UnwrapField;
impl<T> Func<Option<T>> for UnwrapField {
    type Output = T;

    fn call(&mut self, i: Option<T>) -> Self::Output {
        i.unwrap()
    }
}

pub struct GetView;
impl<'a, A, D: Dimension> Func<&'a Array<A, D>> for GetView {
    type Output = ArrayView<'a, A, D>;

    fn call(&mut self, i: &'a Array<A, D>) -> Self::Output {
        i.view()
    }
}

pub struct ArcArrayFrom;
impl<A, D: Dimension> Func<Array<A, D>> for ArcArrayFrom {
    type Output = ArcArray<A, D>;

    fn call(&mut self, i: Array<A, D>) -> Self::Output {
        ArcArray::from(i)
    }
}

#[macro_export]
macro_rules! impl_array_carrier {
    ($name:ident) => {
        impl ArrayCarrier for $name<$crate::Owned> {
            type Partial = $name<$crate::Partial>;
            type Arcd = $name<$crate::Arcd>;
            type View<'a> = $name<$crate::View<'a>>;

            fn build(partial: Self::Partial) -> Result<Self, Self::Partial> {
                use $crate::reexport::frunk::ToRef;
                use $crate::reexport::frunk_utils::WithGeneric;
                let all_fields_present = partial
                    .to_ref()
                    .map_to_list($crate::IsSome)
                    .into_iter()
                    .all(|x| x);
                if all_fields_present {
                    Ok(partial.hmap($crate::UnwrapField))
                } else {
                    Err(partial)
                }
            }

            fn views(&self) -> Self::View<'_> {
                use $crate::reexport::frunk::ToRef;
                use $crate::reexport::frunk_utils::WithGeneric;
                self.to_ref().hmap($crate::GetView)
            }

            fn arcs(self) -> Self::Arcd {
                use $crate::reexport::frunk_utils::WithGeneric;
                self.hmap($crate::ArcArrayFrom)
            }
        }
    };
}
