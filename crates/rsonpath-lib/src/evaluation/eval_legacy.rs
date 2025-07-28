use crate::evaluation::lut_query_data::*;
use crate::{
    engine::{Compiler, Engine, RsonpathEngine},
    input::OwnedBytes,
};
use csv::Writer;
use rsonpath_syntax::parse;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;
use std::{fs, io::BufReader};

use crate::evaluation::track_config::{QUERY_REPETITIONS, TRACK_SKIPPING_ON};

// Measures the time taken for rq for given JSON+Queries. "empty-list-opt" has here no effect.
//
// Run with: cargo run --bin eval --release -- eval-legacy ../rsonpath/res/json ../rsonpath/res/data/speed/local
// Run with: cargo run --bin eval --release -- eval-legacy ricardo-jsons plot-results
// OR WITH:
// Run with: cargo run --bin eval --release -- eval-legacy-empty-list-opt-off ../rsonpath/res/json ../rsonpath/res/data/speed/local
// Run with: cargo run --bin eval --release -- eval-legacy-empty-list-opt-off ricardo-jsons plot-results
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
pub fn run(data_dir_path: &str, base_path: &str, use_empty_list_opt: bool) {
    let mut result_dir_path: String;
    if use_empty_list_opt {
        println!("rq-legacy QUERY_REPETITIONS {QUERY_REPETITIONS}");
        result_dir_path = format!("{}/rq_legacy", base_path);
    } else {
        print!("rq-legacy-empty-list-opt-off QUERY_REPETITIONS {QUERY_REPETITIONS}");
        result_dir_path = format!("{}/rq_legacy_empty_list_opt_off", base_path);
    }

    // Abort conditions
    if use_empty_list_opt && cfg! {feature = "empty-list-opt"} != use_empty_list_opt {
        println!("empty-list-opt is currently disabled. For fair comparisons with rsonpath-lut enable it.");
        return;
    }
    if !use_empty_list_opt && cfg! {feature = "empty-list-opt"} != use_empty_list_opt {
        println!(
            "For building the rq-legacy-empty-list-opt-off run the empty-list-opt must be disabled for this analysis."
        );
        return;
    }
    if TRACK_SKIPPING_ON {
        println!("Disable tracking of skips before running because it slows down the algorithm.");
        return;
    }

    // Create results dir
    fs::create_dir_all(&result_dir_path).expect("Failed to create directory");

    // GB_1
    eval_all(&data_dir_path, &result_dir_path, QUERY_BESTBUY);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF1);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF2);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF4);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_GOOGLE);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_NSPL);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_TWITTER);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_TWITTER_SCALED);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_WALMART);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_WALMART_SCALED);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_WIKI);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_WIKI_SCALED);

    println!("Done");
}

pub fn eval_all(data_dir_path: &str, result_dir_path: &str, query_data: &str) {
    // Extract input
    let (json_filename, queries) = read_queries(query_data);
    let filename = json_filename.strip_suffix(".json").unwrap();
    println!("JSON: {}", filename);

    // All necessary paths to CSV and PNG
    let json_path = format!("{}/{}.json", data_dir_path, filename);

    measure_query(&json_path, &result_dir_path, filename, &queries);
}

// Measure query time
fn measure_query(json_path: &str, result_dir_path: &str, filename: &str, queries: &[(String, String)]) {
    let query_csv_path = if cfg!(feature = "empty-list-opt") {
        format!("{}/rq_legacy_time.csv", result_dir_path)
    } else {
        format!("{}/rq_legacy_empty_list_opt_off_time.csv", result_dir_path)
    };
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
        wrt.write_record(&["JSON", "QUERY_ID", "QUERY_TEXT", "QUERY_TIME_SECONDS", "REPETITIONS"])
            .expect("Failed to write header");
    }

    // Read input once
    let input = {
        let mut file = BufReader::new(fs::File::open(json_path).expect("Fail @ open File"));
        let mut buf = vec![];
        file.read_to_end(&mut buf).expect("Fail @ file read");
        OwnedBytes::new(buf)
    };

    for (query_id, query_text) in queries {
        let query = parse(&query_text).expect("Fail @ parse query");
        let engine = RsonpathEngine::compile_query(&query).expect("Fail query");

        // Warm up
        for _ in 0..QUERY_REPETITIONS {
            let _ = engine.count(&input).expect("Failed count");
        }

        // Measure query time
        let mut result = 0;
        let mut total_time = 0.0;

        for _ in 0..QUERY_REPETITIONS {
            let start = Instant::now();
            result = engine.count(&input).expect("Fail count");
            total_time += start.elapsed().as_secs_f64();
        }

        let avg_time = total_time / (QUERY_REPETITIONS as f64);
        println!(
            "  - File: {}, Query {}: {}, Time = {:.5}s Result = {}",
            filename, query_id, query_text, avg_time, result
        );

        wrt.write_record(&[
            filename,
            &query_id,
            &query_text,
            &format!("{:.5}", avg_time),
            &format!("{}", QUERY_REPETITIONS),
        ])
        .expect("Failed to write to CSV");
    }

    wrt.flush().expect("Failed to flush CSV");
    println!("Generated: {}", query_csv_path);
}
