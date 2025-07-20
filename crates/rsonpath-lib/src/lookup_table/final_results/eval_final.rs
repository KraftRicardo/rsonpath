use crate::lookup_table::performance::lut_query_data::{
    QUERY_BESTBUY, QUERY_CROSSREF1, QUERY_CROSSREF2, QUERY_CROSSREF4, QUERY_GOOGLE, QUERY_NSPL, QUERY_TWITTER,
    QUERY_WALMART, QUERY_WIKI,
};
use crate::lookup_table::{BUILD_REPETITIONS, QUERY_REPETITIONS, WARM_UP_BUILD_REPETITIONS, WARM_UP_QUERY_REPETITIONS};
use crate::{
    engine::{Compiler, Engine, RsonpathEngine},
    input::OwnedBytes,
    lookup_table::{LookUpTable, LUT},
};
use csv::Writer;
use rsonpath_lib_ref::engine::{Compiler as CompilerLegacy, Engine as EngineLegacy};
use serde_json::Value;
use serde_json_path::JsonPath;
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufReader, Read};
use std::path::Path;
use std::time::Instant;

const SERDE_NAME: &str = "SERDE";
const RQ_LEGACY_NAME: &str = "rq-legacy";
const RQ_LUT_CUTOFF_0_NAME: &str = "rq-lut-cutoff-0";
const RQ_LUT_CUTOFF_512_NAME: &str = "rq-lut-cutoff-512";

// This compares serde, rq-legacy, rq-lut-cutoff-0 and rq-lut-cutoff-512 for several queries and
// also measure the build time for each (if there is a build step needed).
//
// Run with: cargo run --bin lut --release -- eval-final res/json res/data/speed/local/final
// Run with: cargo run --bin lut --release -- eval-final ricardo-jsons plot-results
pub fn evaluate(data_dir_path: &str, result_dir_path: &str) {
    println!("final");

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

    let json_path = format!("{}/{}.json", data_dir_path, filename);

    fs::create_dir_all(&result_dir_path).expect("Failed to create directory");

    // Measurements
    measure_build(&json_path, filename, result_dir_path);
    measure_query_index(&json_path, filename, result_dir_path, queries);
}

fn measure_query_index(json_path: &str, filename: &str, result_dir_path: &str, queries: &[(&str, &str)]) {
    let query_csv_path = format!("{}/query.csv", result_dir_path);
    let file_exists = Path::new(&query_csv_path).exists();

    // Open CSV in append mode
    let mut wtr = Writer::from_writer(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&query_csv_path)
            .expect("Failed to open build CSV"),
    );

    // Write header if the file is new
    if !file_exists {
        wtr.write_record(&["JSON", "ALGORITHM", "QUERY_ID", "QUERY_TEXT", "AVERAGE_TIME"])
            .expect("F");
        wtr.flush().expect("Failed to flush build CSV");
    }

    // Measurements
    let serde_query_times = query_serde_index(&json_path, queries);
    let rq_legacy_query_times = query_rq_legacy_index(&json_path, queries);
    let rq_lut_cutoff_0_query_times = query_rq_lut_index(&json_path, queries, 0);
    let rq_lut_cutoff_512_query_times = query_rq_lut_index(&json_path, queries, 512);

    for (index, (query_id, query_text)) in queries.iter().enumerate() {
        // Write the results
        wtr.write_record(&[
            filename,
            SERDE_NAME,
            query_id,
            query_text,
            &format!("{:.5}", serde_query_times[index]),
        ])
        .expect("Fail write");
        wtr.write_record(&[
            filename,
            RQ_LEGACY_NAME,
            query_id,
            query_text,
            &format!("{:.5}", rq_legacy_query_times[index]),
        ])
        .expect("Fail write");
        wtr.write_record(&[
            filename,
            RQ_LUT_CUTOFF_0_NAME,
            query_id,
            query_text,
            &format!("{:.5}", rq_lut_cutoff_0_query_times[index]),
        ])
        .expect("Fail write");
        wtr.write_record(&[
            filename,
            RQ_LUT_CUTOFF_512_NAME,
            query_id,
            query_text,
            &format!("{:.5}", rq_lut_cutoff_512_query_times[index]),
        ])
        .expect("Fail write");
    }

    wtr.flush().expect("Failed to flush build CSV");
    println!("Generated: {query_csv_path}")
}

