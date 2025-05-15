use frunk::{Generic, ToRef};
use frunk_utils_derives::ToRef;
use generic_lib::{AllFieldsPresent, ArrayFields, FieldArcs, FieldViews, HMappable, UnwrapFields};
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
        let hlist = frunk::into_generic(self.to_ref());
        hlist.all_fields_present()
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
        let hlist = frunk::into_generic(partial);
        Ok(frunk::from_generic(HMappable::map(hlist, UnwrapFields)))
    }

    fn views(&self) -> Self::Views<'_> {
        let hlist = frunk::into_generic(self.to_ref());
        frunk::from_generic(HMappable::map(hlist, FieldViews))
    }

    fn arcs(self) -> Self::Arcs {
        let hlist = frunk::into_generic(self);
        frunk::from_generic(HMappable::map(hlist, FieldArcs))
    }
}
