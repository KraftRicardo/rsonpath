use crate::lookup_table::analysis::distance_distribution;
use crate::lookup_table::performance::lut_evaluation::{measure_performance, EvalConfig};
use crate::lookup_table::performance::lut_query_data::{
    QUERY_BESTBUY, QUERY_BESTBUY_SHORT, QUERY_CROSSREF0, QUERY_CROSSREF1, QUERY_CROSSREF2, QUERY_CROSSREF4,
    QUERY_GOOGLE, QUERY_GOOGLE_SHORT, QUERY_NSPL, QUERY_TWITTER, QUERY_TWITTER_SHORT, QUERY_WALMART,
    QUERY_WALMART_SHORT, QUERY_WIKI,
};
use crate::lookup_table::{pair_data, util_path, REPETITIONS};
use std::fs;
use std::io::Write;

pub const BUILD_REPETITIONS: usize = 3;
pub const QUERY_REPETITIONS: usize = 5;

// Run with: cargo run --bin lut --release -- eval-lut-construction .a_test_data .a_final_results
// Run with: cargo run --bin lut --release -- eval-lut-construction ricardo-jsons final-results-10
pub fn evaluate(data_dir_path: &str, base_path: &str) {
    println!("lut construction evaluation");

    let final_dir_path = format!("{}/speed/lut-construction", base_path);
    fs::create_dir_all(&final_dir_path).expect("Failed to create directory");

    // MB_1
    // eval_all(&data_dir_path, &final_dir_path, QUERY_BESTBUY_SHORT);
    // eval_all(&data_dir_path, &final_dir_path, QUERY_CROSSREF0);
    // eval_all(&data_dir_path, &final_dir_path, QUERY_GOOGLE_SHORT);
    // eval_all(&data_dir_path, &final_dir_path, QUERY_TWITTER_SHORT);
    // eval_all(&data_dir_path, &final_dir_path, QUERY_WALMART_SHORT);

    // GB_1
    eval_all(&data_dir_path, &final_dir_path, QUERY_BESTBUY);
    eval_all(&data_dir_path, &final_dir_path, QUERY_CROSSREF1);
    eval_all(&data_dir_path, &final_dir_path, QUERY_CROSSREF2);
    eval_all(&data_dir_path, &final_dir_path, QUERY_CROSSREF4);
    eval_all(&data_dir_path, &final_dir_path, QUERY_GOOGLE);
    eval_all(&data_dir_path, &final_dir_path, QUERY_NSPL);
    eval_all(&data_dir_path, &final_dir_path, QUERY_TWITTER);
    eval_all(&data_dir_path, &final_dir_path, QUERY_WALMART);
    eval_all(&data_dir_path, &final_dir_path, QUERY_WIKI);

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

    println!("Measuring LUT size with REPETITIONS = {}", REPETITIONS);
    measure_performance(&mut config, cutoff).expect("Fail");

    // Write CSV header and data
    let csv_path = format!("{}/result.csv", final_dir_path);
    let mut csv_file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(csv_path)
        .expect("Fail");
    if csv_file.metadata().expect("Fail").len() == 0 {
        writeln!(csv_file, "{}", head_line).expect("Fail");
    }
    writeln!(csv_file, "{}", data_line).expect("Fail");
}
