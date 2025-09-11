use crate::evaluation::query_data::*;
use crate::evaluation::track_config::QUERY_REPETITIONS;
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

// Run with: cargo run --bin eval --release -- eval-optimal ../rsonpath/res/json ../rsonpath/res/data/speed/local/rq_legacy_skip_time
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
    println!("eval-optimal QUERY_REPETITIONS {QUERY_REPETITIONS}");

    if !cfg! {feature = "track-skipping"} {
        println!("Enable tracking of skips because otherwise this measurement does not work. Abort!");
        return;
    }
    if !cfg! {feature = "empty-list-opt"} {
        println!(
            "For fair comparisons with rq_legacy and rq_lut the feature empty-list-opt needs to be enabled. Abort!"
        );
        return;
    }

    eval_all(&data_dir_path, &result_dir_path, QUERY_BESTBUY);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF1);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF2);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF4);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_GOOGLE);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_NSPL);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_TWITTER);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_TWITTER_SINGLE);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_WALMART);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_WALMART_SINGLE);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_WIKI);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_WIKI_SINGLE);

    println!("Done")
}

pub fn add_skip_time(added_time: u64) {
    // println!("adding {}", added_time);
    ACCUMULATED_SKIP_TIME.fetch_add(added_time, Ordering::Relaxed);
}

fn eval_all(data_dir_path: &str, results_dir_path: &str, query_data_csv: &str) {
    let (json_path, json_name, queries) = extract_input(data_dir_path, query_data_csv);
    println!("JSON: {json_name}");

    fs::create_dir_all(&results_dir_path).expect("Failed to create directory");

    let results_csv_path = format!("{results_dir_path}/rq_legacy_skip_time_repetitions={QUERY_REPETITIONS}.csv");
    let file_exists = Path::new(&results_csv_path).exists();

    let mut wtr = Writer::from_writer(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&results_csv_path)
            .expect("Failed to open build CSV"),
    );

    if !file_exists {
        wtr.write_record(&[
            "JSON",
            "QUERY_ID",
            "QUERY_TEXT",
            "SKIP_TIME_NANO_SECONDS",
            "REPETITIONS",
        ])
        .expect("Failed to write header");
        wtr.flush().expect("Failed to flush CSV");
    }

    for (query_id, query_text) in queries {
        do_query(&json_path, &query_id, &query_text);

        let skip_time_nano_seconds = ACCUMULATED_SKIP_TIME.load(Ordering::Relaxed) as f64 / QUERY_REPETITIONS as f64;
        println!(", Skip time= {skip_time_nano_seconds}");

        wtr.write_record(&[
            format!("{}", json_name),
            format!("{}", query_id),
            format!("{}", query_text),
            format!("{:.}", skip_time_nano_seconds),
            format!("{}", QUERY_REPETITIONS),
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

    for _ in 0..QUERY_REPETITIONS {
        _ = engine.count(&input).expect("Fail count");
    }

    ACCUMULATED_SKIP_TIME.store(0, Ordering::Relaxed);

    let mut result = 0;
    for _ in 0..QUERY_REPETITIONS {
        result = engine.count(&input).expect("Fail count");
    }
    print!(
        "  - File: {}, Q: {} = {}, Result = {}",
        json_path, query_id, query_text, result
    );
}
