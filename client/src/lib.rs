use frunk::{Generic, ToRef};
use frunk_utils::WithGeneric;
use frunk_utils_derives::ToRef;
use generic_lib::{
    Arcd, ArrayFields, Domain, FieldArcs, FieldViews, Owned, Partial, UnwrapFields, View,
};
use ndarray::{Ix1, Ix2, Ix3};

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

impl ArrayFields for SimulationState {
    type Partial = PartialSimulationState;
    type Arcs = SimulationStateArcs;
    type Views<'a> = SimulationStateView<'a>;

    fn build(partial: Self::Partial) -> Result<Self, Self::Partial> {
        if !generic_lib::all_fields_present(partial.to_ref()) {
            return Err(partial);
        }
        Ok(partial.hmap(UnwrapFields))
    }

    fn views(&self) -> Self::Views<'_> {
        self.to_ref().hmap(FieldViews)
    }

    fn arcs(self) -> Self::Arcs {
        self.hmap(FieldArcs)
    }
}
