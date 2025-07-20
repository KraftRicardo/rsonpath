use std::io::{BufReader, Read};

use crate::{
    engine::{skip_tracker, Engine},
    lookup_table::LUT,
};

use crate::lookup_table::performance::lut_query_data::*;
use crate::lookup_table::performance::lut_skip_evaluation::{get_filename, SkipMode};
use crate::lookup_table::{SKIP_MODE, TRACK_SKIPPING_ON};
use crate::{
    engine::{Compiler, RsonpathEngine},
    input::OwnedBytes,
    lookup_table::LookUpTable,
};
use std::fs;
use std::path::Path;

pub const COUNTER_FILE_PATH: &str = ".a_lut_tests/performance/skip_tracker/COUNTER_";

// Make sure SkipMode==COUNT
// Run with: cargo run --bin lut --release -- analyse-distance-distribution-per-query res/json res/data/analysis/distance_distribution_per_query
// Run with: cargo run --bin lut --release -- analyse-distance-distribution-per-query res/json res/data/analysis/skip_counter

// TODO: should I create a sub category for when the empty-list-opt is enabled? no because I can also track there
#[inline]
pub fn analyse_distance_distribution_per_query(json_dir_path: &str, result_dir_path: &str) {
    if !TRACK_SKIPPING_ON {
        println!("TRACK_SKIPPING_ON = FALSE, so abort. Set it to TRUE if you want this to work.");
        return;
    }
    if SKIP_MODE == SkipMode::OFF {
        println!("No tracking set. Abort.");
        return;
    }
    // if SKIP_MODE == SkipMode::COUNT {
    //
    // }

    // Input
    let cutoff = 0;

    if SKIP_MODE == SkipMode::TRACK {
        let csv_dir_path = format!("{result_dir_path}/cutoff={cutoff}");
        let path = Path::new(&csv_dir_path);
        fs::create_dir_all(path).expect("Fail at creating result folder.");
    }

    // kB_1
    // track_skip_count(json_dir_path, result_dir_path, QUERY_BUGS, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_BUGS_2, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_JOHN, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_JOHN_BIG, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_NUMBERS, cutoff);

    // // MB_1
    // track_skip_count(json_dir_path, result_dir_path, QUERY_CANADA, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_OPENFOOD, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_PEOPLE, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_PRETTY_PEOPLE, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_TWITTER_MINI, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_POKEMON_MINI, cutoff);
    //
    // // MB_15
    // track_skip_count(json_dir_path, result_dir_path, QUERY_AST, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_DUMMY_10, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_DUMMY_20, cutoff);
    //
    // // MB_100
    // track_skip_count(json_dir_path, result_dir_path, QUERY_APP, cutoff);
    track_skip_count(json_dir_path, result_dir_path, QUERY_BESTBUY_SHORT, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_CROSSREF0, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_GOOGLE_SHORT, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_POKEMON, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_TWITTER_SHORT, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_WALMART_SHORT, cutoff);
    //
    // // GB_1
    // track_skip_count(json_dir_path, result_dir_path, QUERY_BESTBUY, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_CROSSREF1, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_CROSSREF2, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_CROSSREF4, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_GOOGLE, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_NSPL, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_TWITTER, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_WALMART, cutoff);
    // track_skip_count(json_dir_path, result_dir_path, QUERY_WIKI, cutoff);
    //
    // // GB_25
    // track_skip_count(json_dir_path, result_dir_path, QUERY_NESTED_COL, cutoff);
}

fn track_skip_count(json_dir_path: &str, result_dir_path: &str, test_data: (&str, &[(&str, &str)]), cutoff: usize) {
    let (json_name, queries) = test_data;
    let json_path = format!("{}/{}", json_dir_path, json_name);
    println!("json_path: {}", json_path);

    // Build LUT with set cutoff
    let mut lut = LUT::build(&json_path, cutoff).expect("Fail @ building LUT");

    for &(query_id, query_text) in queries {
        let new_lut = track(lut, &json_path, result_dir_path, query_id, query_text, cutoff);
        lut = new_lut;
    }
}

fn track(lut: LUT, json_path: &str, result_dir_path: &str, query_id: &str, query_text: &str, cutoff: usize) -> LUT {
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

    // Save to csv
    let filename = get_filename(json_path);
    if SKIP_MODE == SkipMode::COUNT {
        let csv_path = format!("{result_dir_path}/COUNTER_{filename}.csv");
        _ = skip_tracker::save_count_to_csv(json_path, &csv_path, filename, query_id, query_text);
        println!("Generated: {}", csv_path);
    } else if SKIP_MODE == SkipMode::TRACK {
        let csv_path = format!("{result_dir_path}/cutoff={cutoff}/{filename}_query={query_id}.csv");
        _ = skip_tracker::save_track_to_csv(&csv_path);
        println!("Generated: {}", csv_path);
    }

    engine.take_lut().expect("Failed to retrieve LUT from engine")
}
