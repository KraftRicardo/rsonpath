use crate::evaluation::lut_query_data::{
    QUERY_BESTBUY, QUERY_CROSSREF1, QUERY_CROSSREF2, QUERY_CROSSREF4, QUERY_GOOGLE, QUERY_NSPL, QUERY_TWITTER,
    QUERY_WALMART, QUERY_WIKI,
};
use crate::evaluation::track_config::{REPETITIONS, TRACK_SKIPPING_ON, WARM_UP_REPETITIONS};
use crate::{
    engine::{Compiler, Engine, RsonpathEngine},
    input::OwnedBytes,
};
use csv::Writer;
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static ACCUMULATED_SKIP_TIME: AtomicU64 = AtomicU64::new(0);

// Run with: cargo run --bin eval --release -- eval-optimal ../rsonpath/.a_test_data ../rsonpath/.a_final_results/speed/optimal
// Run with: cargo run --bin eval --release -- eval-optimal ricardo-jsons plot-results
//
// "data_dir_path" is the path to folder holding the input JSON files.
// "result_dir_path" is the path to the folder where the results will be saved
//
// Data will be saved in "{result_dir_path}/optimal_time.csv".
// Result csv structure example:
//  JSON,CUTOFF,QUERY_ID,QUERY_TEXT,SKIP_TIME_NANO_SECONDS
//  crossref1_(551MB),0,1,$.items[2].resource.primary.URL,66067533.6
//  crossref1_(551MB),64,1,$.items[2].resource.primary.URL,66067022.6
//  ...
pub fn run(data_dir_path: &str, result_dir_path: &str) {
    if !TRACK_SKIPPING_ON {
        println!("Enable Skip Tracking first and disable the empty list optimization. Abort");
        return;
    }
    if !cfg! {feature = "empty-list-opt"} {
        println!("For fair comparisons with rsonpath-lut this feature needs to be enabled. Abort!");
        return;
    }

    eval_all(&data_dir_path, &result_dir_path, QUERY_BESTBUY);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF1);
    eval_all(&data_dir_path, &result_dir_path, QUERY_GOOGLE);
    eval_all(&data_dir_path, &result_dir_path, QUERY_NSPL);
    eval_all(&data_dir_path, &result_dir_path, QUERY_TWITTER);
    eval_all(&data_dir_path, &result_dir_path, QUERY_WALMART);
    eval_all(&data_dir_path, &result_dir_path, QUERY_WIKI);

    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF2);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF4);

    println!("Done")
}

pub fn add_skip_time(added_time: u64) {
    // println!("adding {}", added_time);
    ACCUMULATED_SKIP_TIME.fetch_add(added_time, Ordering::Relaxed);
}

fn eval_all(data_dir_path: &str, results_dir_path: &str, test_data: (&str, &[(&str, &str)])) {
    let (json_filename, queries) = test_data;
    let filename = json_filename.strip_suffix(".json").unwrap();
    println!("JSON: {}", filename);

    fs::create_dir_all(&results_dir_path).expect("Failed to create directory");

    let results_csv_path = format!("{}/optimal_time.csv", results_dir_path);
    let file_exists = Path::new(&results_csv_path).exists();

    let mut wtr = Writer::from_writer(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&results_csv_path)
            .expect("Failed to open build CSV"),
    );

    if !file_exists {
        println!("File did not exist, writing header.");
        wtr.write_record(&["JSON", "QUERY_ID", "QUERY_TEXT", "SKIP_TIME_NANO_SECONDS"])
            .expect("Failed to write header");
        wtr.flush().expect("Failed to flush CSV");
    }

    let json_path = format!("{}/{}.json", data_dir_path, filename);

    for (query_id, query_text) in queries {
        do_query(&json_path, query_id, query_text);

        let skip_time_nano_seconds = ACCUMULATED_SKIP_TIME.load(Ordering::Relaxed) as f64 / REPETITIONS as f64;

        wtr.write_record(&[
            format!("{}", filename),
            format!("{}", query_id),
            format!("{}", query_text),
            format!("{:.}", skip_time_nano_seconds),
        ])
        .expect("Failed to write query result");
    }

    wtr.flush().expect("Failed to flush CSV after all queries");
}

fn do_query(json_path: &str, query_id: &str, query_text: &str) {
    let input = {
        let mut buf = vec![];
        let mut file = BufReader::new(fs::File::open(json_path).expect("Failed to open input file"));
        file.read_to_end(&mut buf).expect("Failed to read input file");
        OwnedBytes::new(buf)
    };

    let query = rsonpath_syntax::parse(query_text).expect("Failed to parse query");
    let mut engine = RsonpathEngine::compile_query(&query).expect("Failed to compile query");

    for _ in 0..WARM_UP_REPETITIONS {
        _ = engine.count(&input).expect("Fail count");
    }

    ACCUMULATED_SKIP_TIME.store(0, Ordering::Relaxed);

    let mut result = 0;
    for _ in 0..REPETITIONS {
        result = engine.count(&input).expect("Fail count");
    }
    println!(
        "  - File: {}, Q: {} = {}, Result = {}",
        json_path, query_id, query_text, result
    );
}
