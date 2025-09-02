use crate::lookup_table::analysis::distance_distribution_per_json;
use crate::lookup_table::extra::util_path;
use crate::lookup_table::luts::lut_hash_map::LutHashMap;
use crate::lookup_table::luts::lut_hash_map_double::LutHashMapDouble;
use crate::lookup_table::luts::lut_hash_map_group::LutHashMapGroup;
use crate::lookup_table::luts::lut_perfect_naive::LutPerfectNaive;
use crate::lookup_table::luts::lut_phf::LutPHF;
use crate::lookup_table::luts::lut_phf_double::LutPHFDouble;
use crate::lookup_table::luts::lut_phf_group::LutPHFGroup;
use crate::lookup_table::luts::lut_ptr_hash_double::LutPtrHashDouble;
use crate::lookup_table::luts::lut_vfunc_double::LutVFuncDouble;
use crate::lookup_table::luts::pair_data;
use crate::lookup_table::speed::query_data::*;
use crate::lookup_table::{LookUpTable, LookUpTableLambda, BUILD_REPETITIONS, QUERY_REPETITIONS};
use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;
use std::fs;
use std::io::Write;

/// Allocator to track how much allocations are happening during a specific time frame
#[global_allocator]
pub static HEAP_TRACKER: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

/// Helper struct to reduce the number of parameters when calling functions
pub struct EvalConfig<'a> {
    pub(crate) json_path: &'a str,
    pub(crate) keys: Vec<usize>,
    pub(crate) head_line: &'a mut String,
    pub(crate) data_line: &'a mut String,
}

/// This experiment measures the build_time, key-access-time (query every key once) and heap size
/// of the different LUT implementations that are enabled in the lut_evaluations.rs.
/// The output is a csv holding the results for each LUT implementation on different, e.g.:
///     name,input_size_bytes,num_keys,perfect_naive_BUILD,perfect_naive_QUERY,perfect_naive_HEAP, ...
///     bestbuy_short_(103MB),103231582,680183,0.211277691,0.014003506,58966256,0.065568449, ...
///     crossref0_(320MB),320401529,3130672,1.353163136,0.063439373,1201499616,0.489899481, ...
///     ...
/// The output will add more columns the more different LUT implementations are added.
///
/// Run with: cargo run --bin lut --release -- eval-lut-construction res/json res/data/speed/local/lut_construction
/// Run with: cargo run --bin lut --release -- eval-lut-construction ricardo-jsons final-results-10
#[inline]
pub fn run(data_dir_path: &str, result_dir_path: &str) {
    println!("eval-lut-construction");

    let cutoff: usize = 0;

    fs::create_dir_all(result_dir_path).expect("Failed to create directory");

    // MB_100
    eval_all(data_dir_path, result_dir_path, QUERY_BESTBUY_SHORT, cutoff);

    // GB_1
    // eval_all(data_dir_path, result_dir_path, QUERY_BESTBUY, cutoff);
    // eval_all(data_dir_path, result_dir_path, QUERY_CROSSREF1, cutoff);
    // eval_all(data_dir_path, result_dir_path, QUERY_CROSSREF2, cutoff);
    // eval_all(data_dir_path, result_dir_path, QUERY_CROSSREF4, cutoff);
    // eval_all(data_dir_path, result_dir_path, QUERY_GOOGLE, cutoff);
    // eval_all(data_dir_path, result_dir_path, QUERY_NSPL, cutoff);
    // eval_all(data_dir_path, result_dir_path, QUERY_TWITTER, cutoff);
    // eval_all(data_dir_path, result_dir_path, QUERY_WALMART, cutoff);
    // eval_all(data_dir_path, result_dir_path, QUERY_WIKI, cutoff);

    // 25 GB
    // eval_all(data_dir_path, result_dir_path, QUERY_NESTED_COL);

    println!("Done");
}

