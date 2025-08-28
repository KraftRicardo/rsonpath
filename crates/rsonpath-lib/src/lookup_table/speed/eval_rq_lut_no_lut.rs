use crate::lookup_table::speed::query_data::*;
use crate::lookup_table::SkipMode::OFF;
use crate::lookup_table::{QUERY_REPETITIONS, SKIP_MODE, TRACK_SKIPPING_ON};
use crate::{
    engine::{Compiler, Engine, RsonpathEngine},
    input::OwnedBytes,
};
use csv::Writer;
use std::fs::OpenOptions;
use std::path::Path;
use std::time::Instant;
use std::{
    fs,
    io::{BufReader, Read},
};

/// Run with: cargo run --bin lut --release -- eval-rq-lut-no-lut res/json res/data/speed/local/rq_lut_no_lut
///
/// Run rq-lut without using a LUT but just using the non-lut version of rq. This is to see whether
/// there is a measurable speed change in the changes of the base code.
/// Output is one csv per json file. Structure e.g. :
///     QUERY_ID,QUERY_TEXT,QUERY_TIME_SECONDS
///     1,$..freeShipping,0.08720
///     2,$.products[*].videoChapters,0.79716
///     ...
pub fn run(data_dir_path: &str, result_dir_path: &str) {
    println!("rq-lut-no-lut");

    if TRACK_SKIPPING_ON || SKIP_MODE != OFF {
        println!("Disable tracking of skips before running because it slows down the algorithm.");
        return;
    }
    if !(cfg! {feature = "empty-list-opt"}) {
        println!("empty-list-opt not set, aborting");
        return;
    }

    // Create results dir
    fs::create_dir_all(&result_dir_path).expect("Failed to create directory");

    // GB_1
    eval_all(&data_dir_path, &result_dir_path, QUERY_BESTBUY);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF1);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF2);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF4);
    eval_all(&data_dir_path, &result_dir_path, QUERY_GOOGLE);
    eval_all(&data_dir_path, &result_dir_path, QUERY_NSPL);
    eval_all(&data_dir_path, &result_dir_path, QUERY_TWITTER);
    eval_all(&data_dir_path, &result_dir_path, QUERY_WALMART);
    eval_all(&data_dir_path, &result_dir_path, QUERY_WIKI);

    println!("Done");
}

fn eval_all(data_dir_path: &str, result_dir_path: &str, query_data_csv: &str) {
    let (json_path, _, queries) = extract_input(data_dir_path, query_data_csv);

    measure_query(&json_path, &result_dir_path, query_data_csv, queries);
}

// Measure query time
fn measure_query(json_path: &str, result_dir_path: &str, query_data_csv: &str, queries: Vec<(String, String)>) {
    let query_csv_path = format!("{}/rq_lut_no_lut_time.csv", result_dir_path);
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

    // Read legacy input once
    let input = {
        let mut buf = vec![];
        let mut file = BufReader::new(fs::File::open(json_path).expect("Failed to open input file"));
        file.read_to_end(&mut buf).expect("Failed to read input file");
        OwnedBytes::new(buf)
    };

    for (query_id, query_text) in queries {
        let query = rsonpath_syntax::parse(&query_text).expect("Failed to parse query");
        let engine = RsonpathEngine::compile_query(&query).expect("Failed to compile query");

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
            "  - File: {}, Query {}: {}, Time = {:.5}s, Result = {}",
            query_data_csv, query_id, query_text, avg_time, result
        );

        wrt.write_record(&[
            query_data_csv.to_string(),
            query_id,
            query_text,
            format!("{:.5}", avg_time),
            QUERY_REPETITIONS.to_string(),
        ])
        .expect("Failed to write to CSV");
    }

    wrt.flush().expect("Failed to flush CSV");
    println!("Generated: {query_csv_path}")
}
