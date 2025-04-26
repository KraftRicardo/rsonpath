use crate::lookup_table::final_results::eval_rq_lut_no_lut::{QUERY_REPETITIONS, WARM_UP_QUERY_REPETITIONS};
use crate::lookup_table::performance::lut_skip_evaluation::SkipMode;
use crate::lookup_table::{SKIP_MODE, TRACK_SKIPPING_TIME_DURING_PERFORMANCE_TEST};
use crate::{
    engine::{Compiler, Engine, RsonpathEngine},
    input::OwnedBytes,
};
use csv::Writer;
use ph::fmph::keyset::GetIterator;
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::time::Instant;

pub const TRACK_SKIP_CUTOFF: usize = 0;

static SKIP_TIME_ATOMIC_CUTOFF_0: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_64: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_128: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_256: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_512: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_1024: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_2048: AtomicU64 = AtomicU64::new(0);

pub fn evaluate() {
    if SKIP_MODE != SkipMode::TRACK || !TRACK_SKIPPING_TIME_DURING_PERFORMANCE_TEST {
        println!("Wrong paramters. Abort");
        return;
    }

    // eval_test_data(QUERY_GOOGLE, cutoff);
    // eval_test_data(QUERY_BESTBUY, cutoff);
    // eval_test_data(QUERY_TWITTER, cutoff);
}

pub fn add_skip_time(added_time: u64) {
    SKIP_TIME_ATOMIC_CUTOFF_0.fetch_add(added_time, std::sync::atomic::Ordering::Relaxed);
    SKIP_TIME_ATOMIC_CUTOFF_64.fetch_add(added_time, std::sync::atomic::Ordering::Relaxed);
    SKIP_TIME_ATOMIC_CUTOFF_128.fetch_add(added_time, std::sync::atomic::Ordering::Relaxed);
    //...
}

fn eval_test_data(data_dir_path: &str, base_path: &str, test_data: (&str, &[(&str, &str)])) {
    // Extract input
    let (json_filename, queries) = test_data;
    let filename = json_filename.strip_suffix(".json").unwrap();
    println!("JSON: {}", filename);

    let json_path = format!("{}/{}.json", data_dir_path, filename);

    let results_csv_path = format!("{}/speed/final/query.csv", base_path);
    let file_exists = Path::new(&results_csv_path).exists();

    // Open CSV in append mode
    let mut wtr = Writer::from_writer(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&results_csv_path)
            .expect("Failed to open build CSV"),
    );

    // Write header if the file is new
    if !file_exists {
        println!("File did not exist");
        wtr.write_record(&["JSON", "CUTOFF", "QUERY_ID", "QUERY_TEXT", "SKIP_TIME_NANO_SECONDS"])
            .expect("F");
        wtr.flush().expect("Failed to flush build CSV");
    }

    for (query_id, query_text) in queries {
        // We have to do the query. Skip tracking is a side effect
        do_query(&json_path, query_id, query_text);

        let cutoffs = vec![0, 64, 128, 256, 512, 1024, 2048];
        for cutoff in cutoffs {
            let mut skip_time_nano_seconds = 0;
            if cutoff == 0 {
                skip_time_nano_seconds = SKIP_TIME_ATOMIC_CUTOFF_0.load(std::sync::atomic::Ordering::Relaxed);
            } else if cutoff == 64 {
                skip_time_nano_seconds = SKIP_TIME_ATOMIC_CUTOFF_64.load(std::sync::atomic::Ordering::Relaxed);
            } else if cutoff == 128 {
                skip_time_nano_seconds = SKIP_TIME_ATOMIC_CUTOFF_128.load(std::sync::atomic::Ordering::Relaxed);
            }
            // ...

            wtr.write_record(&[
                format!("{}", filename),
                format!("{}", cutoff),
                format!("{}", cutoff),
                format!("{}", query_id),
                format!("{}", query_text),
                format!("{}", skip_time_nano_seconds),
            ])
            .expect("F");
        }
    }
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

    let mut result = 0;
    for _ in 0..QUERY_REPETITIONS {
        result = engine.count(&input).expect("Fail count");
    }
    println!("  - Q: {} = {}, Result = {}", query_id, query_text, result);
}
