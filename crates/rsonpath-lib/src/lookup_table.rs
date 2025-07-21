use crate::lookup_table::luts::lut_ptr_hash_double::LutPtrHashDouble;
use crate::lookup_table::speed::lut_skip_evaluation::SkipMode;

pub mod analysis;
pub mod extra;
pub mod luts;
pub mod performance;
pub mod speed;

// CONFIG
pub const SKIP_MODE: SkipMode = SkipMode::TRACK;
pub const TRACK_SKIPPING_ON: bool = true;
// pub const QUERY_REPETITIONS: usize = 20;
pub const QUERY_REPETITIONS: usize = 1;
// pub const BUILD_REPETITIONS: usize = 3;
pub const BUILD_REPETITIONS: usize = 1;

// Different LUT implementation approaches and experiments. LutPtrHashDouble and LutVFuncDouble
// perform the best by far memory, build time and query speed wise.
// pub type LUT = LutHashMap;
// pub type LUT = LutHashMapDouble;
// pub type LUT = LutHashMapGroup;
// pub type LUT = LutPHF;
// pub type LUT = LutPHFDouble;
// pub type LUT = LutPHFGroup;
// pub type LUT = LutSicHash; // bugged and broken
pub type LUT = LutPtrHashDouble;
// pub type LUT = LutVFuncDouble;

/// Lookup-table = LUT
pub trait LookUpTable {
    fn build(json_path: &str, cutoff: usize) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;

    fn get(&self, key: &usize) -> Option<usize>;

    fn get_cutoff(&self) -> usize;
}

pub trait LookUpTableLambda: LookUpTable {
    fn build_lambda(
        lambda: usize,
        json_path: &str,
        cutoff: usize,
        threaded: bool,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}

// TODO if time? fn build(json_path: &str) -> Result<Self, Box<dyn std::error::Error + Sync + Send>>
// TODO if time? Consider builder pattern
