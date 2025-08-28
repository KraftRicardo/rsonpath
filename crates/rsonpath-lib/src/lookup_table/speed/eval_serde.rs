use crate::lookup_table::speed::eval_distance_cutoff::heap_value;
use crate::lookup_table::speed::eval_lut_construction::HEAP_TRACKER;
use crate::lookup_table::speed::query_data::*;
use crate::lookup_table::{BUILD_REPETITIONS, QUERY_REPETITIONS};
use csv::Writer;
use serde_json::Value;
use serde_json_path::JsonPath;
use stats_alloc::Region;
use std::fs::OpenOptions;
use std::path::Path;
use std::time::Instant;
use std::{fs, io::BufReader};

/// Measures build time and query for given json and their queries using the serde crate.
/// Output will be saved in build.csv and serde_time.csv.
/// build.csv e.g.:
///     JSON,BUILD_TIME_SECONDS,SIZE_IN_BYTES
///     crossref1_(551MB),4.83730,3030361082
///     bestbuy_large_record_(1GB),11.80644,5177256301
///     ...
/// serde_time.csv e.g.:
///     JSON,QUERY_ID,QUERY_TEXT,QUERY_TIME_SECONDS,REPETITIONS
///     bestbuy_large_record_(1GB),1,$..freeShipping,1.77164,1
///     bestbuy_large_record_(1GB),2,$.products[*].videoChapters,0.13674,1
///     bestbuy_large_record_(1GB),3,$.products[*].additionalFeatures[*],0.08287,1
///     ...
/// Run with: cargo run --bin lut --release -- eval-serde res/json res/data/speed/local/serde
/// Run with: cargo run --bin lut --release -- eval-serde ricardo-jsons final-results-2
pub fn run(data_dir_path: &str, result_dir_path: &str) {
    println!("eval-serde");

    fs::create_dir_all(&result_dir_path).expect("Failed to create directory");

    // GB_1
    eval_all(&data_dir_path, result_dir_path, QUERY_BESTBUY);
    // eval_all(&data_dir_path, result_dir_path, QUERY_CROSSREF1);
    // eval_all(&data_dir_path, result_dir_path, QUERY_CROSSREF2);
    // eval_all(&data_dir_path, result_dir_path, QUERY_CROSSREF4);
    // eval_all(&data_dir_path, result_dir_path, QUERY_GOOGLE);
    // eval_all(&data_dir_path, result_dir_path, QUERY_NSPL);
    // eval_all(&data_dir_path, result_dir_path, QUERY_TWITTER);
    // eval_all(&data_dir_path, result_dir_path, QUERY_WALMART);
    // eval_all(&data_dir_path, result_dir_path, QUERY_WIKI);

    println!("Done");
}

fn eval_all(data_dir_path: &str, result_dir_path: &str, query_data_csv: &str) {
    let (json_path, _, queries) = extract_input(data_dir_path, query_data_csv);

    measure_build(&json_path, result_dir_path, query_data_csv);
    measure_query(&json_path, result_dir_path, query_data_csv, queries);
}

// Measure query time
fn measure_query(json_path: &str, result_dir_path: &str, query_data_csv: &str, queries: Vec<(String, String)>) {
    let query_csv_path = format!("{result_dir_path}/serde_time.csv");
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
        wrt.write_record(["JSON", "QUERY_ID", "QUERY_TEXT", "QUERY_TIME_SECONDS", "REPETITIONS"])
            .expect("Failed to write header");
    }

    // Build serde tree once
    let file = fs::File::open(json_path).expect("Failed to open file");
    let reader = BufReader::new(file);
    let json_value: Value = serde_json::from_reader(reader).expect("Failed to parse JSON");

    for (query_id, query_text) in queries {
        // Parse the JSONPath query
        let path = JsonPath::parse(&query_text).expect("Could not parse query JSON");

        // Warm up
        for _ in 0..QUERY_REPETITIONS {
            let _ = path.query(&json_value);
        }

        // Measure query time
        let mut result = 0;
        let mut total_time = 0.0;

        for _ in 0..QUERY_REPETITIONS {
            let start = Instant::now();
            let nodes = path.query(&json_value);
            total_time += start.elapsed().as_secs_f64();
            result = nodes.len() as u64;
        }

        let avg_time = total_time / (QUERY_REPETITIONS as f64);
        println!(
            "  - query = {}, query_text={}, time = {:.5}s, result = {}",
            query_id, query_text, avg_time, result
        );

        wrt.write_record([
            query_data_csv.to_string(),
            query_id,
            query_text,
            format!("{:.5}", avg_time),
            QUERY_REPETITIONS.to_string(),
        ])
        .expect("Failed to write to CSV");
    }

    wrt.flush().expect("Failed to flush CSV");
    println!("Generated: {query_csv_path}");
}

/// Measure build time
fn measure_build(json_path: &str, serde_dir_path: &str, query_data_csv: &str) {
    let build_csv_path = format!("{}/build.csv", serde_dir_path);
    let file_exists = Path::new(&build_csv_path).exists();

    // Open CSV in append mode
    let mut wtr = Writer::from_writer(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&build_csv_path)
            .expect("Failed to open build CSV"),
    );

    // Write header if the file is new
    if !file_exists {
        print!("File did not exist");
        wtr.write_record(["JSON", "BUILD_TIME_SECONDS", "SIZE_IN_BYTES"])
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
    for _ in 0..BUILD_REPETITIONS {
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
    wtr.write_record([query_data_csv, &format!("{:.5}", avg_time), &heap_bytes.to_string()])
        .expect("Failed to write build record");

    wtr.flush().expect("Failed to flush build CSV");
    println!("Generated: {build_csv_path}");
}
