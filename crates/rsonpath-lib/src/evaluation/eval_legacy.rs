use crate::evaluation::query_data::*;
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

use crate::evaluation::track_config::QUERY_REPETITIONS;

/// Measures the execution time of `rq` (no LUT) for a given set of JSON files and queries.
///
/// This function benchmarks query execution and saves the results to a CSV file.
///
/// # Parameters
///
/// - `data_dir_path`: Path to the folder holding the input JSON files.
/// - `base_path`: Path to the folder where the results will be saved.
/// - `use_empty_list_opt`: Enables or disables the empty-list optimization (no effect here).
///
/// # Output
///
/// Results are saved in:
///
/// ```text
/// {base_path}/rq_legacy/rq_legacy_time_repetitions={QUERY_REPETITIONS}.csv
/// OR
/// {base_path}/rq_legacy_empty_list_opt_off/rq_legacy_empty_list_opt_off_time_repetitions={QUERY_REPETITIONS}.csv
/// ```
///
/// The CSV has the following structure:
///
/// ```text
/// JSON,QUERY_ID,QUERY_TEXT,QUERY_TIME_SECONDS
/// crossref1_(551MB),1,$.items[2].resource.primary.URL,0.13103
/// crossref1_(551MB),2,$.items[*].URL,0.13692
/// ...
/// ```
///
/// # Examples
/// Running with predefined JSONs and plotting results:
/// ```bash
/// cargo run --bin eval --release -- eval-legacy ../rsonpath/res/json ../rsonpath/res/data/speed/local
/// cargo run --bin eval --release -- eval-legacy ricardo-jsons plot-results
/// ```
/// Running with the empty-list optimization explicitly disabled:
/// ```bash
/// cargo run --bin eval --release -- eval-legacy-empty-list-opt-off ../rsonpath/res/json ../rsonpath/res/data/speed/local
/// cargo run --bin eval --release -- eval-legacy-empty-list-opt-off ricardo-jsons plot-results
/// ```
pub fn evaluate_rq_query_speed(data_dir_path: &str, base_path: &str, use_empty_list_opt: bool) {
    let mut result_dir_path: String;
    if use_empty_list_opt {
        println!("rq-legacy QUERY_REPETITIONS {QUERY_REPETITIONS}");
        result_dir_path = format!("{base_path}/rq_legacy");
    } else {
        print!("rq-legacy-empty-list-opt-off QUERY_REPETITIONS {QUERY_REPETITIONS}");
        result_dir_path = format!("{base_path}/rq_legacy_empty_list_opt_off");
    }

    // Abort conditions
    if use_empty_list_opt && cfg! {feature = "empty-list-opt"} != use_empty_list_opt {
        println!("empty-list-opt is currently disabled. For fair comparisons with rq_lut enable it.");
        return;
    }
    if !use_empty_list_opt && cfg! {feature = "empty-list-opt"} != use_empty_list_opt {
        println!(
            "For building the rq-legacy-empty-list-opt-off run the empty-list-opt must be disabled for this analysis."
        );
        return;
    }
    if cfg! {feature = "track-skipping"} {
        println!("Disable tracking of skips before running because it slows down the algorithm.");
        return;
    }

    // Create results dir
    fs::create_dir_all(&result_dir_path).expect("Failed to create directory");

    let use_count = false;

    // GB_1
    // eval_all(&data_dir_path, &result_dir_path, QUERY_BESTBUY, use_count);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF1, use_count);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF2, use_count);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF4, use_count);
    eval_all(&data_dir_path, &result_dir_path, QUERY_GOOGLE, use_count);
    eval_all(&data_dir_path, &result_dir_path, QUERY_NSPL, use_count);
    eval_all(&data_dir_path, &result_dir_path, QUERY_TWITTER, use_count);
    eval_all(&data_dir_path, &result_dir_path, QUERY_TWITTER_SINGLE, use_count);
    eval_all(&data_dir_path, &result_dir_path, QUERY_WALMART, use_count);
    eval_all(&data_dir_path, &result_dir_path, QUERY_WALMART_SINGLE, use_count);
    eval_all(&data_dir_path, &result_dir_path, QUERY_WIKI, use_count);
    eval_all(&data_dir_path, &result_dir_path, QUERY_WIKI_SINGLE, use_count);

    println!("Done");
}

// use_count: Flag whether you want to query the count or the node result
pub fn eval_all(data_dir_path: &str, result_dir_path: &str, query_data_csv: &str, use_count: bool) {
    let (json_path, json_name, queries) = extract_input(data_dir_path, query_data_csv);
    println!("JSON: {json_name}");

    if use_count {
        println!("Mode:Count");
        measure_query_count(&json_path, &result_dir_path, &json_name, &queries);
    } else {
        println!("Mode:Node");
        measure_query_node(&json_path, &result_dir_path, query_data_csv, &queries);
    }
}

// Measure query time
fn measure_query_count(json_path: &str, result_dir_path: &str, json_name: &str, queries: &[(String, String)]) {
    let query_csv_path = if cfg!(feature = "empty-list-opt") {
        format!("{result_dir_path}/rq_legacy_time_repetitions={QUERY_REPETITIONS}.csv")
    } else {
        format!("{result_dir_path}/rq_legacy_empty_list_opt_off_time_repetitions={QUERY_REPETITIONS}.csv")
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
        println!("  - File:{json_name}, Query{query_id}:{query_text}, Time:{avg_time}s Result:{result}, Repetitions:{QUERY_REPETITIONS}",);

        wrt.write_record(&[
            json_name,
            &query_id,
            &query_text,
            &format!("{avg_time}"),
            &format!("{QUERY_REPETITIONS}"),
        ])
        .expect("Failed to write to CSV");
    }

    wrt.flush().expect("Failed to flush CSV");
    println!("Generated: {}", query_csv_path);
}

fn measure_query_node(json_path: &str, result_dir_path: &str, json_name: &str, queries: &[(String, String)]) {
    let query_csv_path = if cfg!(feature = "empty-list-opt") {
        format!("{result_dir_path}/rq_legacy_time_node_repetitions={QUERY_REPETITIONS}.csv")
    } else {
        format!("{result_dir_path}/rq_legacy_empty_list_opt_off_time_node_repetitions={QUERY_REPETITIONS}.csv")
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
            // Node
            let mut sink = vec![];
            engine.matches(&input, &mut sink).expect("Fail @ engine matching.");
        }

        // Measure query time
        let mut result = 0;
        let mut total_time = 0.0;

        for _ in 0..QUERY_REPETITIONS {
            // INDEX
            let mut sink = vec![];
            let start = Instant::now();
            engine.matches(&input, &mut sink).expect("Fail @ engine matching.");
            total_time += start.elapsed().as_secs_f64();
            result = sink.len();
        }

        let avg_time = total_time / (QUERY_REPETITIONS as f64);
        println!("  - File:{json_name}, Query{query_id}:{query_text}, Time:{avg_time}s Result:{result}, Repetitions:{QUERY_REPETITIONS}",);

        wrt.write_record(&[
            json_name,
            &query_id,
            &query_text,
            &format!("{avg_time}"),
            &format!("{QUERY_REPETITIONS}"),
        ])
        .expect("Failed to write to CSV");
    }

    wrt.flush().expect("Failed to flush CSV");
    println!("Generated: {}", query_csv_path);
}
