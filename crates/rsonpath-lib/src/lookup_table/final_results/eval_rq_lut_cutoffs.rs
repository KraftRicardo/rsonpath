use crate::lookup_table::performance::lut_query_data::{
    QUERY_BESTBUY, QUERY_CROSSREF1, QUERY_CROSSREF2, QUERY_CROSSREF4, QUERY_GOOGLE, QUERY_NSPL, QUERY_TWITTER,
    QUERY_WALMART, QUERY_WIKI,
};
use crate::lookup_table::performance::lut_skip_evaluation::SkipMode;
use crate::lookup_table::SKIP_MODE;
use crate::{
    engine::{Compiler, Engine, RsonpathEngine},
    input::OwnedBytes,
    lookup_table::{performance::lut_evaluation::HEAP_TRACKER, util_path, LookUpTable, LUT},
};
use csv::Writer;
use stats_alloc::Region;
use std::fs::OpenOptions;
use std::path::Path;
use std::time::Instant;
use std::{
    fs,
    io::{self, BufReader, Read, Write},
    process::Command,
};

pub const QUERY_REPETITIONS: usize = 20;
pub const BUILD_REPETITIONS: usize = 3;
pub const WARM_UP_QUERY_REPETITIONS: usize = 10;
pub const WARM_UP_REPETITIONS: usize = 3;

// run with: cargo run --bin lut --release -- cutoff .a_test_data .a_final_results
// run with: cargo run --bin lut --release -- cutoff ricardo-jsons final-results-4
pub fn evaluate(data_dir_path: &str, base_path: &str) {
    println!("lut_ptrhash_double_empty_list_opt");

    // let cutoffs: Vec<usize> = vec![0, 64, 128, 192, 256, 320, 384, 448, 512, 1024, 2048, 4096, 8192];

    let cutoffs = vec![0, 64, 128, 512, 8192];

    if SKIP_MODE != SkipMode::OFF {
        println!("Skipping mode or Strategy are not set correctly. Aborting");
        return;
    }

    let result_dir_path = format!("{}/speed/lut_ptrhash_double_empty_list_opt", base_path);
    fs::create_dir_all(&result_dir_path).expect("Failed to create directory");

    // GB_1
    eval_all(&data_dir_path, &result_dir_path, QUERY_BESTBUY, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF1, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF2, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF4, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_GOOGLE, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_NSPL, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_TWITTER, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_WALMART, &cutoffs);
    eval_all(&data_dir_path, &result_dir_path, QUERY_WIKI, &cutoffs);

    println!("Done");
}

// run with: cargo run --bin lut --release -- cutoff-plot
pub fn plot() {
    // GB_1
    // plot_all(QUERY_BESTBUY);
    // plot_all(QUERY_CROSSREF1);
    // plot_all(QUERY_CROSSREF2);
    // plot_all(QUERY_CROSSREF4);
    // plot_all(QUERY_GOOGLE);
    // plot_all(QUERY_NSPL);
    // plot_all(QUERY_TWITTER);
    // plot_all(QUERY_WALMART);
    // plot_all(QUERY_WIKI);
}

fn eval_all(data_dir_path: &str, result_dir_path: &str, test_data: (&str, &[(&str, &str)]), cutoffs: &Vec<usize>) {
    // Extract input
    let (json_filename, queries) = test_data;
    let filename = json_filename.strip_suffix(".json").unwrap();
    println!("JSON: {}", filename);

    // Measurements
    for cutoff in cutoffs {
        println!("  cutoff {cutoff}");

        // All necessary paths to CSV and PNG
        let cutoff_dir_path = format!("{}/{}", result_dir_path, cutoff);
        fs::create_dir_all(&cutoff_dir_path).expect("Failed to create directory");

        let json_path = format!("{}/{}.json", data_dir_path, filename);

        measure_build(&json_path, &cutoff_dir_path, filename, *cutoff);
        measure_query(&json_path, &result_dir_path, filename, *cutoff, queries);
    }
}

