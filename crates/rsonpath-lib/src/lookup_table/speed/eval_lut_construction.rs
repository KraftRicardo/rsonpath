use crate::lookup_table::analysis::distance_distribution;
use crate::lookup_table::extra::util_path;
use crate::lookup_table::luts::pair_data;
use crate::lookup_table::speed::lut_evaluation::{measure_performance, EvalConfig};
use crate::lookup_table::speed::lut_query_data::*;
use crate::lookup_table::QUERY_REPETITIONS;
use std::fs;
use std::io::Write;

/// This experiment measures the build_time, key-access-time (query every key once) and heap size
/// of the different LUT implementations that are enabled in the lut_evaluations.rs.
/// The output is a csv holding the results for each LUT implementation on different, e.g.:
///     name,input_size_bytes,num_keys,perfect_naive_BUILD,perfect_naive_QUERY,perfect_naive_HEAP, ...
///     bestbuy_short_(103MB),103231582,680183,0.211277691,0.014003506,58966256,0.065568449, ...
///     crossref0_(320MB),320401529,3130672,1.353163136,0.063439373,1201499616,0.489899481, ...
///     ...
/// The output will add more columns the more different LUT implementations are added.
///
/// Run with: cargo run --bin lut --release -- eval-lut-construction res/json res/data/speed/local/lut_construction
/// Run with: cargo run --bin lut --release -- eval-lut-construction ricardo-jsons final-results-10
pub fn run(data_dir_path: &str, result_dir_path: &str) {
    println!("eval-lut-construction");

    let cutoff: usize = 0;

    fs::create_dir_all(&result_dir_path).expect("Failed to create directory");

    // MB_100
    eval_all(&data_dir_path, &result_dir_path, QUERY_BESTBUY_SHORT, cutoff);

    // GB_1
    // eval_all(&data_dir_path, &result_dir_path, QUERY_BESTBUY, cutoff);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF1, cutoff);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF2, cutoff);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_CROSSREF4, cutoff);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_GOOGLE, cutoff);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_NSPL, cutoff);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_TWITTER, cutoff);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_WALMART, cutoff);
    // eval_all(&data_dir_path, &result_dir_path, QUERY_WIKI, cutoff);

    // 25 GB
    // eval_all(&data_dir_path, &result_dir_path, QUERY_NESTED_COL);

    println!("Done");
}

fn eval_all(data_dir_path: &str, final_dir_path: &str, query_data_csv: &str, cutoff: usize) {
    let json_path = extract_path(data_dir_path, query_data_csv);
    println!("Cutoff = {cutoff}");

    let file = fs::File::open(&json_path).expect("Fail");
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
