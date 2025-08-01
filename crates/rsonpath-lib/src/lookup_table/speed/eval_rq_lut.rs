use crate::lookup_table::speed::lut_query_data::*;
use crate::lookup_table::speed::lut_skip_evaluation::SkipMode::OFF;
use crate::lookup_table::{LookUpTable, LUT, QUERY_REPETITIONS, SKIP_MODE, TRACK_SKIPPING_ON};
use crate::{
    engine::{Compiler, Engine, RsonpathEngine},
    input::OwnedBytes,
};
use csv::Writer;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;
use std::{fs, io::BufReader};

// This was written in order to compare the speed (y-axis) of rq-lut vs. rq-legacy vs. optimal time
// per query (x-axis).
//
// Run with: cargo run --bin lut --release -- eval-rq-lut res/json res/data/speed/local/rq_lut
// Run with: cargo run --bin lut --release -- eval-rq-lut ricardo-jsons plot-results
//
// "data_dir_path" path to the folder holding the input JSON files.
// "result_dir_path" path to the folder where the results will be saved
//
// Data will be saved in "{result_dir_path}/rq_lut_time.csv"
// Example structure of the csv:
//  JSON,CUTOFF,QUERY_ID,QUERY_TEXT,QUERY_TIME_SECONDS
//  google_map_large_record_(1.1GB),0,0,$[4000].routes[*].bounds,0.00652
//  google_map_large_record_(1.1GB),0,1,$[*].routes[*].legs[*].steps[*].html_instructions,0.47458
//  ...
pub fn run(data_dir_path: &str, result_dir_path: &str) {
    println!("eval-rq-lut");

    // 2^40 = 1,099,511,627,776, we basically use a cutoff so high we do not trigger the skipping
    // with the lut. We want to see how slow the overall application is.
    let cutoffs = vec![0, 64, 128, 512, 1024, 2048, 4096, 8192, 1099511627776];
    // let cutoffs = vec![0, 64];

    if TRACK_SKIPPING_ON || SKIP_MODE != OFF {
        println!("Disable tracking of skips before running because it slows down the algorithm.");
        return;
    }
    if !cfg! {feature = "empty-list-opt"} {
        println!("Turn the empty-list-opt feature for better performance!");
        return;
    }

    // Create results dir
    fs::create_dir_all(&result_dir_path).expect("Failed to create directory");

    // GB_1
    eval_all(&data_dir_path, &result_dir_path, QUERY_BESTBUY, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF1, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF2, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF4, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_GOOGLE, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_NSPL, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_TWITTER, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_TWITTER_SCALED, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_WALMART, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_WALMART_SCALED, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_WIKI, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_WIKI_SCALED, &cutoffs);

    println!("Done");
}

fn eval_all(data_dir_path: &str, result_dir_path: &str, query_data: &str, cutoffs: &Vec<usize>) {
    // Extract input
    let (json_filename, queries) = read_queries(query_data);
    let filename = json_filename.strip_suffix(".json").unwrap();
    println!("JSON: {}", filename);

    // All necessary paths to CSV and PNG
    let json_path = format!("{}/{}.json", data_dir_path, filename);

    // Measurements
    for cutoff in cutoffs {
        measure_query(&json_path, &result_dir_path, filename, &queries, *cutoff);
    }
}

// Measure query time
fn measure_query(json_path: &str, result_dir_path: &str, filename: &str, queries: &[(String, String)], cutoff: usize) {
    let query_csv_path = format!("{}/rq_lut_time.csv", result_dir_path);
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
        wrt.write_record(&["JSON", "CUTOFF", "QUERY_ID", "QUERY_TEXT", "QUERY_TIME_SECONDS"])
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
        let query = rsonpath_syntax::parse(&query_text).expect("Failed to parse query");
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
            "  - File: {}, Cutoff: {}, Query {}: {}, Time = {:.5}s, Result = {}",
            filename, cutoff, query_id, query_text, avg_time, result
        );

        wrt.write_record(&[
            filename,
            cutoff.to_string().as_str(),
            query_id.as_str(),
            query_text.as_str(),
            &format!("{:.5}", avg_time),
        ])
        .expect("Failed to write to CSV");
    }

    wrt.flush().expect("Failed to flush CSV");
    println!("Generated: {}", query_csv_path);
}