fn query_rq_lut_index(json_path: &str, queries: &[(&str, &str)], cutoff: usize) -> Vec<f64> {
    let mut lut = LUT::build(json_path, cutoff).expect("Failed to build LUT");

    let input = {
        let mut buf = vec![];
        let mut file = BufReader::new(fs::File::open(json_path).expect("Failed to open input file"));
        file.read_to_end(&mut buf).expect("Failed to read input file");
        OwnedBytes::new(buf)
    };

    let mut avg_times = vec![];
    for &(query_id, query_text) in queries {
        let query = rsonpath_syntax::parse(query_text).expect("Failed to parse query");
        let mut engine = RsonpathEngine::compile_query(&query).expect("Failed to compile query");
        engine.add_lut(lut);

        for _ in 0..WARM_UP_QUERY_REPETITIONS {
            // COUNT
            // let _ = engine.count(&input).expect("Warmup failed");

            // INDEX
            let mut sink = vec![];
            engine.matches(&input, &mut sink).expect("Fail @ engine matching.");
        }

        let mut result = 0;
        let mut total_time = 0.0;

        for _ in 0..QUERY_REPETITIONS {
            // COUNT
            // let start = Instant::now();
            // result = engine.count(&input).expect("Query execution failed");
            // total_time += start.elapsed().as_secs_f64();

            // INDEX
            let mut sink = vec![];
            let start = Instant::now();
            engine.matches(&input, &mut sink).expect("Fail @ engine matching.");
            total_time += start.elapsed().as_secs_f64();
            result = sink.len();
        }

        let avg_time = total_time / QUERY_REPETITIONS as f64;

        println!(
            "  - LUT ({}): id={}, query_text={}, time={:.5}s, result={}",
            cutoff, query_id, query_text, avg_time, result
        );
        avg_times.push(avg_time);

        lut = engine.take_lut().expect("Failed to retrieve LUT");
    }

    avg_times
}

fn query_rq_legacy_index(json_path: &str, queries: &[(&str, &str)]) -> Vec<f64> {
    let legacy_input = {
        let mut file = BufReader::new(fs::File::open(json_path).expect("Failed to open file"));
        let mut buf = vec![];
        file.read_to_end(&mut buf).expect("Failed to read file");
        rsonpath_lib_ref::input::OwnedBytes::new(buf)
    };

    let mut avg_times = vec![];
    for &(query_id, query_text) in queries {
        let legacy_query = syntax_ref::parse(query_text).expect("Failed to parse query");
        let legacy_engine =
            rsonpath_lib_ref::engine::RsonpathEngine::compile_query(&legacy_query).expect("Failed to compile query");

        for _ in 0..WARM_UP_QUERY_REPETITIONS {
            // COUNT
            // let _ = legacy_engine.count(&legacy_input).expect("Warmup failed");

            // INDEX
            let mut sink = vec![];
            legacy_engine
                .matches(&legacy_input, &mut sink)
                .expect("Fail @ engine matching.");
        }

        let mut result = 0;
        let mut total_time = 0.0;

        for _ in 0..QUERY_REPETITIONS {
            // COUNT
            // let start = Instant::now();
            // result = legacy_engine.count(&legacy_input).expect("Query execution failed");
            // total_time += start.elapsed().as_secs_f64();

            // INDEX
            let mut sink = vec![];
            let start = Instant::now();
            legacy_engine
                .matches(&legacy_input, &mut sink)
                .expect("Fail @ engine matching.");
            total_time += start.elapsed().as_secs_f64();
            result = sink.len();
        }

        let avg_time = total_time / QUERY_REPETITIONS as f64;

        println!(
            "  - LEGACY: id={}, query_text={}, time={:.5}s, result={}",
            query_id, query_text, avg_time, result
        );
        avg_times.push(avg_time);
    }

    avg_times
}

