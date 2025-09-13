use crate::classification;
use crate::classification::simd::Simd;
use crate::input::{self, error, Input};
use crate::lookup_table::luts::pair_data;
use crate::lookup_table::speed::eval_distance_cutoff::heap_value;
use crate::lookup_table::speed::eval_lut_construction::HEAP_TRACKER;
use crate::lookup_table::speed::query_data::{
    extract_input, QUERY_BESTBUY, QUERY_CROSSREF1, QUERY_CROSSREF2, QUERY_CROSSREF4, QUERY_GOOGLE, QUERY_NSPL,
    QUERY_TWITTER, QUERY_WALMART, QUERY_WIKI, QUERY_WIKI_SINGLE,
};
use crate::lookup_table::{LookUpTable, BUILD_REPETITIONS, LUT};
use csv::Writer;
use stats_alloc::Region;
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;
use std::time::Instant;

/// Benchmark the construction time and memory usage of lookup tables (LUTs).
///
/// This function measures:
/// - The average build time of LUTs across multiple repetitions.
/// - The memory footprint (heap usage) of the LUTs.
/// - The performance of pair collection routines for the given datasets.
///
/// Results are written to CSV files in `result_dir_path`, with one file for LUT
/// build metrics and another for pair collection metrics.
///
/// # Arguments
/// * `data_dir_path`   – Path to the input data directory containing JSON query files.
/// * `result_dir_path` – Path to the output directory where results will be stored.
///
/// # Notes
/// - If the `track-skipping` feature is enabled, this function exits early since
///   skip tracking interferes with accurate benchmarking.
/// - CSV files are created if they do not exist, and appended to otherwise.
/// - The function prints intermediate measurements to stdout for progress tracking.
///
/// # Panics
/// Panics if directories or files cannot be created or written, or if LUT
/// construction fails unexpectedly.
///
/// # Example
/// ```bash
/// cargo run --bin lut --release -- eval-lut-build-speed-and-size res/json res/data/speed/local/lut_build_speed_and_size
/// cargo run --bin lut --release -- eval-lut-build-speed-and-size ricardo-jsons plot-results
/// ```
#[inline]
pub fn evaluate_lut_build_speed_and_size(data_dir_path: &str, result_dir_path: &str) {
    println!("eval_lut_build_speed_and_size");

    let cutoffs = vec![0, 64];
    // let cutoffs = vec![
    //     0, 64, 128, 192, 256, 320, 384, 448, 512, 576, 640, 1024, 2048, 4096, 8192,
    // ];

    if cfg! {feature = "track-skipping"} {
        println!("Disable tracking of skips before running because it slows down the algorithm.");
        return;
    }

    fs::create_dir_all(result_dir_path).expect("Failed to create directory");

    // GB_1
    eval_all(data_dir_path, result_dir_path, QUERY_BESTBUY, &cutoffs);
    eval_all(data_dir_path, result_dir_path, QUERY_CROSSREF1, &cutoffs);
    eval_all(data_dir_path, result_dir_path, QUERY_CROSSREF2, &cutoffs);
    eval_all(data_dir_path, result_dir_path, QUERY_CROSSREF4, &cutoffs);
    eval_all(data_dir_path, result_dir_path, QUERY_GOOGLE, &cutoffs);
    eval_all(data_dir_path, result_dir_path, QUERY_NSPL, &cutoffs);
    eval_all(data_dir_path, result_dir_path, QUERY_TWITTER, &cutoffs);
    eval_all(data_dir_path, result_dir_path, QUERY_WALMART, &cutoffs);
    eval_all(data_dir_path, result_dir_path, QUERY_WIKI, &cutoffs);

    println!("Done");
}

fn eval_all(data_dir_path: &str, result_dir_path: &str, query_data_csv: &str, cutoffs: &Vec<usize>) {
    let (json_path, _, _) = extract_input(data_dir_path, query_data_csv);

    // All necessary paths to CSV and PNG
    let build_csv = format!("{result_dir_path}/build_repetitions={BUILD_REPETITIONS}.csv");
    let file_exists = Path::new(&build_csv).exists();

    // Open CSV in append mode
    let mut wtr = Writer::from_writer(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&build_csv)
            .expect("Failed to open build CSV"),
    );

    // Write header if the file is new
    if !file_exists {
        wtr.write_record([
            "JSON",
            "CUTOFF",
            "BUILD_TIME_SECONDS",
            "COLLECTION_TIME_SECONDS",
            "SIZE_IN_BYTES",
            "REPETITIONS",
        ])
        .expect("Failed to write header");
    }

    // Measurements
    for cutoff in cutoffs {
        println!("  cutoff: {cutoff}");

        // Measure size
        let start_heap = Region::new(HEAP_TRACKER);
        let lut = LUT::build(&json_path, *cutoff).expect("Failed to build LUT");
        let heap_bytes = heap_value(start_heap.change());
        println!("Measured heap with cutoff={}", lut.get_cutoff());
        drop(lut);

        // Warm-up
        for _ in 0..BUILD_REPETITIONS {
            let _ = LUT::build(&json_path, *cutoff).expect("Warm-up build failed");
        }

        // Measure build time
        let mut total_time_build = 0.0;
        for _ in 0..BUILD_REPETITIONS {
            let start = Instant::now();
            let _ = LUT::build(&json_path, *cutoff).expect("Timed build failed");
            total_time_build += start.elapsed().as_secs_f64();
        }
        let avg_time_build = total_time_build / BUILD_REPETITIONS as f64;

        // Warm-up
        for _ in 0..BUILD_REPETITIONS {
            let _ = collect_without_result(&json_path);
        }

        // Measure pair collection time
        let mut total_time_collection = 0.0;
        for _ in 0..BUILD_REPETITIONS {
            let start = Instant::now();
            let _ = collect_without_result(&json_path);
            total_time_collection += start.elapsed().as_secs_f64();
        }
        let avg_time_collection = total_time_collection / BUILD_REPETITIONS as f64;

        // Write the results
        println!(" build time:{avg_time_build:.5}s, collection:{avg_time_collection:.5}s, size:{heap_bytes}B");
        wtr.write_record([
            query_data_csv,
            &format!("{cutoff}"),
            &format!("{avg_time_build}"),
            &format!("{avg_time_collection}"),
            &heap_bytes.to_string(),
            &format!("{BUILD_REPETITIONS}"),
        ])
        .expect("Failed to write build record");

        wtr.flush().expect("Failed to flush build CSV");
    }
}

fn collect_without_result(json_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(json_path).expect("Failed to open file");
    // SAFETY: We keep the file open throughout the entire duration.
    let input = unsafe { input::MmapInput::map_file(&file)? };
    let simd_c = classification::simd::configure();

    classification::simd::config_simd!(simd_c => |simd| {
        classification::simd::dispatch_simd!(simd; &input, simd => fn<I, V>(
            input: &I,
            simd: V,
        ) -> Result<(), error::InputError> where
        I: Input,
        V: Simd, {
                let _ = pair_data::find_pairs(input, simd, 0)?;
                Ok(())
            })
    })
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
