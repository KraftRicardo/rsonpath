use csv::Writer;
use std::io::Write;
use std::process::Command;
use std::{
    fs, io,
    io::{BufReader, Read},
};

use super::lut_query_data;
use crate::lookup_table::performance::lut_query_data::QUERY_BESTBUY;
use crate::lookup_table::{util_path, LookUpTable, LUT};
use crate::result::MatchCount;
use crate::{
    engine::{Compiler, Engine, RsonpathEngine},
    input::OwnedBytes,
};
use serde_json::Value;
use std::path::Path;

const QUERY_REPETITIONS: usize = 10;
const RESULTS_PATH: &str = "../rsonpath-default/.b_lut_tests/eval_rq_vs_rq_lut";
const OPT_TEXT: &str = "opt";

// Run with: cargo run --bin lut --release -- empty
pub fn evaluate() {
    warm_up_cpu();

    let cutoff = 128;
    println!("Using cutoff: {}", cutoff);

    // GB_1
    eval_lut(QUERY_BESTBUY, cutoff);
    // eval_lut(QUERY_CROSSREF1);
    // eval_lut(QUERY_CROSSREF2);
    // eval_lut(QUERY_CROSSREF4);
    // eval_lut(QUERY_GOOGLE);
    // eval_lut(QUERY_NSPL);
    // eval_lut(QUERY_TWITTER);
    // eval_lut(QUERY_WALMART);
    // eval_lut(QUERY_WIKI);
}

// Run with: cargo run --bin lut --release -- empty-plot
pub fn plot() {
    // GB_1
    plot_all(QUERY_BESTBUY);
    // plot_all(QUERY_CROSSREF1);
    // plot_all(QUERY_CROSSREF2);
    // plot_all(QUERY_CROSSREF4);
    // plot_all(QUERY_GOOGLE);
    // plot_all(QUERY_NSPL);
    // plot_all(QUERY_TWITTER);
    // plot_all(QUERY_WALMART);
    // plot_all(QUERY_WIKI);
}

fn plot_all(test_data: (&str, &[(&str, &str)])) {
    todo!()
}

fn eval_lut(test_data: (&str, &[(&str, &str)]), cutoff: usize) {
    let mut optimization_on = cfg! {feature = "empty-list-opt"};

    // Extract input
    let (json_path, queries) = test_data;
    let filename = util_path::extract_filename(json_path);
    println!("JSON: {}", json_path);

    // All necessary paths to CSV and PNG
    let mut result_csv_path: String = "".to_string();
    if optimization_on {
        println!("Optimization is set!");
        result_csv_path = format!("{}/{}_lut_{}.csv", RESULTS_PATH, filename, OPT_TEXT);
    } else {
        println!("Optimization is NOT set!");
        result_csv_path = format!("{}/{}.csv", RESULTS_PATH, filename);
    }
    println!("Saving data at: {}", result_csv_path);

    // Write headers for the CSV files
    let mut wtr = Writer::from_path(&result_csv_path).expect("Could not open query CSV");
    wtr.write_record(&["QUERY_ID", "TIME_IN_SECONDS", "RESULT"]).unwrap();

    let mut lut = LUT::build(json_path, cutoff).expect("Fail @ build lut");

    // Measurements
    for (query_id, query_text) in queries {
        println!("Query ID: {} = {}", query_id, query_text);

        let query = rsonpath_syntax::parse(query_text).expect("Fail @ parse query");
        let mut engine = RsonpathEngine::compile_query(&query).expect("Fail @ compile query");
        engine.add_lut(lut);

        let mut query_time_total = 0.0;
        let mut result = 0;
        for _ in 0..QUERY_REPETITIONS {
            let input = {
                let mut file = BufReader::new(fs::File::open(json_path).expect("Fail @ open File"));
                let mut buf = vec![];
                file.read_to_end(&mut buf).expect("Fail @ file read");
                OwnedBytes::new(buf)
            };

            let start_query = std::time::Instant::now();
            result = engine.count(&input).expect("Failed to run query normally");
            query_time_total += start_query.elapsed().as_secs_f64();
        }
        let query_time_average = query_time_total / (QUERY_REPETITIONS as f64);
        lut = engine.take_lut().expect("Fail at taking LUT back");
        println!("  - LUT: Time = {:.5}s Result = {}", query_time_average, result);

        let time_text = format!("{:.5}", query_time_average);
        let results_text = format!("{}", result);
        wtr.write_record(&[query_id, time_text.as_str(), results_text.as_str()])
            .unwrap();
    }
    wtr.flush().unwrap();
}

fn plot_with_python(json_path: &str, rq_csv_path: &str, rq_opt_csv_path: &str) {
    todo!()
}

pub fn warm_up_cpu() {
    println!("Warming up CPU...");

    let json_path = ".a_lut_tests/test_data/MB_100/twitter_short_(80MB).json";
    let query_text = "$[*].geo";

    let query = rsonpath_syntax::parse(query_text).expect("Fail @ parse query");
    let engine = RsonpathEngine::compile_query(&query).expect("Fail @ compile query");

    let warmup_repetitions = 200;
    for _ in 0..warmup_repetitions {
        let input = {
            let mut file = BufReader::new(fs::File::open(json_path).expect("Fail @ open File"));
            let mut buf = vec![];
            file.read_to_end(&mut buf).expect("Fail @ file read");
            OwnedBytes::new(buf)
        };

        let _ = engine.count(&input).expect("Failed to run query normally");
    }

    println!("CPU warm-up complete.");
}
