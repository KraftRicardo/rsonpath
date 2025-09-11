use crate::lookup_table::speed::query_data::*;
use crate::lookup_table::{LookUpTable, LUT, QUERY_REPETITIONS};
use crate::{
    engine::{Compiler, Engine, RsonpathEngine},
    input::OwnedBytes,
};
use csv::Writer;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;
use std::time::Instant;
use std::{fs, io::BufReader};

/// Evaluates the query speed of rq-lut so it can be compared to rq-legacy and its optimal time.
///
/// The comparison is done by measuring speed (y-axis) against query time (x-axis).
/// Results are saved as CSV files for further analysis.
///
/// # Arguments
/// * `data_dir_path` - Path to the folder holding the input JSON files.
/// * `result_dir_path` - Path to the folder where the results will be saved.
///
/// # Output
/// Data will be saved in:
///
/// ```text
/// {result_dir_path}/rq_lut_time.csv
/// ```
///
/// # CSV Structure
/// ```text
/// JSON,CUTOFF,QUERY_ID,QUERY_TEXT,QUERY_TIME_SECONDS
/// google_map_large_record_(1.1GB),0,0,$[4000].routes[*].bounds,0.00652
/// google_map_large_record_(1.1GB),0,1,$[*].routes[*].legs[*].steps[*].html_instructions,0.47458
/// ...
/// ```
///
/// # Example
/// ```bash
/// cargo run --bin lut --release -- eval-rq-lut res/json res/data/speed/local/rq_lut
/// cargo run --bin lut --release -- eval-rq-lut ricardo-jsons plot-results
/// ```
#[inline]
pub fn evaluate_rq_lut_query_speed(data_dir_path: &str, result_dir_path: &str) {
    println!("eval-rq-lut");

    // 2^40 = 1,099,511,627,776, we basically use a cutoff so high we do not trigger the skipping
    // with the lut. We want to see how slow the overall application is.
    let cutoffs = vec![
        0,
        64,
        128,
        192,
        256,
        320,
        384,
        448,
        512,
        576,
        640,
        1024,
        2048,
        4096,
        8192,
        1099511627776,
    ];

    if cfg! {feature = "track-skipping"} {
        println!("Disable tracking of skips before running because it slows down the algorithm.");
        return;
    }
    if !cfg! {feature = "empty-list-opt"} {
        println!("Turn the empty-list-opt feature for better performance!");
        return;
    }

    // Create results dir
    fs::create_dir_all(result_dir_path).expect("Failed to create directory");

    // GB_1
    // evaluate(data_dir_path, result_dir_path, QUERY_BESTBUY, &cutoffs);
    // evaluate(data_dir_path, result_dir_path, QUERY_CROSSREF1, &cutoffs);
    // evaluate(data_dir_path, result_dir_path, QUERY_CROSSREF2, &cutoffs);
    // evaluate(data_dir_path, result_dir_path, QUERY_CROSSREF4, &cutoffs);
    // evaluate(data_dir_path, result_dir_path, QUERY_GOOGLE, &cutoffs);
    // evaluate(data_dir_path, result_dir_path, QUERY_NSPL, &cutoffs);
    evaluate(data_dir_path, result_dir_path, QUERY_TWITTER, &cutoffs);
    evaluate(data_dir_path, result_dir_path, QUERY_TWITTER_SINGLE, &cutoffs);
    evaluate(data_dir_path, result_dir_path, QUERY_WALMART, &cutoffs);
    evaluate(data_dir_path, result_dir_path, QUERY_WALMART_SINGLE, &cutoffs);
    evaluate(data_dir_path, result_dir_path, QUERY_WIKI, &cutoffs);
    evaluate(data_dir_path, result_dir_path, QUERY_WIKI_SINGLE, &cutoffs);

    println!("Done");
}

/// Measure the query times of rq-lut for different cutoffs. Results will be written into a csv.
fn evaluate(data_dir_path: &str, result_dir_path: &str, query_data_csv: &str, cutoffs: &Vec<usize>) {
    let (json_path, json_name, queries) = extract_input(data_dir_path, query_data_csv);

    // Measurements
    for cutoff in cutoffs {
        measure_query(&json_path, result_dir_path, &json_name, &queries, *cutoff);
    }
}

// Measure query time of rq-lut for the given queries and cutoff on a given json.
fn measure_query(json_path: &str, result_dir_path: &str, filename: &str, queries: &[(String, String)], cutoff: usize) {
    let query_csv_path = format!("{result_dir_path}/rq_lut_time.csv");
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
        wrt.write_record([
            "JSON",
            "CUTOFF",
            "QUERY_ID",
            "QUERY_TEXT",
            "QUERY_TIME_SECONDS",
            "REPETITIONS",
        ])
        .expect("Failed to write header");
    }

    // Build LUT once and read input file into memory
    let mut lut = LUT::build(json_path, cutoff).expect("Failed to build LUT");

    let input = {
        let mut buf = vec![];
        let mut file = BufReader::new(fs::File::open(json_path).expect("Failed to open input file"));
        file.read_to_end(&mut buf).expect("Failed to read input file");
        OwnedBytes::new(buf)
    };

    for (query_id, query_text) in queries {
        let query = rsonpath_syntax::parse(query_text).expect("Failed to parse query");
        let mut engine = RsonpathEngine::compile_query(&query).expect("Failed to compile query");
        engine.add_lut(lut);

        // Warm up
        for _ in 0..QUERY_REPETITIONS {
            let _ = engine.count(&input).expect("Query execution failed");
        }

        // Measure query time
        let mut result = 0;
        let mut total_time = 0.0;

        for _ in 0..QUERY_REPETITIONS {
            let start = Instant::now();
            result = engine.count(&input).expect("Query execution failed");
            total_time += start.elapsed().as_secs_f64();
        }

        let avg_time = total_time / (QUERY_REPETITIONS as f64);
        lut = engine.take_lut().expect("Failed to retrieve LUT");

        println!(
            "  - File: {filename}, Cutoff: {cutoff}, Query {query_id}: {query_text}, Time = {avg_time:.5}s, Result = {result}",
        );

        wrt.write_record([
            filename,
            cutoff.to_string().as_str(),
            query_id.as_str(),
            query_text.as_str(),
            &format!("{avg_time:.5}"),
            QUERY_REPETITIONS.to_string().as_str(),
        ])
        .expect("Failed to write to CSV");
    }

    wrt.flush().expect("Failed to flush CSV");
    println!("Generated: {query_csv_path}");
}
