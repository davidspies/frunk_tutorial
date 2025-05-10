use generic_lib::ArrayFields;
use ndarray::{
    ArcArray, ArcArray1, ArcArray2, Array1, Array2, Array3, ArrayView1, ArrayView2, ArrayView3, Ix3,
};

pub struct SimulationState {
    pub positions: Array2<f64>,
    pub velocities: Array2<f64>,
    pub particle_types: Array1<i32>,
    pub is_active_mask: Array1<bool>,
    pub density_field: Array3<f32>,
    pub event_timestamps: Array1<i64>,
    pub connectivity_matrix: Array2<u8>,
    pub sensor_readings: Array2<f32>,
}

pub struct PartialSimulationState {
    pub positions: Option<Array2<f64>>,
    pub velocities: Option<Array2<f64>>,
    pub particle_types: Option<Array1<i32>>,
    pub is_active_mask: Option<Array1<bool>>,
    pub density_field: Option<Array3<f32>>,
    pub event_timestamps: Option<Array1<i64>>,
    pub connectivity_matrix: Option<Array2<u8>>,
    pub sensor_readings: Option<Array2<f32>>,
}

pub struct SimulationStateArcs {
    pub positions: ArcArray2<f64>,
    pub velocities: ArcArray2<f64>,
    pub particle_types: ArcArray1<i32>,
    pub is_active_mask: ArcArray1<bool>,
    pub density_field: ArcArray<f32, Ix3>,
    pub event_timestamps: ArcArray1<i64>,
    pub connectivity_matrix: ArcArray2<u8>,
    pub sensor_readings: ArcArray2<f32>,
}

pub struct SimulationStateView<'a> {
    pub positions: ArrayView2<'a, f64>,
    pub velocities: ArrayView2<'a, f64>,
    pub particle_types: ArrayView1<'a, i32>,
    pub is_active_mask: ArrayView1<'a, bool>,
    pub density_field: ArrayView3<'a, f32>,
    pub event_timestamps: ArrayView1<'a, i64>,
    pub connectivity_matrix: ArrayView2<'a, u8>,
    pub sensor_readings: ArrayView2<'a, f32>,
}

impl PartialSimulationState {
    fn all_fields_present(&self) -> bool {
        let Self {
            positions,
            velocities,
            particle_types,
            is_active_mask,
            density_field,
            event_timestamps,
            connectivity_matrix,
            sensor_readings,
        } = self;
        positions.is_some()
            && velocities.is_some()
            && particle_types.is_some()
            && is_active_mask.is_some()
            && density_field.is_some()
            && event_timestamps.is_some()
            && connectivity_matrix.is_some()
            && sensor_readings.is_some()
    }
}

impl ArrayFields for SimulationState {
    type Partial = PartialSimulationState;
    type Arcs = SimulationStateArcs;
    type Views<'a> = SimulationStateView<'a>;

    fn build(partial: Self::Partial) -> Result<Self, Self::Partial> {
        if !partial.all_fields_present() {
            return Err(partial);
        }
        Ok(SimulationState {
            positions: partial.positions.unwrap(),
            velocities: partial.velocities.unwrap(),
            particle_types: partial.particle_types.unwrap(),
            is_active_mask: partial.is_active_mask.unwrap(),
            density_field: partial.density_field.unwrap(),
            event_timestamps: partial.event_timestamps.unwrap(),
            connectivity_matrix: partial.connectivity_matrix.unwrap(),
            sensor_readings: partial.sensor_readings.unwrap(),
        })
    }

    fn views(&self) -> Self::Views<'_> {
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

    fn arcs(self) -> Self::Arcs {
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
