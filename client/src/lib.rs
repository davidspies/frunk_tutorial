use frunk::Generic;
use frunk_utils_derives::ToRef;
use generic_lib::{Arcd, Domain, Owned, Partial, View, impl_array_fields};
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
impl_array_fields!(SimulationStateG);

pub type SimulationState = SimulationStateG<Owned>;
pub type PartialSimulationState = SimulationStateG<Partial>;
pub type SimulationStateArcs = SimulationStateG<Arcd>;
pub type SimulationStateView<'a> = SimulationStateG<View<'a>>;