fn query_serde_index(json_path: &str, queries: &[(&str, &str)]) -> Vec<f64> {
    let file = fs::File::open(json_path).expect("Failed to open file");
    let reader = BufReader::new(file);
    let json_value: Value = serde_json::from_reader(reader).expect("Failed to parse JSON");

    let mut avg_times = vec![];
    for &(query_id, query_text) in queries {
        let path = JsonPath::parse(query_text).expect("Failed to parse JSONPath");

        for _ in 0..WARM_UP_QUERY_REPETITIONS {
            let _ = path.query(&json_value);
        }

        let mut result = 0;
        let mut total_time = 0.0;

        for _ in 0..QUERY_REPETITIONS {
            let start = Instant::now();
            let nodes = path.query(&json_value);
            total_time += start.elapsed().as_secs_f64();
            result = nodes.len() as u64;
        }

        let avg_time = total_time / QUERY_REPETITIONS as f64;

        println!(
            "  - SERDE: id={}, query_text={}, time={:.5}s, result={}",
            query_id, query_text, avg_time, result
        );
        avg_times.push(avg_time);
    }

    avg_times
}

fn measure_build(json_path: &str, filename: &str, result_dir_path: &str) {
    let build_csv_path = format!("{}/build.csv", result_dir_path);
    let file_exists = Path::new(&build_csv_path).exists();

    // Open CSV in append mode
    let mut wtr = Writer::from_writer(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&build_csv_path)
            .expect(&format!("Failed to open build CSV {}", build_csv_path)),
    );

    // Write header if the file is new
    if !file_exists {
        println!("File did not exist");
        wtr.write_record(&["JSON", "ALGORITHM", "BUILD_TIME_SECONDS"])
            .expect("Failed to write header");
        wtr.flush().expect("Failed to flush build CSV");
    }

    // Measurements
    let serde_build_time = build_serde(&json_path);
    let rq_legacy_build_time = 0; // Because it does not need to build anything
    let rq_lut_cutoff_0_build_time = build_rq_lut(&json_path, 0);
    let rq_lut_cutoff_512_build_time = build_rq_lut(&json_path, 512);

    // Write the results
    wtr.write_record(&[filename, SERDE_NAME, &format!("{:.5}", serde_build_time)])
        .expect("Fail write");
    wtr.write_record(&[filename, RQ_LEGACY_NAME, &format!("{:.5}", rq_legacy_build_time)])
        .expect("Fail write");
    wtr.write_record(&[
        filename,
        RQ_LUT_CUTOFF_0_NAME,
        &format!("{:.5}", rq_lut_cutoff_0_build_time),
    ])
    .expect("Fail write");
    wtr.write_record(&[
        filename,
        RQ_LUT_CUTOFF_512_NAME,
        &format!("{:.5}", rq_lut_cutoff_512_build_time),
    ])
    .expect("Fail write");

    wtr.flush().expect("Failed to flush build CSV");
    println!("Generated: {build_csv_path}")
}

fn build_rq_lut(json_path: &str, cutoff: usize) -> f64 {
    // Warm-up
    for _ in 0..WARM_UP_BUILD_REPETITIONS {
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
    println!(" rq-lut-cutoff={}: build time = {:.5}s", cutoff, avg_time);

    avg_time
}

fn build_serde(json_path: &str) -> f64 {
    // Warm-up
    for _ in 0..WARM_UP_BUILD_REPETITIONS {
        let file = fs::File::open(json_path).expect("Failed to open file");
        let reader = BufReader::new(file);
        let _: Value = serde_json::from_reader(reader).expect("Failed to parse JSON");
    }

    // Measure build time
    let mut total_time = 0.0;
    for _ in 0..BUILD_REPETITIONS {
        let start = Instant::now();

        let file = fs::File::open(json_path).expect("Failed to open file");
        let reader = BufReader::new(file);
        let _: Value = serde_json::from_reader(reader).expect("Failed to parse JSON");

        total_time += start.elapsed().as_secs_f64();
    }

    let avg_time = total_time / BUILD_REPETITIONS as f64;
    println!(" SERDE build time = {:.5}s", avg_time);

    avg_time
}
