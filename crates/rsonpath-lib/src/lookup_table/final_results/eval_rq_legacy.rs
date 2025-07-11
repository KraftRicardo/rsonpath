use crate::lookup_table::performance::lut_query_data::{
    QUERY_BESTBUY, QUERY_CROSSREF1, QUERY_CROSSREF2, QUERY_CROSSREF4, QUERY_GOOGLE, QUERY_NSPL, QUERY_TWITTER,
    QUERY_WALMART, QUERY_WIKI,
};
use crate::lookup_table::performance::lut_skip_evaluation::SkipMode::OFF;
use crate::lookup_table::{SKIP_MODE, TRACK_SKIPPING_ON};
use csv::Writer;
use rsonpath_lib_ref::engine::Compiler;
use rsonpath_lib_ref::engine::{Compiler as CompilerLegacy, Engine as EngineLegacy};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;
use std::{fs, io::BufReader};

pub const QUERY_REPETITIONS: usize = 1;
pub const WARM_UP_QUERY_REPETITIONS: usize = 1;

// Measures the time taken for rq-legacy for given JSON+Queries.
//
// Run with: cargo run --bin lut --release -- eval-rq-legacy .a_test_data .a_final_results
// Run with: cargo run --bin lut --release -- eval-rq-legacy ricardo-jsons final-results-2
//
// "data_dir_path" path to the folder holding the input JSON files.
// "base_path" path to the folder where the results will be saved
//
// Data will be saved in "{base_path}/speed/rq-legacy/legacy_time.csv"
// Example structure of the csv:
//  JSON,QUERY_ID,QUERY_TEXT,QUERY_TIME_SECONDS
//  crossref1_(551MB),1,$.items[2].resource.primary.URL,0.13103
//  crossref1_(551MB),2,$.items[*].URL,0.13692
//  ...
pub fn evaluate(data_dir_path: &str, base_path: &str) {
    println!("rq-legacy");

    if (TRACK_SKIPPING_ON || SKIP_MODE != OFF) {
        println!("Disable tracking of skips before running because it slows down the algorithm.");
        return;
    }

    // Create results dir
    let result_dir_path = format!("{}/speed/rq-legacy", base_path);
    fs::create_dir_all(&result_dir_path).expect("Failed to create directory");

    // GB_1
    // eval_all(&data_dir_path, &result_dir_path, QUERY_BESTBUY);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF1);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF2);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF4);
    eval_all(&data_dir_path, &result_dir_path, QUERY_GOOGLE);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_NSPL);
    eval_all(&data_dir_path, &result_dir_path, QUERY_TWITTER);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_WALMART);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_WIKI);

    println!("Done");
}

fn eval_all(data_dir_path: &str, result_dir_path: &str, test_data: (&str, &[(&str, &str)])) {
    // Extract input
    let (json_filename, queries) = test_data;
    let filename = json_filename.strip_suffix(".json").unwrap();
    println!("JSON: {}", filename);

    // All necessary paths to CSV and PNG
    let json_path = format!("{}/{}.json", data_dir_path, filename);

    measure_query(&json_path, &result_dir_path, filename, queries);
}

// Measure query time
fn measure_query(json_path: &str, result_dir_path: &str, filename: &str, queries: &[(&str, &str)]) {
    let query_csv_path = format!("{}/rq-legacy_time.csv", result_dir_path);
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
        wrt.write_record(&["JSON", "QUERY_ID", "QUERY_TEXT", "QUERY_TIME_SECONDS"])
            .expect("Failed to write header");
    }

    // Read legacy input once
    let legacy_input = {
        let mut file = BufReader::new(fs::File::open(json_path).expect("Fail @ open File"));
        let mut buf = vec![];
        file.read_to_end(&mut buf).expect("Fail @ file read");
        rsonpath_lib_ref::input::OwnedBytes::new(buf)
    };

    for &(query_id, query_text) in queries {
        let legacy_query = syntax_ref::parse(query_text).expect("Fail @ parse query");
        let legacy_engine = rsonpath_lib_ref::engine::RsonpathEngine::compile_query(&legacy_query).expect("Fail query");

        // Warm up
        for _ in 0..WARM_UP_QUERY_REPETITIONS {
            let _ = legacy_engine.count(&legacy_input).expect("Failed count");
        }

        // Measure query time
        let mut result = 0;
        let mut total_time = 0.0;

        for _ in 0..QUERY_REPETITIONS {
            let start = Instant::now();
            result = legacy_engine.count(&legacy_input).expect("Fail count");
            total_time += start.elapsed().as_secs_f64();
        }

        let avg_time = total_time / (QUERY_REPETITIONS as f64);
        println!(
            "  - File: {}, Query {}: {}, Time = {:.5}s Result = {}",
            filename, query_id, query_text, avg_time, result
        );

        wrt.write_record(&[filename, query_id, query_text, &format!("{:.5}", avg_time)])
            .expect("Failed to write to CSV");
    }

    wrt.flush().expect("Failed to flush CSV");
    println!("Data written to {}", query_csv_path);
}
