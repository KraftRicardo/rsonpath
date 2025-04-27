use crate::lookup_table::performance::lut_query_data::{
    QUERY_BESTBUY, QUERY_CROSSREF1, QUERY_CROSSREF2, QUERY_CROSSREF4, QUERY_GOOGLE, QUERY_NSPL, QUERY_TWITTER,
    QUERY_WALMART, QUERY_WIKI,
};
use crate::lookup_table::performance::lut_skip_evaluation::SkipMode;
use crate::lookup_table::{SKIP_MODE, TRACK_SKIPPING_ON};
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

pub const QUERY_REPETITIONS: usize = 5;

static SKIP_TIME_ATOMIC_CUTOFF_0: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_64: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_128: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_256: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_512: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_1024: AtomicU64 = AtomicU64::new(0);
static SKIP_TIME_ATOMIC_CUTOFF_2048: AtomicU64 = AtomicU64::new(0);

// Run with: cargo run --bin lut --release -- eval-t-optimal .a_test_data .a_final_results
// Run with: cargo run --bin lut --release -- eval-final ricardo-jsons final_results-7
pub fn evaluate(data_dir_path: &str, base_path: &str) {
    if SKIP_MODE != SkipMode::TRACK || !TRACK_SKIPPING_ON || cfg! {feature = "empty-list-opt"} {
        println!("Wrong parameters. Abort");
        return;
    }

    let cutoffs = vec![0, 64, 128, 256, 512, 1024, 2048];

    eval_all(&data_dir_path, &base_path, QUERY_BESTBUY, &cutoffs);
    eval_all(&data_dir_path, &base_path, QUERY_CROSSREF1, &cutoffs);
    eval_all(&data_dir_path, &base_path, QUERY_CROSSREF2, &cutoffs);
    eval_all(&data_dir_path, &base_path, QUERY_CROSSREF4, &cutoffs);
    eval_all(&data_dir_path, &base_path, QUERY_GOOGLE, &cutoffs);
    eval_all(&data_dir_path, &base_path, QUERY_NSPL, &cutoffs);
    eval_all(&data_dir_path, &base_path, QUERY_TWITTER, &cutoffs);
    eval_all(&data_dir_path, &base_path, QUERY_WALMART, &cutoffs);
    eval_all(&data_dir_path, &base_path, QUERY_WIKI, &cutoffs);
}

pub fn add_skip_time(distance: usize, added_time: u64) {
    if distance > 0 {
        SKIP_TIME_ATOMIC_CUTOFF_0.fetch_add(added_time, Ordering::Relaxed);
    }
    if distance > 64 {
        SKIP_TIME_ATOMIC_CUTOFF_64.fetch_add(added_time, Ordering::Relaxed);
    }
    if distance > 128 {
        SKIP_TIME_ATOMIC_CUTOFF_128.fetch_add(added_time, Ordering::Relaxed);
    }
    if distance > 256 {
        SKIP_TIME_ATOMIC_CUTOFF_256.fetch_add(added_time, Ordering::Relaxed);
    }
    if distance > 512 {
        SKIP_TIME_ATOMIC_CUTOFF_512.fetch_add(added_time, Ordering::Relaxed);
    }
    if distance > 1024 {
        SKIP_TIME_ATOMIC_CUTOFF_1024.fetch_add(added_time, Ordering::Relaxed);
    }
    if distance > 2048 {
        SKIP_TIME_ATOMIC_CUTOFF_2048.fetch_add(added_time, Ordering::Relaxed);
    }
}

fn eval_all(data_dir_path: &str, base_path: &str, test_data: (&str, &[(&str, &str)]), cutoffs: &Vec<usize>) {
    let (json_filename, queries) = test_data;
    let filename = json_filename.strip_suffix(".json").unwrap();
    println!("JSON: {}", filename);

    let results_dir_path = format!("{}/speed/optimal", base_path);
    fs::create_dir_all(&results_dir_path).expect("Failed to create directory");

    let results_csv_path = format!("{}/query.csv", results_dir_path);
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
        wtr.write_record(&["JSON", "CUTOFF", "QUERY_ID", "QUERY_TEXT", "SKIP_TIME_NANO_SECONDS"])
            .expect("Failed to write header");
        wtr.flush().expect("Failed to flush CSV");
    }

    let json_path = format!("{}/{}.json", data_dir_path, filename);

    for (query_id, query_text) in queries {
        reset_skip_counters();

        do_query(&json_path, query_id, query_text);

        for cutoff in cutoffs {
            let skip_time_nano_seconds = match cutoff {
                0 => SKIP_TIME_ATOMIC_CUTOFF_0.load(Ordering::Relaxed),
                64 => SKIP_TIME_ATOMIC_CUTOFF_64.load(Ordering::Relaxed),
                128 => SKIP_TIME_ATOMIC_CUTOFF_128.load(Ordering::Relaxed),
                256 => SKIP_TIME_ATOMIC_CUTOFF_256.load(Ordering::Relaxed),
                512 => SKIP_TIME_ATOMIC_CUTOFF_512.load(Ordering::Relaxed),
                1024 => SKIP_TIME_ATOMIC_CUTOFF_1024.load(Ordering::Relaxed),
                2048 => SKIP_TIME_ATOMIC_CUTOFF_2048.load(Ordering::Relaxed),
                _ => 0,
            } as f64
                / QUERY_REPETITIONS as f64;

            wtr.write_record(&[
                format!("{}", filename),
                format!("{}", cutoff),
                format!("{}", query_id),
                format!("{}", query_text),
                format!("{:.}", skip_time_nano_seconds),
            ])
            .expect("Failed to write query result");
        }
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

    let mut result = 0;
    for _ in 0..QUERY_REPETITIONS {
        result = engine.count(&input).expect("Fail count");
    }
    println!("  - Q: {} = {}, Result = {}", query_id, query_text, result);
}

fn reset_skip_counters() {
    SKIP_TIME_ATOMIC_CUTOFF_0.store(0, Ordering::Relaxed);
    SKIP_TIME_ATOMIC_CUTOFF_64.store(0, Ordering::Relaxed);
    SKIP_TIME_ATOMIC_CUTOFF_128.store(0, Ordering::Relaxed);
    SKIP_TIME_ATOMIC_CUTOFF_256.store(0, Ordering::Relaxed);
    SKIP_TIME_ATOMIC_CUTOFF_512.store(0, Ordering::Relaxed);
    SKIP_TIME_ATOMIC_CUTOFF_1024.store(0, Ordering::Relaxed);
    SKIP_TIME_ATOMIC_CUTOFF_2048.store(0, Ordering::Relaxed);
}
