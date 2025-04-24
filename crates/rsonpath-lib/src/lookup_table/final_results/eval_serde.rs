use crate::result::MatchCount;
use crate::{
    engine::{Compiler, Engine, RsonpathEngine},
    input::OwnedBytes,
};
use csv::Writer;
use serde_json::{json, Value};
// use serde_json_path::JsonPath;
use crate::lookup_table::performance::distance_cutoff_evaluation;
use crate::lookup_table::performance::lut_evaluation::HEAP_TRACKER;
use crate::lookup_table::LUT;
use distance_cutoff_evaluation::heap_value;
use ptr_hash::util;
use stats_alloc::Region;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::Instant;
use std::{
    fs, io,
    io::{BufReader, Read},
};

pub const QUERY_REPETITIONS: usize = 10;
pub const BUILD_REPETITIONS: usize = 10;

pub fn evaluate(data_dir_path: &str, base_path: &str) {
    todo!()
}

fn eval_all(data_dir_path: &str, result_dir_path: &str, test_data: (&str, &[(&str, &str)])) {
    // Extract input
    let (json_filename, queries) = test_data;
    let filename = json_filename.strip_suffix(".json").unwrap();
    println!("JSON: {}", filename);

    // All necessary paths to CSV and PNG
    let json_path = format!("{}/{}.json", data_dir_path, filename);
    let serde_dir_path = format!("{}/serde", result_dir_path,);
    fs::create_dir_all(&serde_dir_path).expect("Failed to create directory");

    // measure_build(json_path, serde_dir_path, filename);
}

// fn eval_serde(json_value: &Value, query_text: &str) -> (f64, u64) {
//     // Parse the JSONPath query
//     let path = JsonPath::parse(query_text).expect("Could not parse query JSON");
//
//     let mut query_time_total = 0.0;
//     let mut result = 0;
//
//     for _ in 0..QUERY_REPETITIONS {
//         let start_query = std::time::Instant::now();
//         let nodes = path.query(&json_value);
//         query_time_total += start_query.elapsed().as_secs_f64();
//         result = nodes.len() as u64;
//     }
//
//     let query_time_average = query_time_total / (QUERY_REPETITIONS as f64);
//     println!("  - SERDE: Time = {:.5}s Result = {}", query_time_average, result);
//
//     (query_time_average, result)
// }
//
// fn measure_build(json_path: &str, serde_dir_path: &str, filename: &str) {
//     let build_csv = format!("{}/build.csv", serde_dir_path);
//     let file_exists = Path::new(&build_csv).exists();
//
//     // Ensure the directory exists
//     fs::create_dir_all(serde_dir_path).expect("Failed to create directory");
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
//
//     let file = fs::File::open(json_path).expect("Failed to open file");
//     let reader = BufReader::new(file);
//     let json_value: Value = serde_json::from_reader(reader).expect("Failed to parse JSON");
//
//     let heap_bytes = heap_value(start_heap.change());
//
//     drop(json_value);
//
//     // Warm-up
//     for _ in 0..distance_cutoff_evaluation::BUILD_REPETITIONS {
//         let file = fs::File::open(json_path).expect("Failed to open file");
//         let reader = BufReader::new(file);
//         let _ = serde_json::from_reader(reader).expect("Failed to parse JSON");
//     }
//
//     // Measure build time
//     let mut total_time = 0.0;
//     for _ in 0..distance_cutoff_evaluation::BUILD_REPETITIONS {
//         let start = Instant::now();
//
//         let file = fs::File::open(json_path).expect("Failed to open file");
//         let reader = BufReader::new(file);
//         let _ = serde_json::from_reader(reader).expect("Failed to parse JSON");
//
//         total_time += start.elapsed().as_secs_f64();
//     }
//
//     let avg_time = total_time / distance_cutoff_evaluation::BUILD_REPETITIONS as f64;
//     println!(" build time = {:.5}s, size = {} B", avg_time, heap_bytes);
//
//     // Write the results
//     wtr.write_record(&[filename, &format!("{:.5}", avg_time), &heap_bytes.to_string()])
//         .expect("Failed to write build record");
//
//     wtr.flush().expect("Failed to flush build CSV");
// }

pub fn plot() {
    todo!()
}
