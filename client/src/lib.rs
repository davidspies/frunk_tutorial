use frunk::Generic;
use frunk_utils_derives::ToRef;
use generic_lib::{AllFieldsPresent, derive_all_fields_present};
use ndarray::{ArcArray, Array, ArrayView, Ix1, Ix2, Ix3};

pub struct SimulationState {
    pub positions: Array<f64, Ix2>,
    pub velocities: Array<f64, Ix2>,
    pub particle_types: Array<i32, Ix1>,
    pub is_active_mask: Array<bool, Ix1>,
    pub density_field: Array<f32, Ix3>,
    pub event_timestamps: Array<i64, Ix1>,
    pub connectivity_matrix: Array<u8, Ix2>,
    pub sensor_readings: Array<f32, Ix2>,
}

#[derive(Generic, ToRef)]
pub struct PartialSimulationState {
    pub positions: Option<Array<f64, Ix2>>,
    pub velocities: Option<Array<f64, Ix2>>,
    pub particle_types: Option<Array<i32, Ix1>>,
    pub is_active_mask: Option<Array<bool, Ix1>>,
    pub density_field: Option<Array<f32, Ix3>>,
    pub event_timestamps: Option<Array<i64, Ix1>>,
    pub connectivity_matrix: Option<Array<u8, Ix2>>,
    pub sensor_readings: Option<Array<f32, Ix2>>,
}
derive_all_fields_present!(PartialSimulationState);

pub struct SimulationStateArcs {
    pub positions: ArcArray<f64, Ix2>,
    pub velocities: ArcArray<f64, Ix2>,
    pub particle_types: ArcArray<i32, Ix1>,
    pub is_active_mask: ArcArray<bool, Ix1>,
    pub density_field: ArcArray<f32, Ix3>,
    pub event_timestamps: ArcArray<i64, Ix1>,
    pub connectivity_matrix: ArcArray<u8, Ix2>,
    pub sensor_readings: ArcArray<f32, Ix2>,
}

pub struct SimulationStateView<'a> {
    pub positions: ArrayView<'a, f64, Ix2>,
    pub velocities: ArrayView<'a, f64, Ix2>,
    pub particle_types: ArrayView<'a, i32, Ix1>,
    pub is_active_mask: ArrayView<'a, bool, Ix1>,
    pub density_field: ArrayView<'a, f32, Ix3>,
    pub event_timestamps: ArrayView<'a, i64, Ix1>,
    pub connectivity_matrix: ArrayView<'a, u8, Ix2>,
    pub sensor_readings: ArrayView<'a, f32, Ix2>,
}

impl PartialSimulationState {
    pub fn build(self) -> Result<SimulationState, Self> {
        if !self.all_fields_present() {
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
