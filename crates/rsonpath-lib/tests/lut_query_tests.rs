use log::debug;
use rsonpath::lookup_table::speed::query_data::*;
use rsonpath::{
    engine::{Compiler, Engine, RsonpathEngine},
    input::OwnedBytes,
    lookup_table::{LookUpTable, LUT},
};
use std::{
    error::Error,
    fs,
    io::{BufReader, Read},
};

#[test]
fn query_bestbuy() -> Result<(), Box<dyn Error>> {
    compare_rq_lut_vs_rq_legacy(QUERY_BESTBUY)
}

#[test]
fn query_crossref1() -> Result<(), Box<dyn Error>> {
    compare_rq_lut_vs_rq_legacy(QUERY_CROSSREF1)
}

#[test]
fn query_crossref2() -> Result<(), Box<dyn Error>> {
    compare_rq_lut_vs_rq_legacy(QUERY_CROSSREF2)
}

#[test]
fn query_crossref4() -> Result<(), Box<dyn Error>> {
    compare_rq_lut_vs_rq_legacy(QUERY_CROSSREF4)
}

#[test]
fn query_google() -> Result<(), Box<dyn Error>> {
    compare_rq_lut_vs_rq_legacy(QUERY_GOOGLE)
}

#[test]
fn query_nspl() -> Result<(), Box<dyn Error>> {
    compare_rq_lut_vs_rq_legacy(QUERY_NSPL)
}

#[test]
fn query_twitter() -> Result<(), Box<dyn Error>> {
    compare_rq_lut_vs_rq_legacy(QUERY_TWITTER)
}

#[test]
fn query_twitter_scaled() -> Result<(), Box<dyn Error>> {
    compare_rq_lut_vs_rq_legacy(QUERY_TWITTER_SCALED)
}

#[test]
fn query_walmart() -> Result<(), Box<dyn Error>> {
    compare_rq_lut_vs_rq_legacy(QUERY_WALMART)
}

#[test]
fn query_walmart_scaled() -> Result<(), Box<dyn Error>> {
    compare_rq_lut_vs_rq_legacy(QUERY_WALMART_SCALED)
}

#[test]
fn query_wiki() -> Result<(), Box<dyn Error>> {
    compare_rq_lut_vs_rq_legacy(QUERY_WIKI)
}

#[test]
fn query_wiki_scaled() -> Result<(), Box<dyn Error>> {
    compare_rq_lut_vs_rq_legacy(QUERY_WIKI_SCALED)
}

/// Run all with:
///     cargo test --test lut_query_tests
/// Or run single ones with:
///     cargo test --test lut_query_tests -- query_bestbuy --nocapture | rg "(tail_skipping|lut_query_tests)"
///     cargo test --test lut_query_tests -- query_bestbuy --nocapture | rg "(lut_query_tests)"
fn compare_rq_lut_vs_rq_legacy(query_data: &str) -> Result<(), Box<dyn Error>> {
    let cutoff = 0;

    // Enables to see log messages when running tests
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();
    debug!("Using cutoff {}", cutoff);

    // Build LUT once at the beginning
    let queries = read_queries_test(query_data);
    let json_path = format!("../../res/json/{}", "error1.json");
    debug!("Building LUT: {json_path}, cutoff: {cutoff}");
    let mut lut = LUT::build(&json_path, cutoff).expect("Fail @ building LUT");

    // Run all queries
    for (query_name, query_text) in queries {
        debug!("Query: {}", query_name);

        let input = {
            let mut file = BufReader::new(fs::File::open(&json_path).expect("Fail @ open File"));
            let mut buf = vec![];
            file.read_to_end(&mut buf).expect("Fail @ file read");
            OwnedBytes::new(buf)
        };
        let query = rsonpath_syntax::parse(&query_text).expect("Fail @ parse query");

        // Query normally and skip iteratively (ITE)
        let mut engine = RsonpathEngine::compile_query(&query).expect("Fail @ compile query");
        let result = engine.count(&input).expect("Failed to run query normally");

        // Query normally and skip using the lookup table (LUT)
        engine.add_lut(lut);
        let lut_result = engine.count(&input).expect("LUT: Failed to run query normally");

        assert_eq!(lut_result, result);

        lut = engine.take_lut().expect("Failed to retrieve LUT from engine")
    }

    Ok(())
}

#[test]
fn debug_test_1() -> Result<(), Box<dyn Error>> {
    debug_one_query()
}

/// Examples:
/// cargo test --test lut_query_tests -- debug_test_1 --nocapture | rg "(tail_skipping|lut_query_tests)"
/// cargo test --test lut_query_tests -- debug_test_1 --nocapture | rg "(lut_query_tests)"
/// cargo test --test lut_query_tests -- debug_test_1 --nocapture
///
/// If you need to run on no-simd mode:
/// cargo test --test lut_query_tests --no-default-features --features serde -- debug_test_1 --nocapture | rg "(tail_skipping|lut_query_tests)"
fn debug_one_query() -> Result<(), Box<dyn Error>> {
    let json_path = "../rsonpath-test/documents/json/large/wikidata_properties.json";
    let query = "$..P7103.claims.P31..references..snaks.P4656..hash";
    // let json_path = "../../res/json/1_error.json";
    // let query = "$[1:5: 2]";
    let cutoff = 0;

    // Enables to see log messages when running tests
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();
    debug!("Using cutoff {}", cutoff);

    // Build LUT once at the beginning
    debug!("Building LUT: {json_path}, cutoff: {cutoff}");
    let lut = LUT::build(&json_path, cutoff).expect("Fail @ building LUT");

    // Run
    debug!("Query: {}", query);

    let input = {
        let mut file = BufReader::new(fs::File::open(&json_path).expect("Fail @ open File"));
        let mut buf = vec![];
        file.read_to_end(&mut buf).expect("Fail @ file read");
        OwnedBytes::new(buf)
    };
    let query = rsonpath_syntax::parse(&query).expect("Fail @ parse query");

    // Query normally and skip iteratively (ITE)
    let mut engine = RsonpathEngine::compile_query(&query).expect("Fail @ compile query");
    let result = engine.count(&input).expect("Failed to run query normally");
    debug!("ITE RESULT: {result}");

    // Query normally and skip using the lookup table (LUT)
    engine.add_lut(lut);
    let lut_result = engine.count(&input).expect("LUT: Failed to run query normally");
    debug!("LUT RESULT: {result}");

    assert_eq!(lut_result, result);

    Ok(())
}
