use csv::ReaderBuilder;
use std::fs::File;
use std::io::BufReader;

// Where we expect to find the query sets csv files
const QUERY_DATA_FOLDER: &str = "../rsonpath/res/query";
const QUERY_DATA_FOLDER_TEST: &str = "../../../rsonpath/res/query";

// ##########
// 1 kB
// ##########
pub const QUERY_ALPHABET: &str = "alphabet_(2kB)";
pub const QUERY_JOHN: &str = "john_119";
pub const QUERY_NUMBERS: &str = "numbers_117";

// ##########
// 1 MB
// ##########
pub const QUERY_CANADA: &str = "canada_(3MB)";
pub const QUERY_OPENFOOD: &str = "openfood_(867kB)";
pub const QUERY_PEOPLE: &str = "people_(1.2MB)";
pub const QUERY_PRETTY_PEOPLE: &str = "pretty_people_(1.8MB)";
pub const QUERY_TWITTER_MINI: &str = "twitter_(767kB)";

// ##########
// 15 MB
// ##########
pub const QUERY_AST: &str = "ast_(26MB)";
pub const QUERY_DUMMY_10: &str = "dummy_(10MB)";
pub const QUERY_DUMMY_20: &str = "dummy_(20MB)";
pub const QUERY_POKEMON_MINI: &str = "pokemon_(6MB)";

// ##########
// 100 MB
// ##########
pub const QUERY_APP: &str = "app_(97MB)";
pub const QUERY_POKEMON: &str = "pokemon_(173MB)";
pub const QUERY_BESTBUY_SHORT: &str = "bestbuy_short_(103MB)";
pub const QUERY_CROSSREF0: &str = "crossref0_(320MB)";
pub const QUERY_GOOGLE_SHORT: &str = "google_map_short_(107MB)";
pub const QUERY_TWITTER_SHORT: &str = "twitter_short_(80MB)";
pub const QUERY_WALMART_SHORT: &str = "walmart_short_(95MB)";

// ##########
// 1 GB
// ##########
pub const QUERY_BESTBUY: &str = "bestbuy_large_record_(1GB)";
pub const QUERY_CROSSREF1: &str = "crossref1_(551MB)";
pub const QUERY_CROSSREF2: &str = "crossref2_(1.1GB)";
pub const QUERY_CROSSREF4: &str = "crossref4_(2.1GB)";
pub const QUERY_GOOGLE: &str = "google_map_large_record_(1.1GB)";
pub const QUERY_NSPL: &str = "nspl_large_record_(1.2GB)";
pub const QUERY_TWITTER: &str = "twitter_large_record_(843MB)";
pub const QUERY_TWITTER_SINGLE: &str = "twitter_large_record_(843MB)_single";
pub const QUERY_WALMART: &str = "walmart_large_record_(995MB)";
pub const QUERY_WALMART_SINGLE: &str = "walmart_large_record_(995MB)_single";
pub const QUERY_WIKI: &str = "wiki_large_record_(1.1GB)";
pub const QUERY_WIKI_SINGLE: &str = "wiki_large_record_(1.1GB)_single";

// ##########
// 25 GB
// ##########
pub const QUERY_NESTED_COL: &str = "nested_col_(27.7GB)";

#[inline]
#[must_use]
pub fn read_queries(file_name: &str) -> Vec<(String, String)> {
    let csv_path = format!("{QUERY_DATA_FOLDER}/{file_name}.csv");
    read_queries_from_csv(&csv_path)
}

#[inline]
#[must_use]
pub fn read_queries_test(file_name: &str) -> Vec<(String, String)> {
    let csv_path = format!("{QUERY_DATA_FOLDER_TEST}/{file_name}.csv");
    read_queries_from_csv(&csv_path)
}

/// Reads the query data from a csv. It extracts the QUERY_ID and QUERY_TEXT field which are
/// expected to be the first 2 columns.
///
/// Expected csv structure:
///      QUERY_ID,QUERY_TEXT,COUNT_RESULT,SKIP_PERCENTAGE
///      1,$..freeShipping,230089,0.00000000000000000
///      2,$..additionalFeatures[*].feature,61098,0.00000000000000000
///      3,$..includedItemList[*],9096,0.00000000000000000
///      4,$.products[*].videoChapters,769,0.09227438889806847
///      ...
///
/// Check the extract_input function to see which csv_path names are viable.
fn read_queries_from_csv(csv_path: &str) -> Vec<(String, String)> {
    let file = File::open(csv_path).expect(format!("Cannot open CSV file {csv_path}").as_str());
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

/// Based on the input this functions read the queries from the csv and also gives the associated
/// json_path and json_name. We need the json_name so we can differentiate e.g. between QUERY_TWITTER
/// and QUERY_TWITTER_SCALED which both want to query on the same json but have different queries.
#[inline]
#[must_use]
pub fn extract_input(data_dir_path: &str, query_data_csv: &str) -> (String, String, Vec<(String, String)>) {
    let queries = read_queries(query_data_csv);
    let json_name = format!("{}.json", remove_common_suffix(query_data_csv));
    let json_path = format!("{data_dir_path}/{json_name}");
    println!("JSON: {json_path}");

    (json_path, json_name, queries)
}

/// Extract only the json_path of the given query set, because some experiments only need the json
/// and no queries.
#[inline]
#[must_use]
pub fn extract_path(data_dir_path: &str, query_data_csv: &str) -> String {
    let json_name = format!("{}.json", remove_common_suffix(query_data_csv));
    let json_path = format!("{data_dir_path}/{json_name}");
    println!("JSON: {json_path}");

    json_path
}

/// Common suffixes will be removed if possible
/// List of common suffixes:
///     "_scaled": Used to mark query sets that only cover a single query with different ranges
fn remove_common_suffix(csv_path: &str) -> String {
    remove_suffix(csv_path, "_scaled")
}

/// Remove the suffix from the string if possible and return it.
fn remove_suffix(s: &str, suffix: &str) -> String {
    if s.ends_with(suffix) {
        s[..s.len() - suffix.len()].parse().unwrap()
    } else {
        s.parse().unwrap()
    }
}
