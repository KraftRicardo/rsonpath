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
use std::time::Instant;

static ACCUMULATED_SKIP_TIME: AtomicU64 = AtomicU64::new(0);

/// Evaluate skip-time performance of the RQ-Legacy engine on a set of benchmark queries.
///
/// This function runs a fixed collection of query workloads (BestBuy, CrossRef, Google, NSPL,
/// Twitter, Walmart, Wikipedia) on JSON inputs located in `data_dir_path`.
/// It measures the time spent in *skipping* operations during query execution,
/// averages this over `QUERY_REPETITIONS`, and appends the results to a CSV file
/// in `result_dir_path`.
///
/// Requirements
/// ------------
/// - Must be compiled with the `track-skipping` feature enabled (otherwise skip time is not tracked).
/// - Must be compiled with the `empty-list-opt` feature enabled for fair comparison with `rq_lut`.
///
/// Inputs
/// ------
/// * `data_dir_path` - Path to the directory containing JSON benchmark files.
/// * `result_dir_path` - Path to the directory where the result CSV will be written.
///   The directory will be created if it does not exist.
///
/// Outputs
/// -------
/// A CSV file is created (or appended to) at:
/// - `{result_dir_path}/rq_legacy_skip_time_node_repetitions={QUERY_REPETITIONS}.csv`
///
/// CSV schema:
/// ```text
/// JSON,QUERY_ID,QUERY_TEXT,SKIP_TIME_NANO_SECONDS,REPETITIONS
/// crossref1_(551MB).json,1,$.items[2].resource.primary.URL,66067533.6,20
/// bestbuy_large_record_(1GB).json,2,$.products[*].videoChapters,12033412.5,20
/// ```
///
/// Example
/// -------
/// ```bash
/// cargo run --bin eval --release -- eval-optimal ../rsonpath/res/json ../rsonpath/res/data/speed/local/rq_legacy_skip_time
/// cargo run --bin eval --release -- eval-optimal ricardo-jsons plot-results
/// ```
pub fn evaluate_rq_legacy_skip_time_speed(data_dir_path: &str, result_dir_path: &str) {
    println!("eval_legacy_skip_time");

    if !cfg! {feature = "track-skipping"} {
        println!("Enable tracking of skips because otherwise this measurement does not work. Abort!");
        return;
    }
    if !cfg! {feature = "empty-list-opt"} {
        println!("For fair comparisons with rq_legacy and rq_lut enable empty-list-opt. Abort!");
        return;
    }

    let use_count = false;

    eval(&data_dir_path, &result_dir_path, QUERY_BESTBUY, use_count);
    eval(&data_dir_path, &result_dir_path, QUERY_CROSSREF1, use_count);
    eval(&data_dir_path, &result_dir_path, QUERY_CROSSREF2, use_count);
    eval(&data_dir_path, &result_dir_path, QUERY_CROSSREF4, use_count);
    eval(&data_dir_path, &result_dir_path, QUERY_GOOGLE, use_count);
    eval(&data_dir_path, &result_dir_path, QUERY_NSPL, use_count);
    eval(&data_dir_path, &result_dir_path, QUERY_TWITTER, use_count);
    eval(&data_dir_path, &result_dir_path, QUERY_TWITTER_SINGLE, use_count);
    eval(&data_dir_path, &result_dir_path, QUERY_WALMART, use_count);
    eval(&data_dir_path, &result_dir_path, QUERY_WALMART_SINGLE, use_count);
    eval(&data_dir_path, &result_dir_path, QUERY_WIKI, use_count);
    eval(&data_dir_path, &result_dir_path, QUERY_WIKI_SINGLE, use_count);

    println!("Done")
}

fn eval(data_dir_path: &str, results_dir_path: &str, query_data_csv: &str, use_count: bool) {
    let (json_path, json_name, queries) = extract_input(data_dir_path, query_data_csv);

    let results_csv_path;
    if use_count {
        println!("Mode:Count, JSON:{json_name}");
        results_csv_path = format!("{results_dir_path}/rq_legacy_skip_time_repetitions={QUERY_REPETITIONS}.csv");
    } else {
        println!("Mode:Node, JSON:{json_name}");
        results_csv_path = format!("{results_dir_path}/rq_legacy_skip_time_node_repetitions={QUERY_REPETITIONS}.csv");
    }

    fs::create_dir_all(&results_dir_path).expect("Failed to create directory");
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
        let input = {
            let mut buf = vec![];
            let mut file = BufReader::new(fs::File::open(&json_path).expect("Failed to open input file"));
            file.read_to_end(&mut buf).expect("Failed to read input file");
            OwnedBytes::new(buf)
        };

        let query = rsonpath_syntax::parse(&query_text).expect("Failed to parse query");
        let mut engine = RsonpathEngine::compile_query(&query).expect("Failed to compile query");

        let mut result = 0;
        if use_count {
            // COUNT
            // Warm up
            for _ in 0..QUERY_REPETITIONS {
                _ = engine.count(&input).expect("Fail count");
            }

            // Measurement
            ACCUMULATED_SKIP_TIME.store(0, Ordering::Relaxed);
            for _ in 0..QUERY_REPETITIONS {
                result = engine.count(&input).expect("Fail count");
            }
        } else {
            // INDEX
            // Warm up
            for _ in 0..QUERY_REPETITIONS {
                let mut sink = vec![];
                engine.matches(&input, &mut sink).expect("Fail @ engine matching.");
            }

            // Measurement
            for _ in 0..QUERY_REPETITIONS {
                let mut sink = vec![];
                engine.matches(&input, &mut sink).expect("Fail @ engine matching.");
                result = sink.len() as u64;
            }
        }
        print!("  - File:{json_path}, Query:{query_id}:{query_text}, Result:{result}",);

        let skip_time_nano_seconds = ACCUMULATED_SKIP_TIME.load(Ordering::Relaxed) as f64 / QUERY_REPETITIONS as f64;
        println!(", Skip time= {skip_time_nano_seconds}");

        wtr.write_record(&[
            format!("{json_name}"),
            format!("{query_id}"),
            format!("{query_text}"),
            format!("{skip_time_nano_seconds:.}"),
            format!("{QUERY_REPETITIONS}"),
        ])
        .expect("Failed to write query result");
    }

    wtr.flush().expect("Failed to flush CSV after all queries");
}

pub fn add_skip_time(added_time: u64) {
    // println!("adding {}", added_time);
    ACCUMULATED_SKIP_TIME.fetch_add(added_time, Ordering::Relaxed);
}