// Measure query time
fn measure_query(json_path: &str, result_dir_path: &str, filename: &str, cutoff: usize, queries: &[(&str, &str)]) {
    // Ensure the result directory exists
    fs::create_dir_all(result_dir_path).expect("Failed to create results directory");

    let query_csv_path = format!("{}/{}/{}.csv", result_dir_path, cutoff, filename);
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

    // Build LUT once and read input file into memory
    let mut lut = LUT::build(json_path, cutoff).expect("Failed to build LUT");

    let input = {
        let mut buf = vec![];
        let mut file = BufReader::new(fs::File::open(json_path).expect("Failed to open input file"));
        file.read_to_end(&mut buf).expect("Failed to read input file");
        OwnedBytes::new(buf)
    };

    for &(query_id, query_text) in queries {
        let query = rsonpath_syntax::parse(query_text).expect("Failed to parse query");
        let mut engine = RsonpathEngine::compile_query(&query).expect("Failed to compile query");
        engine.add_lut(lut);

        // Warm up
        for _ in 0..WARM_UP_QUERY_REPETITIONS {
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

        let avg_time = total_time / QUERY_REPETITIONS as f64;
        lut = engine.take_lut().expect("Failed to retrieve LUT");

        println!(
            "  - query = {}, query_text={}, time = {:.5}s, result = {}",
            query_id, query_text, avg_time, result
        );

        wrt.write_record(&[query_id, query_text, &format!("{:.5}", avg_time)])
            .expect("Failed to write to CSV");
    }

    wrt.flush().expect("Failed to flush CSV");
}

fn measure_build(json_path: &str, cutoff_dir_path: &str, filename: &str, cutoff: usize) {
    let build_csv = format!("{}/build.csv", cutoff_dir_path);
    let file_exists = Path::new(&build_csv).exists();

    // Ensure the directory exists
    fs::create_dir_all(cutoff_dir_path).expect("Failed to create directory");

    // Open CSV in append mode
    let mut wtr = Writer::from_writer(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&build_csv)
            .expect("Failed to open build CSV"),
    );

    // Write header if the file is new
    if !file_exists {
        wtr.write_record(&["JSON", "BUILD_TIME_SECONDS", "SIZE_IN_BYTES"])
            .expect("Failed to write header");
    }

    // Measure size
    let start_heap = Region::new(HEAP_TRACKER);
    let lut = LUT::build(json_path, cutoff).expect("Failed to build LUT");
    let heap_bytes = heap_value(start_heap.change());
    println!("Measured heap with cutoff={}", lut.get_cutoff());
    drop(lut);

    // Warm-up
    for _ in 0..WARM_UP_REPETITIONS {
        let _ = LUT::build(json_path, cutoff).expect("Warm-up build failed");
    }

    // Measure build time
    let mut total_time = 0.0;
    for _ in 0..BUILD_REPETITIONS {
        let start = Instant::now();
        let _ = LUT::build(json_path, cutoff).expect("Timed build failed");
        total_time += start.elapsed().as_secs_f64();
    }

    let avg_time = total_time / BUILD_REPETITIONS as f64;
    println!(" build time = {:.5}s, size = {} B", avg_time, heap_bytes);

    // Write the results
    wtr.write_record(&[filename, &format!("{:.5}", avg_time), &heap_bytes.to_string()])
        .expect("Failed to write build record");

    wtr.flush().expect("Failed to flush build CSV");
}

// We take the allocated bytes minus the deallocated and ignore the reallocated bytes because we are interested
// in the total heap space taken
pub fn heap_value(stats: stats_alloc::Stats) -> isize {
    stats.bytes_allocated as isize - stats.bytes_deallocated as isize
}

fn plot_all(result_dir_path: &str, test_data: (&str, &[(&str, &str)])) {
    // Extract input
    let (json_path, queries) = test_data;
    let filename = util_path::extract_filename(json_path);
    println!("JSON: {}", json_path);

    // All necessary paths to CSV and PNG
    let build_csv = format!("{}/{}_build_results.csv", result_dir_path, filename);
    let query_csv = format!("{}/{}_query_results.csv", result_dir_path, filename);
    let counter_csv_path = format!("{}/../skip_tracker/COUNTER_{}.csv", result_dir_path, filename);
    let distance_image_path = format!(
        "{}/../../analysis/distance_distribution/{}_plot.png",
        result_dir_path, filename
    );
    fs::create_dir_all(&result_dir_path).expect("Could not create results directory");

    // Plot it with python
    run_python_statistics_builder(&build_csv, &query_csv, &counter_csv_path, &distance_image_path);
}

fn run_python_statistics_builder(
    build_csv_path: &str,
    query_csv_path: &str,
    counter_csv_path: &str,
    distance_image_path: &str,
) {
    let msg = format!("Failed to open csv_path: {}", build_csv_path);
    let output = Command::new("python")
        .arg("crates/rsonpath-lib/src/lookup_table/python_statistic/distance_cutoff_evaluation.py")
        .arg(build_csv_path)
        .arg(query_csv_path)
        .arg(counter_csv_path)
        .arg(distance_image_path)
        .output()
        .expect(&msg);

    if output.status.success() {
        if let Err(e) = io::stdout().write_all(&output.stdout) {
            eprintln!("Failed to write stdout: {}", e);
        }
    } else {
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
    }
}
