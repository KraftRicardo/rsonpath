use crate::lookup_table::performance::lut_query_data::{
    QUERY_BESTBUY, QUERY_CROSSREF1, QUERY_CROSSREF2, QUERY_CROSSREF4, QUERY_GOOGLE, QUERY_NSPL, QUERY_TWITTER,
    QUERY_WALMART, QUERY_WIKI,
};
use crate::lookup_table::performance::lut_skip_evaluation::SkipMode;
use crate::lookup_table::SKIP_MODE;
use crate::{
    engine::{Compiler, Engine, RsonpathEngine},
    input::OwnedBytes,
    lookup_table::{performance::lut_evaluation::HEAP_TRACKER, util_path, LookUpTable, LUT},
};
use csv::Writer;
use stats_alloc::Region;
use std::fs::OpenOptions;
use std::path::Path;
use std::time::Instant;
use std::{
    fs,
    io::{self, BufReader, Read, Write},
    process::Command,
};

pub const FIND_PAIR_REPETITIONS: usize = 10;

// run with: cargo run --bin lut --release -- eval-find-pair-data .a_test_data .a_final_results
pub fn evaluate(data_dir_path: &str, base_path: &str) {
    println!("lut_ptrhash_double_empty_list_opt");

    // let cutoffs: Vec<usize> = vec![64, 128, 192, 256, 320, 384, 448, 512, 1024, 2048, 4096, 8192];
    //
    // let result_dir_path = format!("{}/speed/find_pair_datat", base_path);
    // fs::create_dir_all(&result_dir_path).expect("Failed to create directory");
    //
    // // GB_1
    // eval_all(&data_dir_path, &result_dir_path, QUERY_BESTBUY, &cutoffs);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF1, &cutoffs);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF2, &cutoffs);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF4, &cutoffs);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_GOOGLE, &cutoffs);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_NSPL, &cutoffs);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_TWITTER, &cutoffs);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_WALMART, &cutoffs);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_WIKI, &cutoffs);
    //
    // println!("Done");
}

// fn eval_all(data_dir_path: &str, result_dir_path: &str, test_data: (&str, &[(&str, &str)]), cutoffs: &Vec<usize>) {
//     // Extract input
//     let (json_filename, queries) = test_data;
//     let filename = json_filename.strip_suffix(".json").unwrap();
//     println!("JSON: {}", filename);
//
//     // Measurements
//     for cutoff in cutoffs {
//         println!("  cutoff {cutoff}");
//
//         // Ensure the result directory exists
//         fs::create_dir_all(result_dir_path).expect("Failed to create results directory");
//
//         let result_csv = format!("{}/find_pair_data.csv", result_dir_path, cutoff, filename);
//         let csv_exists = Path::new(&result_csv).exists();
//
//         let json_path = format!("{}/{}.json", data_dir_path, filename);
//
//         measure_build(&json_path, result_csv, filename, *cutoff);
//     }
// }
//
// fn measure_build(json_path: &str, cutoff_dir_path: &str, filename: &str, cutoff: usize) {
//     let build_csv = format!("{}/build.csv", cutoff_dir_path);
//     let file_exists = Path::new(&build_csv).exists();
//
//     // Ensure the directory exists
//     fs::create_dir_all(cutoff_dir_path).expect("Failed to create directory");
//
//     // Open CSV in append mode
//     let mut wtr = Writer::from_writer(
//         OpenOptions::new()
//             .create(true)
//             .append(true)
//             .open(&build_csv)
//             .expect("Failed to open build CSV"),
//     );
//
//     // Write header if the file is new
//     if !file_exists {
//         wtr.write_record(&["JSON", "BUILD_TIME_SECONDS", "SIZE_IN_BYTES"])
//             .expect("Failed to write header");
//     }
//
//     // Measure size
//     let start_heap = Region::new(HEAP_TRACKER);
//     let lut = LUT::build(json_path, cutoff).expect("Failed to build LUT");
//     let heap_bytes = heap_value(start_heap.change());
//     println!("Measured heap with cutoff={}", lut.get_cutoff());
//     drop(lut);
//
//     // Warm-up
//     for _ in 0..WARM_UP_REPETITIONS {
//         let _ = LUT::build(json_path, cutoff).expect("Warm-up build failed");
//     }
//
//     // Measure build time
//     let mut total_time = 0.0;
//     for _ in 0..BUILD_REPETITIONS {
//         let start = Instant::now();
//         let _ = LUT::build(json_path, cutoff).expect("Timed build failed");
//         total_time += start.elapsed().as_secs_f64();
//     }
//
//     let avg_time = total_time / BUILD_REPETITIONS as f64;
//     println!(" build time = {:.5}s, size = {} B", avg_time, heap_bytes);
//
//     // Write the results
//     wtr.write_record(&[filename, &format!("{:.5}", avg_time), &heap_bytes.to_string()])
//         .expect("Failed to write build record");
//
//     wtr.flush().expect("Failed to flush build CSV");
// }
