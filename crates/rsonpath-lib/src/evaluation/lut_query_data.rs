use csv::ReaderBuilder;
use std::fs::File;
use std::io::BufReader;

// Expected csv structure:
//      ID,Path,Result,SkipPercentage
//      1,$..freeShipping,230089,0.00000000000000000
//      2,$..additionalFeatures[*].feature,61098,0.00000000000000000
//      3,$..includedItemList[*],9096,0.00000000000000000
//      4,$.products[*].videoChapters,769,0.09227438889806847
//      ...

// Note: Query data with the "_scaled" suffix use just one single query that targets different
// of the JSON
const QUERY_DATA_FOLDER: &str = "../rsonpath/res/query";

pub const QUERY_BESTBUY: &str = "bestbuy_large_record_(1GB)";
pub const QUERY_CROSSREF1: &str = "crossref1_(551MB)";
pub const QUERY_CROSSREF2: &str = "crossref2_(1.1GB)"; // Same queries as crossref1
pub const QUERY_CROSSREF4: &str = "crossref4_(2.1GB)"; // Same queries as crossref1
pub const QUERY_GOOGLE: &str = "google_map_large_record_(1.1GB)";
pub const QUERY_NSPL: &str = "nspl_large_record_(1.2GB)";
pub const QUERY_TWITTER: &str = "twitter_large_record_(843MB)";
pub const QUERY_TWITTER_SCALED: &str = "twitter_large_record_(843MB)_scaled";
pub const QUERY_WALMART: &str = "walmart_large_record_(995MB)";
pub const QUERY_WALMART_SCALED: &str = "walmart_large_record_(995MB)_scaled";
pub const QUERY_WIKI: &str = "wiki_large_record_(1.1GB)";
pub const QUERY_WIKI_SCALED: &str = "wiki_large_record_(1.1GB)_scaled";
pub const QUERY_NESTED_COL: &str = "nested_col_(27.7GB)";

/// Reads the query data from a csv. It extracts the QUERY_ID and QUERY_TEXT field which are
/// expected to be the first 2 columns.
pub fn read_queries(file_name: &str) -> Vec<(String, String)> {
    let csv_path = format!("{QUERY_DATA_FOLDER}/{file_name}.csv");
    let file = File::open(&csv_path).expect("Cannot open CSV file");
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(BufReader::new(file));

    let mut queries = Vec::new();

    for result in rdr.records() {
        let record = result.expect("Failed to parse CSV record");
        let id = record.get(0).expect("Missing ID").to_string();
        let path = record.get(1).expect("Missing path").to_string();
        queries.push((id, path));
    }

    queries
}

pub(crate) fn remove_common_suffix(csv_path: &str) -> String {
    remove_suffix(csv_path, "_scaled")
}

fn remove_suffix(s: &str, suffix: &str) -> String {
    if s.ends_with(suffix) {
        (&s[..s.len() - suffix.len()]).parse().unwrap()
    } else {
        s.parse().unwrap()
    }
}
