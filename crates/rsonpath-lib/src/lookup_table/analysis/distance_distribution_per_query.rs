use std::io::{BufReader, Read};

use crate::{
    engine::{skip_tracker, Engine},
    lookup_table::LUT,
};

use crate::lookup_table::speed::lut_query_data;
use crate::lookup_table::speed::lut_query_data::*;
use crate::lookup_table::speed::lut_skip_evaluation::SkipMode::{COUNT, TRACK};
use crate::lookup_table::speed::lut_skip_evaluation::{get_filename, SkipMode};
use crate::lookup_table::{QUERY_REPETITIONS, SKIP_MODE, TRACK_SKIPPING_ON};
use crate::result::MatchCount;
use crate::{
    engine::{Compiler, RsonpathEngine},
    input::OwnedBytes,
    lookup_table::LookUpTable,
};
use std::fs;
use SkipMode::TRACK_TIMED;

pub const COUNTER_FILE_PATH: &str = ".a_lut_tests/performance/skip_tracker/COUNTER_";

// Make sure SkipMode==COUNT
// Run with: cargo run --bin lut --release -- analyse-distance-distribution-per-query res/json res/data/analysis/distance_distribution_per_query
#[inline]
pub fn run(json_dir_path: &str, base_path: &str) {
    // Input
    let cutoff = 0;

    // Abort conditions
    if !TRACK_SKIPPING_ON {
        println!("TRACK_SKIPPING_ON = FALSE, so abort. Set it to TRUE if you want this to work.");
        return;
    }
    if cfg! {feature = "empty-list-opt"} {
        println!("Turn off the empty-list-opt feature to count all skips!");
        return;
    }

    // Create result directory
    let result_dir_path: String;

    if SKIP_MODE == COUNT {
        println!("Skip mode = COUNT");
        result_dir_path = format!("{base_path}/count");
    } else if SKIP_MODE == TRACK {
        println!("Skip mode = TRACK");
        result_dir_path = format!("{base_path}/track/cutoff={cutoff}");
    } else if SKIP_MODE == TRACK_TIMED {
        println!("Skip mode = TRACK_TIMED");
        result_dir_path = format!("{base_path}/track_timed/cutoff={cutoff}");
    } else {
        println!("No tracking set. Abort.");
        return;
    }

    fs::create_dir_all(&result_dir_path).expect("Fail at creating result folder.");

    // kB_1
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_BUGS, cutoff);
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_BUGS_2, cutoff);
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_JOHN, cutoff);
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_JOHN_BIG, cutoff);
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_NUMBERS, cutoff);

    // // MB_1
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_CANADA, cutoff);
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_OPENFOOD, cutoff);
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_PEOPLE, cutoff);
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_PRETTY_PEOPLE, cutoff);
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_TWITTER_MINI, cutoff);
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_POKEMON_MINI, cutoff);
    //
    // // MB_15
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_AST, cutoff);
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_DUMMY_10, cutoff);
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_DUMMY_20, cutoff);
    //
    // // MB_100
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_APP, cutoff);
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_BESTBUY_SHORT, cutoff);
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_CROSSREF0, cutoff);
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_GOOGLE_SHORT, cutoff);
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_POKEMON, cutoff);
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_TWITTER_SHORT, cutoff);
    // track_skip_count(json_dir_path, &result_dir_path, QUERY_WALMART_SHORT, cutoff);

    // DEBUG
    track(json_dir_path, &result_dir_path, QUERY_NSPL_MINI, cutoff);

    // GB_1
    // track(json_dir_path, &result_dir_path, QUERY_BESTBUY, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_CROSSREF1, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_CROSSREF2, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_CROSSREF4, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_GOOGLE, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_NSPL, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_TWITTER, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_TWITTER_SCALED, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_WALMART, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_WALMART_SCALED, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_WIKI, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_WIKI_SCALED, cutoff);

    // GB_25
    // track(json_dir_path, &result_dir_path, QUERY_NESTED_COL, cutoff);
}

fn track(json_dir_path: &str, result_dir_path: &str, query_data: &str, cutoff: usize) {
    let (json_name, queries) = read_queries(query_data);
    let json_path = format!("{}/{}", json_dir_path, json_name);
    println!("json_path: {}", json_path);

    // Build LUT with set cutoff
    let mut lut = LUT::build(&json_path, cutoff).expect("Fail @ building LUT");

    for (query_id, query_text) in queries {
        if SKIP_MODE == COUNT || SKIP_MODE == TRACK {
            let new_lut = track_count(lut, &json_path, result_dir_path, &query_id, &query_text);
            lut = new_lut;
        } else if SKIP_MODE == TRACK_TIMED {
            track_timed(&json_path, result_dir_path, &query_id, &query_text);
        }
    }
}

fn track_count(lut: LUT, json_path: &str, result_dir_path: &str, query_id: &str, query_text: &str) -> LUT {
    print!("\tQuery {} = {}", query_id, query_text);

    // Build query and engine with LUT
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
    print!(" Result={} ", result);

    // Save to csv
    let filename = get_filename(json_path);
    if SKIP_MODE == COUNT {
        let csv_path = format!("{result_dir_path}/COUNTER_{filename}.csv");
        _ = skip_tracker::save_count_to_csv(json_path, &csv_path, filename, query_id, query_text);
        skip_tracker::reset();
        // println!("Write to: {}", csv_path);
    } else if SKIP_MODE == TRACK {
        let csv_path = format!("{result_dir_path}/{filename}_query={query_id}.csv");
        _ = skip_tracker::save_track_to_csv(&csv_path);
        skip_tracker::reset();
        // println!("Write to: {}", csv_path);
    }

    engine.take_lut().expect("Failed to retrieve LUT from engine")
}

fn track_timed(json_path: &str, result_dir_path: &str, query_id: &str, query_text: &str) {
    print!("\tQuery {} = {}", query_id, query_text);

    // Build query
    let query = rsonpath_syntax::parse(query_text).expect("Fail @ parse query");
    let mut engine = RsonpathEngine::compile_query(&query).expect("Fail @ compile query");

    // Get result while tracking skips
    let input = {
        let mut file = BufReader::new(fs::File::open(json_path).expect("Fail @ open File"));
        let mut buf = vec![];
        file.read_to_end(&mut buf).expect("Fail @ file read");
        OwnedBytes::new(buf)
    };

    let mut result = 0;
    for _ in 0..QUERY_REPETITIONS {
        result = engine.count(&input).expect("Failed to run query normally");
    }
    print!("Result={} ", result);

    // Save to csv
    let filename = get_filename(json_path);
    let csv_path = format!("{result_dir_path}/{filename}_query={query_id}.csv");
    _ = skip_tracker::save_track_timed_to_csv(&csv_path);
    skip_tracker::reset();
    println!("Write to: {}", csv_path);
}
