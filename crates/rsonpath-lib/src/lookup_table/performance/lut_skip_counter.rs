use std::io::{BufReader, Read};

use crate::{
    engine::{skip_tracker, Engine},
    lookup_table::LUT,
};

use super::{
    lut_query_data::{QUERY_BESTBUY, QUERY_GOOGLE, QUERY_TWITTER},
    lut_skip_evaluation::{get_filename, SkipMode},
};

use crate::lookup_table::performance::lut_query_data::*;
use crate::lookup_table::SKIP_MODE;
use crate::{
    engine::{Compiler, RsonpathEngine},
    input::OwnedBytes,
    lookup_table::LookUpTable,
};
use std::fs;

pub const COUNTER_FILE_PATH: &str = ".a_lut_tests/performance/skip_tracker/COUNTER_";

// Make sure SkipMode==COUNT
// run with: cargo run --bin lut --release -- skip-count
#[inline]
pub fn track_skips() {
    // Skipping must be enabled!
    if !(SKIP_MODE == SkipMode::OFF) {
        println!("Skip tracking mode={:?}", SKIP_MODE);
    } else {
        println!("No tracking set. Abort.");
    }

    // Input
    let cutoff = 0;

    // kB_1
    track_skip_count(QUERY_ALPHABET, cutoff);
    track_skip_count(QUERY_BUGS, cutoff);
    track_skip_count(QUERY_BUGS_2, cutoff);
    track_skip_count(QUERY_JOHN, cutoff);
    track_skip_count(QUERY_JOHN_BIG, cutoff);
    track_skip_count(QUERY_NUMBERS, cutoff);

    // MB_1
    track_skip_count(QUERY_CANADA, cutoff);
    track_skip_count(QUERY_OPENFOOD, cutoff);
    track_skip_count(QUERY_PEOPLE, cutoff);
    track_skip_count(QUERY_PRETTY_PEOPLE, cutoff);
    track_skip_count(QUERY_TWITTER_MINI, cutoff);
    track_skip_count(QUERY_POKEMON_MINI, cutoff);

    // MB_15
    track_skip_count(QUERY_AST, cutoff);
    track_skip_count(QUERY_DUMMY_10, cutoff);
    track_skip_count(QUERY_DUMMY_20, cutoff);

    // MB_100
    track_skip_count(QUERY_APP, cutoff);
    track_skip_count(QUERY_BESTBUY_SHORT, cutoff);
    track_skip_count(QUERY_CROSSREF0, cutoff);
    track_skip_count(QUERY_GOOGLE_SHORT, cutoff);
    track_skip_count(QUERY_POKEMON, cutoff);
    track_skip_count(QUERY_TWITTER_SHORT, cutoff);
    track_skip_count(QUERY_WALMART_SHORT, cutoff);

    // GB_1
    track_skip_count(QUERY_BESTBUY, cutoff);
    track_skip_count(QUERY_CROSSREF1, cutoff);
    track_skip_count(QUERY_CROSSREF2, cutoff);
    track_skip_count(QUERY_CROSSREF4, cutoff);
    track_skip_count(QUERY_GOOGLE, cutoff);
    track_skip_count(QUERY_NSPL, cutoff);
    track_skip_count(QUERY_TWITTER, cutoff);
    track_skip_count(QUERY_WALMART, cutoff);
    track_skip_count(QUERY_WIKI, cutoff);

    // GB_25
    track_skip_count(QUERY_NESTED_COL, cutoff);
}

fn track_skip_count(test_data: (&str, &[(&str, &str)]), cutoff: usize) {
    let (json_name, queries) = test_data;
    let json_path = format!(".a_test_data/{}", json_name);
    println!("json_path: {}", json_path);

    // Build LUT with set cutoff
    let mut lut = LUT::build(&json_path, cutoff).expect("Fail @ building LUT");

    for &(query_id, query_text) in queries {
        let new_lut = track(lut, &json_path, query_id, query_text, cutoff);
        lut = new_lut;
    }
}

fn track(lut: LUT, json_path: &str, query_id: &str, query_text: &str, cutoff: usize) -> LUT {
    print!("\tQuery {} = {}", query_id, query_text);

    // Build query
    let query = rsonpath_syntax::parse(query_text).expect("Fail @ parse query");
    let mut engine = RsonpathEngine::compile_query(&query).expect("Fail @ compile query");
    engine.add_lut(lut);

    // Get result while tracking skips
    let input = {
        let mut file = BufReader::new(fs::File::open(json_path).expect("Fail @ open File"));
        let mut buf = vec![];
        file.read_to_end(&mut buf).expect("Fail @ file read");
        OwnedBytes::new(buf)
    };
    let result = engine.count(&input).expect("Failed to run query normally");
    print!("Result={} ", result);

    // Saved tracked skips
    let filename = get_filename(json_path);
    if SKIP_MODE == SkipMode::COUNT {
        let csv_path = format!("{}{}.csv", COUNTER_FILE_PATH, filename);
        _ = skip_tracker::save_count_to_csv(json_path, &csv_path, filename, query_id, query_text);
        println!("Saved={}", csv_path);
    } else if SKIP_MODE == SkipMode::TRACK {
        // Save the tracked skips to a csv
        let csv_path = format!(
            ".a_lut_tests/performance/distance_distribution_per_query/{}_query={}_cutoff={}.csv",
            filename, query_id, cutoff
        );
        let save_result = skip_tracker::save_track_to_csv(&csv_path);
        println!("Saved={}", csv_path);
        if let Err(e) = save_result {
            eprintln!("Failed to save to CSV: {}", e);
        }
    }

    engine.take_lut().expect("Failed to retrieve LUT from engine")
}
