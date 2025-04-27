use frunk::{Generic, ToRef};
use frunk_utils::{Func, WithGeneric};
use frunk_utils_derives::ToRef;
use ndarray::{ArcArray, Array, ArrayView, Dimension, Ix1, Ix2, Ix3};

use generic_lib::{Arcd, ArrayCarrier, Domain, Owned, Partial, View};

#[derive(Generic, ToRef)]
pub struct SimulationStateG<D: Domain> {
    pub positions: D::Array<f64, Ix2>,
    pub velocities: D::Array<f64, Ix2>,
    pub particle_types: D::Array<i32, Ix1>,
    pub is_active_mask: D::Array<bool, Ix1>,
    pub density_field: D::Array<f32, Ix3>,
    pub event_timestamps: D::Array<i64, Ix1>,
    pub connectivity_matrix: D::Array<u8, Ix2>,
    pub sensor_readings: D::Array<f32, Ix2>,
}

pub type SimulationState = SimulationStateG<Owned>;
pub type PartialSimulationState = SimulationStateG<Partial>;
pub type SimulationStateArcs = SimulationStateG<Arcd>;
pub type SimulationStateView<'a> = SimulationStateG<View<'a>>;

impl ArrayCarrier for SimulationState {
    type Partial = PartialSimulationState;
    type Arcd = SimulationStateArcs;
    type View<'a> = SimulationStateView<'a>;

    fn build(partial: Self::Partial) -> Result<Self, Self::Partial> {
        struct IsSome;
        impl<'a, T> Func<&'a Option<T>> for IsSome {
            type Output = bool;

            fn call(&mut self, i: &'a Option<T>) -> Self::Output {
                i.is_some()
            }
        }

        let all_fields_present = partial.to_ref().map_to_list(IsSome).into_iter().all(|x| x);
        if !all_fields_present {
            return Err(partial);
        }

        struct UnwrapField;
        impl<T> Func<Option<T>> for UnwrapField {
            type Output = T;

            fn call(&mut self, i: Option<T>) -> Self::Output {
                i.unwrap()
            }
        }

        Ok(partial.hmap(UnwrapField))
    }

    fn views(&self) -> Self::View<'_> {
        struct GetView;
        impl<'a, A, D: Dimension> Func<&'a Array<A, D>> for GetView {
            type Output = ArrayView<'a, A, D>;

            fn call(&mut self, i: &'a Array<A, D>) -> Self::Output {
                i.view()
            }
        }

        self.to_ref().hmap(GetView)
    }

    fn arcs(self) -> Self::Arcd {
        struct ArcArrayFrom;
        impl<A, D: Dimension> Func<Array<A, D>> for ArcArrayFrom {
            type Output = ArcArray<A, D>;

            fn call(&mut self, i: Array<A, D>) -> Self::Output {
                ArcArray::from(i)
            }
        }

        self.hmap(ArcArrayFrom)
    }
}
