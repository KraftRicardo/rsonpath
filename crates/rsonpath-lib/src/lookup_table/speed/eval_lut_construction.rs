use crate::lookup_table::analysis::distance_distribution;
use crate::lookup_table::extra::{pair_data, util_path};
use crate::lookup_table::speed::lut_evaluation::{measure_performance, EvalConfig};
use crate::lookup_table::speed::lut_query_data::*;
use crate::lookup_table::QUERY_REPETITIONS;
use std::fs;
use std::io::Write;

// Run with: cargo run --bin lut --release -- eval-lut-construction res/json res/data/speed/local/lut_construction
// Run with: cargo run --bin lut --release -- eval-lut-construction ricardo-jsons final-results-10
pub fn run(data_dir_path: &str, result_dir_path: &str) {
    println!("eval-lut-construction");

    fs::create_dir_all(&result_dir_path).expect("Failed to create directory");

    // MB_1
    eval_all(&data_dir_path, &result_dir_path, QUERY_BESTBUY_SHORT);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF0);
    eval_all(&data_dir_path, &result_dir_path, QUERY_GOOGLE_SHORT);
    eval_all(&data_dir_path, &result_dir_path, QUERY_TWITTER_SHORT);
    eval_all(&data_dir_path, &result_dir_path, QUERY_WALMART_SHORT);

    // GB_1
    // eval_all(&data_dir_path, &result_dir_path, QUERY_BESTBUY);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF1);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF2);
    eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF4);
    eval_all(&data_dir_path, &result_dir_path, QUERY_GOOGLE);
    eval_all(&data_dir_path, &result_dir_path, QUERY_NSPL);
    eval_all(&data_dir_path, &result_dir_path, QUERY_TWITTER);
    eval_all(&data_dir_path, &result_dir_path, QUERY_WALMART);
    eval_all(&data_dir_path, &result_dir_path, QUERY_WIKI);

    // 25 GB
    eval_all(&data_dir_path, &result_dir_path, QUERY_NESTED_COL);

    println!("Done");
}

fn eval_all(data_dir_path: &str, final_dir_path: &str, test_data: (&str, &[(&str, &str)])) {
    // Extract input
    let (json_filename, _) = test_data;
    let filename = json_filename.strip_suffix(".json").unwrap();
    println!("JSON: {}", filename);

    let json_path = format!("{}/{}.json", data_dir_path, filename);
    let cutoff: usize = 0;
    println!("JSONPATH: {}, cutoff = {}", json_path, cutoff);

    let file = std::fs::File::open(&json_path).expect("Fail");
    let filename = util_path::extract_filename(&json_path);
    let num_keys = distance_distribution::count_num_pairs(&json_path);

    let mut head_line = String::from("name,input_size_bytes,num_keys,");
    let mut data_line = format!("{},{},{},", filename, file.metadata().expect("fail").len(), num_keys);

    let (keys, _) = pair_data::get_pairs_absolute(&json_path, cutoff).expect("Fail @ finding pairs.");

    let mut config = EvalConfig {
        json_path: &json_path,
        keys,
        head_line: &mut head_line,
        data_line: &mut data_line,
    };

    println!("Measuring LUT size with REPETITIONS = {}", QUERY_REPETITIONS);
    measure_performance(&mut config, cutoff).expect("Fail");

    // Write CSV header and data
    let csv_path = format!("{}/result.csv", final_dir_path);
    let mut csv_file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(csv_path)
        .expect("Fail");
    if csv_file.metadata().expect("Fail").len() == 0 {
        writeln!(csv_file, "{}", head_line).expect("Fail");
    }
    writeln!(csv_file, "{}", data_line).expect("Fail");
}
