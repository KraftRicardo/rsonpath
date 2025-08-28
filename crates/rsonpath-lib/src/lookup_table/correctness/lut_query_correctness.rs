use crate::{
    engine::{Compiler, Engine, RsonpathEngine},
    input::OwnedBytes,
    lookup_table::{LookUpTable, LUT},
};

use crate::lookup_table::speed::query_data::*;
use rsonpath_lib_ref::engine::{Compiler as CompilerLegacy, Engine as EngineLegacy};
use std::{
    fs,
    io::{BufReader, Read},
};

/// Run with: cargo run --bin lut --release -- test-query-correctness res/json
///
/// Compares the rq-lut vs. the rq-legacy implementation whether they have the same results. It is
/// a rather unclean test, but it can already cover the difference between the original and the lut
/// adaption.
#[inline]
pub fn run(data_dir_path: &str) {
    let cutoff = 0;

    // GB_1 - COUNT
    test_query_correctness_count(data_dir_path, QUERY_BESTBUY, cutoff);
    test_query_correctness_count(data_dir_path, QUERY_CROSSREF1, cutoff);
    test_query_correctness_count(data_dir_path, QUERY_CROSSREF2, cutoff);
    test_query_correctness_count(data_dir_path, QUERY_CROSSREF4, cutoff);
    test_query_correctness_count(data_dir_path, QUERY_GOOGLE, cutoff);
    test_query_correctness_count(data_dir_path, QUERY_NSPL, cutoff);
    test_query_correctness_count(data_dir_path, QUERY_TWITTER, cutoff);
    test_query_correctness_count(data_dir_path, QUERY_TWITTER_SCALED, cutoff);
    test_query_correctness_count(data_dir_path, QUERY_WALMART, cutoff);
    test_query_correctness_count(data_dir_path, QUERY_WALMART_SCALED, cutoff);
    test_query_correctness_count(data_dir_path, QUERY_WIKI, cutoff);
    test_query_correctness_count(data_dir_path, QUERY_WIKI_SCALED, cutoff);

    // GB_1 - NODES
    test_query_correctness_nodes(data_dir_path, QUERY_BESTBUY, cutoff);
    test_query_correctness_nodes(data_dir_path, QUERY_CROSSREF1, cutoff);
    test_query_correctness_nodes(data_dir_path, QUERY_CROSSREF2, cutoff);
    test_query_correctness_nodes(data_dir_path, QUERY_CROSSREF4, cutoff);
    test_query_correctness_nodes(data_dir_path, QUERY_GOOGLE, cutoff);
    test_query_correctness_nodes(data_dir_path, QUERY_NSPL, cutoff);
    test_query_correctness_nodes(data_dir_path, QUERY_TWITTER, cutoff);
    test_query_correctness_nodes(data_dir_path, QUERY_TWITTER_SCALED, cutoff);
    test_query_correctness_nodes(data_dir_path, QUERY_WALMART, cutoff);
    test_query_correctness_nodes(data_dir_path, QUERY_WALMART_SCALED, cutoff);
    test_query_correctness_nodes(data_dir_path, QUERY_WIKI, cutoff);
    test_query_correctness_nodes(data_dir_path, QUERY_WIKI_SCALED, cutoff);
}

