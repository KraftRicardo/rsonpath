use crate::lookup_table::performance::lut_query_data::{
    QUERY_BESTBUY, QUERY_CROSSREF1, QUERY_CROSSREF2, QUERY_CROSSREF4, QUERY_GOOGLE, QUERY_NSPL, QUERY_TWITTER,
    QUERY_WALMART, QUERY_WIKI,
};
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
    io::{BufReader, Read, Write},
};

pub const QUERY_REPETITIONS: usize = 10;
pub const WARM_UP_QUERY_REPETITIONS: usize = 10;

// Run with: cargo run --bin lut --release -- eval-rq-lut-no-lut .a_test_data .a_final_results
pub fn evaluate(data_dir_path: &str, base_path: &str) {
    println!("rq-lut-no-lut");

    if !(cfg! {feature = "empty-list-opt"}) {
        println!("empty-list-opt not set, aborting");
        return;
    }

    // Create results dir
    let result_dir_path = format!("{}/speed/rq-lut-no-lut", base_path);
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

    // Read legacy input once
    let input = {
        let mut buf = vec![];
        let mut file = BufReader::new(fs::File::open(json_path).expect("Failed to open input file"));
        file.read_to_end(&mut buf).expect("Failed to read input file");
        OwnedBytes::new(buf)
    };

    for &(query_id, query_text) in queries {
        let query = rsonpath_syntax::parse(query_text).expect("Failed to parse query");
        let mut engine = RsonpathEngine::compile_query(&query).expect("Failed to compile query");

        // Warm up
        for _ in 0..WARM_UP_QUERY_REPETITIONS {
            let _ = engine.count(&input).expect("Failed count");
        }

        // Measure query time
        let mut result = 0;
        let mut total_time = 0.0;

        for _ in 0..crate::lookup_table::performance::distance_cutoff_evaluation::QUERY_REPETITIONS {
            let start = Instant::now();
            result = engine.count(&input).expect("Fail count");
            total_time += start.elapsed().as_secs_f64();
        }

        let avg_time = total_time / (QUERY_REPETITIONS as f64);
        println!("  - Time = {:.5}s Result = {}", avg_time, result);

        wrt.write_record(&[query_id, query_text, &format!("{:.5}", avg_time)])
            .expect("Failed to write to CSV");
    }

    wrt.flush().expect("Failed to flush CSV");
}
