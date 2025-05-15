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

pub trait ForEach<F> {
    fn for_each(self, f: F);
}

impl<F> ForEach<F> for HNil {
    fn for_each(self, _f: F) {}
}

impl<F: Func<Head, Output = ()>, Head, Tail: ForEach<F>> ForEach<F> for HCons<Head, Tail> {
    fn for_each(self, mut f: F) {
        let HCons { head, tail } = self;
        f.call(head);
        tail.for_each(f);
    }
}

pub struct AllFieldsPresent<'a>(pub &'a mut bool);
impl<'a, T> Func<&'a Option<T>> for AllFieldsPresent<'_> {
    type Output = ();

    fn call(&mut self, i: &'a Option<T>) -> Self::Output {
        *self.0 &= i.is_some()
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
