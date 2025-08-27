use crate::lookup_table::{LookUpTable, LUT};

use crate::lookup_table::luts::{lut_hash_map, pair_data};
use crate::lookup_table::speed::lut_query_data::*;

/// Checks whether the distances covered by the LUT are actually correct.
///
/// Run with: cargo run --bin lut --release -- test-build-correctness res/json
#[inline]
pub fn run(data_dir_path: &str) {
    let cutoff = 0;

    // GB_1
    test_build_correctness(data_dir_path, QUERY_BESTBUY, cutoff);
    test_build_correctness(data_dir_path, QUERY_CROSSREF1, cutoff);
    test_build_correctness(data_dir_path, QUERY_CROSSREF2, cutoff);
    test_build_correctness(data_dir_path, QUERY_CROSSREF4, cutoff);
    test_build_correctness(data_dir_path, QUERY_GOOGLE, cutoff);
    test_build_correctness(data_dir_path, QUERY_NSPL, cutoff);
    test_build_correctness(data_dir_path, QUERY_TWITTER, cutoff);
    test_build_correctness(data_dir_path, QUERY_WALMART, cutoff);
    test_build_correctness(data_dir_path, QUERY_WIKI, cutoff);
}

fn test_build_correctness(data_dir_path: &str, query_data_csv: &str, cutoff: usize) {
    let (json_path, _, _) = extract_input(data_dir_path, query_data_csv);

    println!("Building LUT...");
    let lut = LUT::build(&json_path, cutoff).expect("Fail @ building LUT");
    let lut_hash_map = lut_hash_map::LutHashMap::build(&json_path, cutoff).expect("Fail @ building lut_hash_map");

    print!("Testing keys:");
    let (keys, values) = pair_data::get_pairs_absolute(&json_path, cutoff).expect("Fail @ finding pairs.");
    let mut count_incorrect = 0;
    for (i, key) in keys.iter().enumerate() {
        let found = lut.get(key).expect("Fail @ get(key) - LUT.");
        let found_hash = lut_hash_map.get(key).expect("Fail @ get(key) - lut_hash_map.");
        if found != values[i] {
            count_incorrect += 1;
            println!(
                "  i: {}, Key {}, Found {}, Expected: {} and {}",
                i, key, found, values[i], found_hash
            );
        }
    }

    print!(" Correct {}/{}", keys.len() - count_incorrect, keys.len());
    println!(" Incorrect {}/{}", count_incorrect, keys.len());

    drop(lut);
}
