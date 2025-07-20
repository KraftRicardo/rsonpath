use crate::lookup_table::implementations::lut_ptr_hash_double::LutPtrHashDouble;
use crate::lookup_table::performance::lut_skip_evaluation::SkipMode;

pub mod analysis;
pub mod final_results;
pub mod implementations;
pub mod packed_stacked_frame;
pub mod pair_data;
pub mod performance;
pub mod pokemon_test_data_generator;
pub mod query_with_lut;
pub mod sichash_test_data_generator;
pub mod util_path;

// CONFIG
pub const SKIP_MODE: SkipMode = SkipMode::TRACK;
pub const TRACK_SKIPPING_ON: bool = true;
pub const REPETITIONS: u64 = 1;

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
