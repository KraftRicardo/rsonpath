use crate::lookup_table::analysis::distance_distribution;
use crate::lookup_table::performance::lut_evaluation::EvalConfig;
use crate::lookup_table::performance::lut_query_data::{
    QUERY_BESTBUY, QUERY_CROSSREF1, QUERY_CROSSREF2, QUERY_CROSSREF4, QUERY_GOOGLE, QUERY_NSPL, QUERY_TWITTER,
    QUERY_WALMART, QUERY_WIKI,
};
use crate::lookup_table::{pair_data, util_path};

pub fn evaluate(data_dir_path: &str, result_dir_path: &str) {
    println!("LUT construction evaluation");

    let results_csv_path = format!("{}/results.csv", result_dir_path);

    // GB_1
    eval(&data_dir_path, &results_csv_path, QUERY_BESTBUY);
    eval(&data_dir_path, &results_csv_path, QUERY_CROSSREF1);
    eval(&data_dir_path, &results_csv_path, QUERY_CROSSREF2);
    eval(&data_dir_path, &results_csv_path, QUERY_CROSSREF4);
    eval(&data_dir_path, &results_csv_path, QUERY_GOOGLE);
    eval(&data_dir_path, &results_csv_path, QUERY_NSPL);
    eval(&data_dir_path, &results_csv_path, QUERY_TWITTER);
    eval(&data_dir_path, &results_csv_path, QUERY_WALMART);
    eval(&data_dir_path, &results_csv_path, QUERY_WIKI);
}

pub fn eval(
    data_dir_path: &str,
    result_csv_path: &str,
    test_data: (&str, &[(&str, &str)]),
) -> Result<(), Box<dyn std::error::Error>> {
    // // Extract input
    // let (json_filename, queries) = test_data;
    // let filename = json_filename.strip_suffix(".json").unwrap();
    // println!("JSON: {}", filename);
    //
    // let json_path = format!("{}/{}.json", data_dir_path, filename);
    //
    // let file = std::fs::File::open(&json_path)?;
    // let num_keys = distance_distribution::count_num_pairs(&json_path);
    //
    // let mut head_line = String::from("name,input_size_bytes,num_keys,");
    // let mut data_line = format!("{},{},{},", filename, file.metadata()?.len(), num_keys);
    //
    // let cutoff: usize = 0;
    // println!("JSONPATH: {}, cutoff = {}", json_path, cutoff);
    //
    // let (keys, _) = pair_data::get_pairs_absolute(&json_path, cutoff).expect("Fail @ finding pairs.");
    //
    // let mut config = EvalConfig {
    //     json_path,
    //     keys,
    //     head_line: &mut head_line,
    //     data_line: &mut data_line,
    // };
    //
    // measure_performance(&mut config, cutoff)?;
    //
    // // Write CSV header and data
    // let mut csv_file = std::fs::OpenOptions::new().append(true).create(true).open(csv_path)?;
    // if csv_file.metadata()?.len() == 0 {
    //     writeln!(csv_file, "{}", head_line)?;
    // }
    // writeln!(csv_file, "{}", data_line)?;
    //

    Ok(())
}
