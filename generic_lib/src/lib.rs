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

pub trait UnwrapFields {
    type Unwrapped;

    fn unwrap_fields(self) -> Self::Unwrapped;
}
impl UnwrapFields for HNil {
    type Unwrapped = HNil;

    fn unwrap_fields(self) -> Self::Unwrapped {
        HNil
    }
}
impl<H, T: UnwrapFields> UnwrapFields for HCons<Option<H>, T> {
    type Unwrapped = HCons<H, T::Unwrapped>;

    fn unwrap_fields(self) -> Self::Unwrapped {
        let HCons { head, tail } = self;
        HCons {
            head: head.unwrap(),
            tail: tail.unwrap_fields(),
        }
    }
}

pub trait FieldViews<'a> {
    type Views;

    fn views(self) -> Self::Views;
}
impl<'a> FieldViews<'a> for HNil {
    type Views = HNil;

    fn views(self) -> Self::Views {
        HNil
    }
}
impl<'a, HElem, HIdx: Dimension, T: FieldViews<'a>> FieldViews<'a>
    for HCons<&'a Array<HElem, HIdx>, T>
{
    type Views = HCons<ArrayView<'a, HElem, HIdx>, T::Views>;

    fn views(self) -> Self::Views {
        let HCons { head, tail } = self;
        HCons {
            head: head.view(),
            tail: tail.views(),
        }
    }
}

pub trait FieldArcs {
    type Arcs;

    fn arcs(self) -> Self::Arcs;
}
impl FieldArcs for HNil {
    type Arcs = HNil;

    fn arcs(self) -> Self::Arcs {
        HNil
    }
}
impl<HElem, HIdx: Dimension, T: FieldArcs> FieldArcs for HCons<Array<HElem, HIdx>, T> {
    type Arcs = HCons<ArcArray<HElem, HIdx>, T::Arcs>;

    fn arcs(self) -> Self::Arcs {
        let HCons { head, tail } = self;
        HCons {
            head: ArcArray::from(head),
            tail: tail.arcs(),
        }
    }
}