fn eval_all(data_dir_path: &str, final_dir_path: &str, query_data_csv: &str, cutoff: usize) {
    let json_path = extract_path(data_dir_path, query_data_csv);
    println!("Cutoff = {cutoff}");

    let file = fs::File::open(&json_path).expect("Fail");
    let filename = util_path::extract_filename(&json_path);
    let num_keys = distance_distribution_per_json::count_num_pairs(&json_path);

    let mut head_line = String::from("name,input_size_bytes,num_keys,");
    let mut data_line = format!("{},{},{},", filename, file.metadata().expect("fail").len(), num_keys);

    let (keys, _) = pair_data::get_pairs_absolute(&json_path, cutoff).expect("Fail @ finding pairs.");

    let mut config = EvalConfig {
        json_path: &json_path,
        keys,
        head_line: &mut head_line,
        data_line: &mut data_line,
    };

    println!("Measuring LUT size with REPETITIONS = {QUERY_REPETITIONS}");
    measure_performance(&mut config, cutoff).expect("Fail");

    // Write CSV header and data
    let csv_path = format!("{final_dir_path}/result.csv");
    let mut csv_file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(csv_path)
        .expect("Fail");
    if csv_file.metadata().expect("Fail").len() == 0 {
        writeln!(csv_file, "{head_line}").expect("Fail");
    }
    writeln!(csv_file, "{data_line}").expect("Fail");
}

#[inline]
pub fn measure_performance(config: &mut EvalConfig, cutoff: usize) -> Result<(), Box<dyn std::error::Error>> {
    // Measure normal LUTs without any special parameter
    eval::<LutPerfectNaive>(config, "perfect_naive", cutoff);
    eval::<LutHashMap>(config, "hash_map", cutoff);
    eval::<LutHashMapDouble>(config, "hash_map_double", cutoff);
    // eval::<LutSicHashDouble>(config, "sic_hash_double", cutoff); // BROKEN
    eval::<LutPtrHashDouble>(config, "ptr_hash_double", cutoff);
    eval::<LutVFuncDouble>(config, "vfunc_double", cutoff);

    for bit_mask in [7, 15] {
        eval_hash_map_group(config, "hash_map_group", bit_mask, cutoff); // BROKEN
    }

    // Measure LUTs with lambda parameter
    for lambda in [1, 5] {
        for threaded in [true, false] {
            eval_phf::<LutPHF>(config, "phf", lambda, threaded, cutoff);
            eval_phf::<LutPHFDouble>(config, "phf_double", lambda, threaded, cutoff);
        }
    }

    // Measure LUTs with bucket parameter
    for lambda in [1, 5] {
        for bit_mask in [3, 7, 15, 31, 63, 127] {
            // for bit_mask in [63, 127, 255, 511] {
            // for bit_mask in [2047, 4095, 8191] {
            // for bit_mask in [2047] {
            // for bit_mask in [63] {
            eval_phf_group(config, "phf_group", bit_mask, lambda, false, cutoff);
        }
    }

    Ok(())
}

fn eval<T: LookUpTable>(config: &mut EvalConfig, name: &str, cutoff: usize) {
    println!("  - {name}");

    // Build time
    // We do it like because the drop() of a big LUT could cost time that we do not want to include in the measurement
    let mut build_time: f64 = 0.0;
    for _i in 0..BUILD_REPETITIONS {
        let start_build = std::time::Instant::now();
        let _ = T::build(config.json_path, cutoff).expect("Fail @ build lut");
        build_time += start_build.elapsed().as_secs_f64();
    }
    build_time /= BUILD_REPETITIONS as f64;

    // Size
    let start_heap = Region::new(HEAP_TRACKER);
    let lut = T::build(config.json_path, cutoff).expect("Fail @ build lut");
    let heap_bytes = heap_value(start_heap.change());

    // Query time
    let mut query_time: f64 = 0.0;
    for _i in 0..QUERY_REPETITIONS {
        let start_query = std::time::Instant::now();
        my_black_box(get_every_key_once(&lut, &config.keys));
        query_time += start_query.elapsed().as_secs_f64();
    }
    query_time /= QUERY_REPETITIONS as f64;

    // Save measurements
    save_measurements(config, name, build_time, query_time, heap_bytes);
}

fn eval_phf<T: LookUpTableLambda>(config: &mut EvalConfig, name: &str, lambda: usize, threaded: bool, cutoff: usize) {
    println!("  - {name}:λ={lambda},threaded={threaded},cutoff={cutoff}");

    // Build time
    let mut build_time: f64 = 0.0;
    for _i in 0..BUILD_REPETITIONS {
        let start_build = std::time::Instant::now();
        let _ = T::build_lambda(lambda, config.json_path, 0, threaded).expect("Fail @ build lut");
        build_time += start_build.elapsed().as_secs_f64();
    }
    build_time /= BUILD_REPETITIONS as f64;

    // Size
    let start_heap = Region::new(HEAP_TRACKER);
    let lut = T::build_lambda(lambda, config.json_path, 0, threaded).expect("Fail @ build lut");
    let heap_bytes = heap_value(start_heap.change());

    // Query time
    let mut query_time: f64 = 0.0;
    for _i in 0..QUERY_REPETITIONS {
        let start_query = std::time::Instant::now();
        my_black_box(get_every_key_once(&lut, &config.keys));
        query_time += start_query.elapsed().as_secs_f64();
    }
    query_time /= QUERY_REPETITIONS as f64;

    // Save measurements
    let name = format!("λ={lambda}:{name}");
    save_measurements(config, &name, build_time, query_time, heap_bytes);
}

