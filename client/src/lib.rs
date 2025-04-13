use frunk::Generic;
use ndarray::{ArcArray, Ix1, Ix2, Ix3};

use generic_lib::{Arcd, Domain, Owned, Partial, View};

#[derive(Generic)]
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

impl PartialSimulationState {
    pub fn build(self) -> Result<SimulationState, Self> {
        let all_fields_present = self.positions.is_some()
            && self.velocities.is_some()
            && self.particle_types.is_some()
            && self.is_active_mask.is_some()
            && self.density_field.is_some()
            && self.event_timestamps.is_some()
            && self.connectivity_matrix.is_some()
            && self.sensor_readings.is_some();
        if !all_fields_present {
            return Err(self);
        }
        Ok(SimulationState {
            positions: self.positions.unwrap(),
            velocities: self.velocities.unwrap(),
            particle_types: self.particle_types.unwrap(),
            is_active_mask: self.is_active_mask.unwrap(),
            density_field: self.density_field.unwrap(),
            event_timestamps: self.event_timestamps.unwrap(),
            connectivity_matrix: self.connectivity_matrix.unwrap(),
            sensor_readings: self.sensor_readings.unwrap(),
        })
    }
}

impl SimulationState {
    pub fn views(&self) -> SimulationStateView {
        SimulationStateView {
            positions: self.positions.view(),
            velocities: self.velocities.view(),
            particle_types: self.particle_types.view(),
            is_active_mask: self.is_active_mask.view(),
            density_field: self.density_field.view(),
            event_timestamps: self.event_timestamps.view(),
            connectivity_matrix: self.connectivity_matrix.view(),
            sensor_readings: self.sensor_readings.view(),
        }
    }

    pub fn arcs(self) -> SimulationStateArcs {
        SimulationStateArcs {
            positions: ArcArray::from(self.positions),
            velocities: ArcArray::from(self.velocities),
            particle_types: ArcArray::from(self.particle_types),
            is_active_mask: ArcArray::from(self.is_active_mask),
            density_field: ArcArray::from(self.density_field),
            event_timestamps: ArcArray::from(self.event_timestamps),
            connectivity_matrix: ArcArray::from(self.connectivity_matrix),
            sensor_readings: ArcArray::from(self.sensor_readings),
        }
    }
}
