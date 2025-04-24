use crate::lookup_table::performance::distance_cutoff_evaluation;
use crate::lookup_table::performance::lut_evaluation::HEAP_TRACKER;
use crate::lookup_table::performance::lut_query_data::{QUERY_BESTBUY, QUERY_CROSSREF1};
use csv::Writer;
use distance_cutoff_evaluation::heap_value;
use serde_json::Value;
use serde_json_path::JsonPath;
use stats_alloc::Region;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use std::{fs, io::BufReader};

pub const QUERY_REPETITIONS: usize = 1;
pub const BUILD_REPETITIONS: usize = 1;
pub const WARM_UP_REPETITIONS: usize = 1;

// Run with: cargo run --bin lut --release -- eval-serde .a_test_data .a_final_results
pub fn evaluate(data_dir_path: &str, base_path: &str) {
    // Create rsults dir
    let result_dir_path = format!("{}/speed/serde", base_path);
    fs::create_dir_all(&result_dir_path).expect("Failed to create directory");

    // GB_1
    eval_all(&data_dir_path, &result_dir_path, QUERY_BESTBUY);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF1);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF2);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF4);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_GOOGLE);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_NSPL);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_TWITTER);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_WALMART);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_WIKI);
}

pub fn plot() {
    todo!()
}

fn eval_all(data_dir_path: &str, result_dir_path: &str, test_data: (&str, &[(&str, &str)])) {
    // Extract input
    let (json_filename, queries) = test_data;
    let filename = json_filename.strip_suffix(".json").unwrap();
    println!("JSON: {}", filename);

    // All necessary paths to CSV and PNG
    let json_path = format!("{}/{}.json", data_dir_path, filename);

    measure_build(&json_path, result_dir_path, filename);
    measure_query(&json_path, &result_dir_path, filename, queries);
}

fn eval_serde(json_value: &Value, query_text: &str) -> (f64, u64) {
    // Parse the JSONPath query
    let path = JsonPath::parse(query_text).expect("Could not parse query JSON");

    let mut query_time_total = 0.0;
    let mut result = 0;

    for _ in 0..QUERY_REPETITIONS {
        let start_query = std::time::Instant::now();
        let nodes = path.query(&json_value);
        query_time_total += start_query.elapsed().as_secs_f64();
        result = nodes.len() as u64;
    }

    let query_time_average = query_time_total / (QUERY_REPETITIONS as f64);
    println!("  - SERDE: Time = {:.5}s Result = {}", query_time_average, result);

    (query_time_average, result)
}

// Measure query time
fn measure_query(json_path: &str, result_dir_path: &str, filename: &str, queries: &[(&str, &str)]) {
    let query_csv_path = format!("{}/{}.csv", result_dir_path, filename);
    let csv_exists = Path::new(&query_csv_path).exists();

    // Open CSV in append mode
    let mut wrt = Writer::from_writer(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&query_csv_path)
            .expect("Failed to open query CSV"),
    );

    // Write header if the file is new
    if !csv_exists {
        wrt.write_record(&["QUERY_ID", "QUERY_TEXT", "QUERY_TIME_SECONDS"])
            .expect("Failed to write header");
    }

    // Build serde tree once
    let file = fs::File::open(json_path).expect("Failed to open file");
    let reader = BufReader::new(file);
    let json_value: Value = serde_json::from_reader(reader).expect("Failed to parse JSON");

    for &(query_id, query_text) in queries {
        // Parse the JSONPath query
        let path = JsonPath::parse(query_text).expect("Could not parse query JSON");

        // Warm up
        for _ in 0..WARM_UP_REPETITIONS {
            let _ = path.query(&json_value);
        }

        // Measure query time
        let mut result = 0;
        let mut total_time = 0.0;

        for _ in 0..crate::lookup_table::performance::distance_cutoff_evaluation::QUERY_REPETITIONS {
            let start = Instant::now();
            let nodes = path.query(&json_value);
            total_time += start.elapsed().as_secs_f64();
            result = nodes.len() as u64;
        }

        let avg_time = total_time / (QUERY_REPETITIONS as f64);
        println!("  - SERDE: Time = {:.5}s Result = {}", avg_time, result);

        wrt.write_record(&[query_id, query_text, &format!("{:.5}", avg_time)])
            .expect("Failed to write to CSV");
    }

    wrt.flush().expect("Failed to flush CSV");
}

fn measure_build(json_path: &str, serde_dir_path: &str, filename: &str) {
    let build_csv = format!("{}/build.csv", serde_dir_path);
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
        print!("File did not exist");
        wtr.write_record(&["JSON", "BUILD_TIME_SECONDS", "SIZE_IN_BYTES"])
            .expect("Failed to write header");
        wtr.flush().expect("Failed to flush build CSV");
    }

    // Measure size
    let start_heap = Region::new(HEAP_TRACKER);

    let file = fs::File::open(json_path).expect("Failed to open file");
    let reader = BufReader::new(file);
    let json_value: Value = serde_json::from_reader(reader).expect("Failed to parse JSON");

    let heap_bytes = heap_value(start_heap.change());
    drop(json_value);

    // Warm-up
    for _ in 0..WARM_UP_REPETITIONS {
        let file = fs::File::open(json_path).expect("Failed to open file");
        let reader = BufReader::new(file);
        let _: Value = serde_json::from_reader(reader).expect("Failed to parse JSON");
    }

    // Measure build time
    let mut total_time = 0.0;
    for _ in 0..BUILD_REPETITIONS {
        let start = Instant::now();

        let file = fs::File::open(json_path).expect("Failed to open file");
        let reader = BufReader::new(file);
        let _: Value = serde_json::from_reader(reader).expect("Failed to parse JSON");

        total_time += start.elapsed().as_secs_f64();
    }

    let avg_time = total_time / BUILD_REPETITIONS as f64;
    println!(" build time = {:.5}s, size = {} B", avg_time, heap_bytes);

    // Write the results
    wtr.write_record(&[filename, &format!("{:.5}", avg_time), &heap_bytes.to_string()])
        .expect("Failed to write build record");

    wtr.flush().expect("Failed to flush build CSV");
}
