use crate::lookup_table::speed::lut_skip_evaluation::SkipMode::OFF;
use crate::lookup_table::{LookUpTable, LUT, QUERY_REPETITIONS, SKIP_MODE, TRACK_SKIPPING_ON};
use crate::{
    engine::{Compiler, Engine, RsonpathEngine},
    input::OwnedBytes,
};
use std::fs;
use std::io::{BufReader, Read};
use std::time::Instant;

// Run with: cargo run --bin lut --release -- eval-valgrind
pub fn run() {
    println!("eval-valgrind");

    if TRACK_SKIPPING_ON || SKIP_MODE != OFF {
        println!("Disable tracking of skips before running because it slows down the algorithm.");
        return;
    }
    if !cfg! {feature = "empty-list-opt"} {
        println!("Turn the empty-list-opt feature for better performance!");
        return;
    }

    let data_dir_path = "res/json";

    let cutoff = 0;
    let json_twitter = "twitter_large_record_(843MB).json";
    let query_twitter_17 = "$[*].user.profile_sidebar_border_color";
    let query_twitter_19 = "$[*].retweeted_status.filter_level";

    eval(json_twitter, query_twitter_17, data_dir_path, cutoff);

    println!("Done");
}

fn eval(json_name: &str, query_text: &str, data_dir_path: &str, cutoff: usize) {
    let json_path = format!("{data_dir_path}/{json_name}");

    // Build LUT
    let lut = LUT::build(&json_path, cutoff).expect("Failed to build LUT");

    // Build input
    let input = {
        let mut buf = vec![];
        let mut file = BufReader::new(fs::File::open(&json_path).expect("Failed to open input file"));
        file.read_to_end(&mut buf).expect("Failed to read input file");
        OwnedBytes::new(buf)
    };

    // Build engine and query
    let query = rsonpath_syntax::parse(query_text).expect("Failed to parse query");
    let mut engine = RsonpathEngine::compile_query(&query).expect("Failed to compile query");
    engine.add_lut(lut);

    let mut result = 0;
    for _ in 0..QUERY_REPETITIONS {
        result = engine.count(&input).expect("Query execution failed");
    }

    // let mut total_time = 0.0;
    // for _ in 0..QUERY_REPETITIONS {
    //     let start = Instant::now();
    //     result = engine.count(&input).expect("Query execution failed");
    //     total_time += start.elapsed().as_secs_f64();
    // }
    // let avg_time = total_time / (QUERY_REPETITIONS as f64);
    //
    // println!(
    //     "  - File: {}, Cutoff: {}, Query: {}, Time = {:.5}s, Result = {}",
    //     json_name, cutoff, query_text, avg_time, result
    // );
}
