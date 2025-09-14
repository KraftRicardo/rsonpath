use std::io::{BufReader, Read};

use crate::{
    engine::{skip_tracker, Engine},
    lookup_table::LUT,
};

use crate::lookup_table::extra::util_path::get_filename;
use crate::lookup_table::speed::query_data::*;
use crate::lookup_table::SkipMode::{COUNT, TRACK, TRACKTIMED};
use crate::lookup_table::{QUERY_REPETITIONS, SKIP_MODE};
use crate::{
    engine::{Compiler, RsonpathEngine},
    input::OwnedBytes,
    lookup_table::LookUpTable,
};
use std::fs;

/// Runs the analysis of distance distributions per query.
///
/// This function executes all queries in the query set according to the selected mode:
/// - **COUNT**: Tracks how many jumps occur.
/// - **TRACK**: Tracks each jump distance individually in a data structure (slower).
/// - **TRACKTIMED**: Tracks each jump distance individually and also measures the time per jump (slowest).
///
/// Results are written to the specified output directory.
///
/// # Arguments
/// * `json_dir_path` - Path to the folder containing the JSON files.
/// * `base_path` - Path where the analysis results will be stored.
/// * `cutoff` - Cutoff value used by the LUT implementation.
///
/// # Example
/// ```bash
/// cargo run --bin lut --release -- analyse-distance-distribution-per-query res/json res/data/analysis/distance_distribution_per_query 0
/// ```
#[inline]
pub fn analyse_distance_distribution_per_query(json_dir_path: &str, base_path: &str, cutoff: usize) {
    // Abort conditions
    if !cfg! {feature = "track-skipping"} {
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
        result_dir_path = format!("{base_path}/count/cutoff={cutoff}");
    } else if SKIP_MODE == TRACK {
        println!("Skip mode = TRACK");
        result_dir_path = format!("{base_path}/track/cutoff={cutoff}");
    } else if SKIP_MODE == TRACKTIMED {
        println!("Skip mode = TRACK_TIMED");
        result_dir_path = format!("{base_path}/track_timed/cutoff={cutoff}");
    } else {
        println!("No tracking set. Abort.");
        return;
    }

    fs::create_dir_all(&result_dir_path).expect("Fail at creating result folder.");

    // DEBUG
    // track(json_dir_path, &result_dir_path, QUERY_NSPL_MINI, cutoff);

    // ##########
    // 1 kB
    // ##########
    // track(json_dir_path, &result_dir_path, QUERY_ALPHABET, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_JOHN, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_NUMBERS, cutoff);

    // ##########
    // 1 MB
    // ##########
    // track(json_dir_path, &result_dir_path, QUERY_CANADA, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_OPENFOOD, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_PEOPLE, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_PRETTY_PEOPLE, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_TWITTER_MINI, cutoff);

    // ##########
    // 15 MB
    // ##########
    // track(json_dir_path, &result_dir_path, QUERY_AST, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_DUMMY_10, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_DUMMY_20, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_POKEMON_MINI, cutoff);

    // ##########
    // 100 MB
    // ##########
    // track(json_dir_path, &result_dir_path, QUERY_APP, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_POKEMON, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_BESTBUY_SHORT, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_CROSSREF0, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_GOOGLE_SHORT, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_TWITTER_SHORT, cutoff);
    // track(json_dir_path, &result_dir_path, QUERY_WALMART_SHORT, cutoff);

    // ##########
    // 1 GB
    // ##########
    track(json_dir_path, &result_dir_path, QUERY_BESTBUY, cutoff);
    track(json_dir_path, &result_dir_path, QUERY_CROSSREF1, cutoff);
    track(json_dir_path, &result_dir_path, QUERY_CROSSREF2, cutoff);
    track(json_dir_path, &result_dir_path, QUERY_CROSSREF4, cutoff);
    track(json_dir_path, &result_dir_path, QUERY_GOOGLE, cutoff);
    track(json_dir_path, &result_dir_path, QUERY_NSPL, cutoff);
    track(json_dir_path, &result_dir_path, QUERY_TWITTER, cutoff);
    track(json_dir_path, &result_dir_path, QUERY_TWITTER_SINGLE, cutoff);
    track(json_dir_path, &result_dir_path, QUERY_WALMART, cutoff);
    track(json_dir_path, &result_dir_path, QUERY_WALMART_SINGLE, cutoff);
    track(json_dir_path, &result_dir_path, QUERY_WIKI, cutoff);
    track(json_dir_path, &result_dir_path, QUERY_WIKI_SINGLE, cutoff);

    // ##########
    // 25 GB
    // ##########
    // track(json_dir_path, &result_dir_path, QUERY_NESTED_COL, cutoff);
}

fn track(data_dir_path: &str, result_dir_path: &str, query_data_csv: &str, cutoff: usize) {
    let (json_path, _, queries) = extract_input(data_dir_path, query_data_csv);

    // Build LUT with set cutoff
    let mut lut = LUT::build(&json_path, cutoff).expect("Fail @ building LUT");

    for (query_id, query_text) in queries {
        if SKIP_MODE == COUNT || SKIP_MODE == TRACK {
            let new_lut = track_count(lut, &json_path, result_dir_path, &query_id, &query_text);
            lut = new_lut;
        } else if SKIP_MODE == TRACKTIMED {
            track_timed(&json_path, result_dir_path, &query_id, &query_text);
        }
    }
}

fn track_count(lut: LUT, json_path: &str, result_dir_path: &str, query_id: &str, query_text: &str) -> LUT {
    print!("\tQuery {query_id} = {query_text}");

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
    print!(" Result={result} ");

    // Save to csv
    let filename = get_filename(json_path);
    if SKIP_MODE == COUNT {
        let csv_path = format!("{result_dir_path}/COUNTER_{filename}.csv");
        skip_tracker::save_count_to_csv(json_path, &csv_path, filename, query_id, query_text, result, true);
        skip_tracker::reset();
        // println!("Write to: {}", csv_path);
    } else if SKIP_MODE == TRACK {
        // let csv_path = format!("{result_dir_path}/{filename}_query={query_id}.csv");
        let csv_path = format!("{result_dir_path}/{filename}_scaled_query={query_id}.csv");
        _ = skip_tracker::save_track_to_csv(&csv_path);
        println!("."); // Needed so the formatting in the console does not break
        skip_tracker::reset();
        // println!("Write to: {}", csv_path);
    }

    engine.take_lut().expect("Failed to retrieve LUT from engine")
}

fn track_timed(json_path: &str, result_dir_path: &str, query_id: &str, query_text: &str) {
    print!("\tQuery {query_id} = {query_text}");

    // Build query
    let query = rsonpath_syntax::parse(query_text).expect("Fail @ parse query");
    let engine = RsonpathEngine::compile_query(&query).expect("Fail @ compile query");

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
    print!("Result={result} ");

    // Save to csv
    let filename = get_filename(json_path);
    let csv_path = format!("{result_dir_path}/{filename}_query={query_id}.csv");
    // let csv_path = format!("{result_dir_path}/{filename}_scaled_query={query_id}.csv");
    _ = skip_tracker::save_track_timed_to_csv(&csv_path);
    skip_tracker::reset();
    println!("Write to: {csv_path}");
}
