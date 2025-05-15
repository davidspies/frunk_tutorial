use frunk::{Generic, ToRef, hlist, hlist_pat};
use frunk_utils_derives::ToRef;
use generic_lib::ArrayFields;
use ndarray::{ArcArray, Array, ArrayView, Ix1, Ix2, Ix3};

#[derive(Generic, ToRef)]
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

#[derive(Generic)]
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

#[derive(Generic)]
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
    fn all_fields_present(&self) -> bool {
        let hlist_pat![
            positions,
            velocities,
            particle_types,
            is_active_mask,
            density_field,
            event_timestamps,
            connectivity_matrix,
            sensor_readings,
        ] = frunk::into_generic(self.to_ref());
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
        let hlist_pat![
            positions,
            velocities,
            particle_types,
            is_active_mask,
            density_field,
            event_timestamps,
            connectivity_matrix,
            sensor_readings,
        ] = frunk::into_generic(partial);
        Ok(frunk::from_generic(hlist![
            positions.unwrap(),
            velocities.unwrap(),
            particle_types.unwrap(),
            is_active_mask.unwrap(),
            density_field.unwrap(),
            event_timestamps.unwrap(),
            connectivity_matrix.unwrap(),
            sensor_readings.unwrap(),
        ]))
    }

    fn views(&self) -> Self::Views<'_> {
        let hlist_pat![
            positions,
            velocities,
            particle_types,
            is_active_mask,
            density_field,
            event_timestamps,
            connectivity_matrix,
            sensor_readings,
        ] = frunk::into_generic(self.to_ref());
        frunk::from_generic(hlist![
            positions.view(),
            velocities.view(),
            particle_types.view(),
            is_active_mask.view(),
            density_field.view(),
            event_timestamps.view(),
            connectivity_matrix.view(),
            sensor_readings.view(),
        ])
    }

    fn arcs(self) -> Self::Arcs {
        let hlist_pat![
            positions,
            velocities,
            particle_types,
            is_active_mask,
            density_field,
            event_timestamps,
            connectivity_matrix,
            sensor_readings,
        ] = frunk::into_generic(self);
        frunk::from_generic(hlist![
            ArcArray::from(positions),
            ArcArray::from(velocities),
            ArcArray::from(particle_types),
            ArcArray::from(is_active_mask),
            ArcArray::from(density_field),
            ArcArray::from(event_timestamps),
            ArcArray::from(connectivity_matrix),
            ArcArray::from(sensor_readings),
        ])
    }
}