fn eval_phf_group(config: &mut EvalConfig, name: &str, bit_mask: usize, lambda: usize, threaded: bool, cutoff: usize) {
    let buckets = bit_mask + 1;
    println!("  - {name}:#{buckets}_λ={lambda}");

    // Build time
    let mut build_time: f64 = 0.0;
    for _i in 0..BUILD_REPETITIONS {
        let start_build = std::time::Instant::now();
        let _ =
            LutPHFGroup::build_buckets(lambda, config.json_path, cutoff, bit_mask, threaded).expect("Fail @ build lut");
        build_time += start_build.elapsed().as_secs_f64();
    }
    build_time /= BUILD_REPETITIONS as f64;

    // Size
    let start_heap = Region::new(HEAP_TRACKER);
    let lut =
        LutPHFGroup::build_buckets(lambda, config.json_path, cutoff, bit_mask, threaded).expect("Fail @ build lut");
    let heap_bytes = heap_value(start_heap.change());

    // Query time
    let mut query_time: f64 = 0.0;
    for _i in 0..QUERY_REPETITIONS {
        let start_query = std::time::Instant::now();
        my_black_box(get_every_key_once(&lut, &config.keys));
        query_time += start_query.elapsed().as_secs_f64();
    }
    query_time /= QUERY_REPETITIONS as f64;

    // Save measurements
    let name = format!("#{buckets}_λ={lambda}:{name}");
    save_measurements(config, &name, build_time, query_time, heap_bytes);
}

fn eval_hash_map_group(config: &mut EvalConfig, name: &str, bit_mask: usize, cutoff: usize) {
    let buckets = bit_mask + 1;
    println!("  - {name}:#{buckets}");

    // Build time
    let mut build_time: f64 = 0.0;
    for _i in 0..BUILD_REPETITIONS {
        let start_build = std::time::Instant::now();
        let _ = LutHashMapGroup::build_buckets(config.json_path, bit_mask, cutoff).expect("Fail @ build lut");
        build_time += start_build.elapsed().as_secs_f64();
    }
    build_time /= BUILD_REPETITIONS as f64;

    // Size
    let start_heap = Region::new(HEAP_TRACKER);
    let lut = LutHashMapGroup::build_buckets(config.json_path, bit_mask, cutoff).expect("Fail @ build lut");
    let heap_bytes = heap_value(start_heap.change());

    // Query time
    let mut query_time: f64 = 0.0;
    for _i in 0..QUERY_REPETITIONS {
        let start_query = std::time::Instant::now();
        my_black_box(get_every_key_once(&lut, &config.keys));
        query_time += start_query.elapsed().as_secs_f64();
    }
    query_time /= QUERY_REPETITIONS as f64;

    // Save measurements
    let name = format!("#{buckets}:{name}");
    save_measurements(config, &name, build_time, query_time, heap_bytes);
}

fn save_measurements(config: &mut EvalConfig, f: &str, build: f64, query: f64, heap: isize) {
    config.head_line.push_str(&format!("{f}_BUILD,{f}_QUERY,{f}_HEAP,",));
    config.data_line.push_str(&format!("{build},{query},{heap},"));

    println!("    - Build time:      {build}");
    println!("    - Query time:      {query}");
    println!("    - Heap bytes:      {heap}");
}

fn get_every_key_once(lut: &dyn LookUpTable, keys: &[usize]) -> usize {
    let mut count = 0;
    for key in keys {
        count += lut.get(key).expect("Fail at getting value!");
    }
    count
}

fn heap_value(stats: stats_alloc::Stats) -> isize {
    // We take the allocated bytes minus the deallocated and ignore the reallocated bytes because we are interested
    // in the total heap space taken
    stats.bytes_allocated as isize - stats.bytes_deallocated as isize

    // Alternative line that should not be used:
    // stats.bytes_allocated as isize - stats.bytes_deallocated as isize + stats.bytes_reallocated
}

// A black box function so that the compiler will not optimize away the values passed into here. Mainly used when
// running tests.
#[inline(never)]
fn my_black_box<T>(_whatever: T) {}
