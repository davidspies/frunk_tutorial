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