/// Compares the rq-lut vs. rq-legacy implementation whether they have the same COUNT results.
/// Will also trigger when the query has COUNT=0 because then it makes no sense to do tests with it.
fn test_query_correctness_count(data_dir_path: &str, query_data_csv: &str, cutoff: usize) {
    let (json_path, _, queries) = extract_input(data_dir_path, query_data_csv);

    println!("Building LUT...");
    let mut lut = LUT::build(&json_path, cutoff).expect("Fail @ building LUT");

    // Run all queries
    println!("Checking queries:");

    let input = {
        let mut file = BufReader::new(fs::File::open(&json_path).expect("Fail @ open File"));
        let mut buf = vec![];
        file.read_to_end(&mut buf).expect("Fail @ file read");
        OwnedBytes::new(buf)
    };

    for (query_name, query_text) in queries {
        println!(" Query {}: \"{}\" ... ", query_name, query_text);

        // ITE (LEGACY)
        let legacy_input = {
            let mut file = BufReader::new(fs::File::open(&json_path).expect("Fail @ open File"));
            let mut buf = vec![];
            file.read_to_end(&mut buf).expect("Fail @ file read");
            rsonpath_lib_ref::input::OwnedBytes::new(buf)
        };

        let legacy_query = syntax_ref::parse(&query_text).expect("Fail @ parse query");
        let legacy_engine = rsonpath_lib_ref::engine::RsonpathEngine::compile_query(&legacy_query)
            .expect("Fail @ compile query legacy");
        let legacy_count = legacy_engine
            .count(&legacy_input)
            .expect("Failed to run query normally");

        // println!("---- ITE STYLE ----");
        let query = rsonpath_syntax::parse(&query_text).expect("Fail @ parse query");
        let mut engine = RsonpathEngine::compile_query(&query).expect("Fail @ compile query");
        let count = engine.count(&input).expect("Failed to run query normally");

        // println!("---- LUT STYLE ----");
        engine.add_lut(lut);
        let lut_count = engine.count(&input).expect("LUT: Failed to run query normally");

        if legacy_count != count {
            println!("\tITE INCORRECT: Found {}, Expected {}", lut_count, count);
        }
        if legacy_count != lut_count {
            println!("\tLUT INCORRECT: Found {}, Expected {}", lut_count, count);
        }

        if legacy_count == 0 {
            println!("\tDO NOT USE THIS QUERY. IT HAS NO RESULTS!")
        }

        lut = engine.take_lut().expect("Failed to retrieve LUT from engine");
    }

    drop(lut);
}

fn test_query_correctness_nodes(data_dir_path: &str, query_data_csv: &str, cutoff: usize) {
    let (json_path, _, queries) = extract_input(data_dir_path, query_data_csv);

    println!("Building LUT");
    let mut lut = LUT::build(&json_path, cutoff).expect("Fail @ building LUT");

    // Run all queries
    println!("Checking queries:");
    for (query_name, query_text) in queries {
        println!(" Query: {} = \"{}\" ... ", query_name, query_text);
        let input = {
            let mut file = BufReader::new(fs::File::open(&json_path).expect("Fail @ open File"));
            let mut buf = vec![];
            file.read_to_end(&mut buf).expect("Fail @ file read");
            OwnedBytes::new(buf)
        };
        let query = rsonpath_syntax::parse(&query_text).expect("Fail @ parse query");

        // Query normally and skip iteratively (ITE)
        println!("---- ITE STYLE ----");
        let mut engine = RsonpathEngine::compile_query(&query).expect("Fail @ compile query");
        let mut sink = vec![];
        engine.matches(&input, &mut sink).expect("Fail @ engine matching.");
        let results = sink
            .into_iter()
            .map(|m| String::from_utf8_lossy(m.bytes()).to_string())
            .collect::<Vec<_>>();

        // Print results
        println!("ITE Results found: ");
        for (i, result) in results.into_iter().enumerate() {
            println!("Result {}:", i);
            println!("{result}");
        }

        // Query normally and skip using the lookup table (LUT)
        println!("---- LUT STYLE ----");
        engine.add_lut(lut);
        let mut sink_lut = vec![];
        engine.matches(&input, &mut sink_lut).expect("Fail @ engine matching.");
        let results_lut = sink_lut
            .into_iter()
            .map(|m| String::from_utf8_lossy(m.bytes()).to_string())
            .collect::<Vec<_>>();

        println!("LUT Results found: ");
        for (i, result) in results_lut.into_iter().enumerate() {
            println!("Result {}:", i);
            println!("{result}");
        }

        lut = engine.take_lut().expect("Failed to retrieve LUT from engine");
    }

    drop(lut);
}
