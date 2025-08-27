use csv::ReaderBuilder;
use std::fs::File;
use std::io::BufReader;

const QUERY_DATA_FOLDER: &str = "res/query";
const QUERY_DATA_FOLDER_TEST: &str = "../../res/query";

// DEBUG
pub const QUERY_NSPL_MINI: &str = "nspl_mini"; // cargo run --bin rq -- -v $.data[*][10] res/json/nspl_mini.json > run.log

// 100 MB
pub const QUERY_BESTBUY_SHORT: &str = "bestbuy_short_(103MB)";
pub const QUERY_CROSSREF0: &str = "crossref0_(320MB)";
pub const QUERY_GOOGLE_SHORT: &str = "google_map_short_(107MB)";
pub const QUERY_TWITTER_SHORT: &str = "twitter_short_(80MB)";
pub const QUERY_WALMART_SHORT: &str = "walmart_short_(95MB)";

// 1 GB
pub const QUERY_BESTBUY: &str = "bestbuy_large_record_(1GB)";
pub const QUERY_CROSSREF1: &str = "crossref1_(551MB)";
pub const QUERY_CROSSREF2: &str = "crossref2_(1.1GB)";
pub const QUERY_CROSSREF4: &str = "crossref4_(2.1GB)";
pub const QUERY_GOOGLE: &str = "google_map_large_record_(1.1GB)";
pub const QUERY_NSPL: &str = "nspl_large_record_(1.2GB)";
pub const QUERY_TWITTER: &str = "twitter_large_record_(843MB)";
pub const QUERY_TWITTER_SCALED: &str = "twitter_large_record_(843MB)_scaled";
pub const QUERY_WALMART: &str = "walmart_large_record_(995MB)";
pub const QUERY_WALMART_SCALED: &str = "walmart_large_record_(995MB)_scaled";
pub const QUERY_WIKI: &str = "wiki_large_record_(1.1GB)";
pub const QUERY_WIKI_SCALED: &str = "wiki_large_record_(1.1GB)_scaled";

// 25 GB
pub const QUERY_NESTED_COL: &str = "nested_col_(27.7GB)";

pub fn read_queries(file_name: &str) -> Vec<(String, String)> {
    let csv_path = format!("{QUERY_DATA_FOLDER}/{file_name}.csv");
    read_queries_from_csv(&csv_path)
}

pub fn read_queries_test(file_name: &str) -> Vec<(String, String)> {
    let csv_path = format!("{QUERY_DATA_FOLDER_TEST}/{file_name}.csv");
    read_queries_from_csv(&csv_path)
}

/// Reads the query data from a csv. It extracts the QUERY_ID and QUERY_TEXT field which are
/// expected to be the first 2 columns.
///
/// Expected csv structure:
///      ID,Path,Result,SkipPercentage
///      1,$..freeShipping,230089,0.00000000000000000
///      2,$..additionalFeatures[*].feature,61098,0.00000000000000000
///      3,$..includedItemList[*],9096,0.00000000000000000
///      4,$.products[*].videoChapters,769,0.09227438889806847
///      ...
///
/// Check the extract_input function to see which csv_path names are viable.
fn read_queries_from_csv(csv_path: &str) -> Vec<(String, String)> {
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

/// Based on the input this functions read the queries from the csv and also gives the associated
/// json_path and json_name. We need the json_name so we can differentiate e.g. between QUERY_TWITTER
/// and QUERY_TWITTER_SCALED which both want to query on the same json but have different queries.
pub fn extract_input(data_dir_path: &str, query_data_csv: &str) -> (String, String, Vec<(String, String)>) {
    let queries = read_queries(query_data_csv);
    let json_name = format!("{}.json", remove_common_suffix(query_data_csv));
    let json_path = format!("{data_dir_path}/{json_name}");
    println!("JSON: {json_path}");

    (json_path, json_name, queries)
}

/// Extract only the json_path of the given query set, because some experiments only need the json
/// and no queries.
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
        (&s[..s.len() - suffix.len()]).parse().unwrap()
    } else {
        s.parse().unwrap()
    }
}

// ##########
// kB_1
// ##########
pub const QUERY_ALPHABET: (&str, &[(&str, &str)]) = (
    "alphabet_(2kB).json",
    &[
        ("1", "$.alphabet[0]"),
        ("2", "$.alphabet[*]"),
        ("3", "$.alphabet[*].ID"),
        ("4", "$.alphabet[*].Letter"),
        ("5", "$.alphabet[*].Pronunciation"),
        ("6", "$.alphabet[10].Letter"),
        ("7", "$.alphabet[25].Pronunciation"),
    ],
);

pub const QUERY_BUGS: (&str, &[(&str, &str)]) = (
    "bugs.json",
    &[
        ("1", "$.a..b"),
        ("2", "$.a"),
        ("3", "$.a[0]"),
        ("4", "$.a[0].c"),
        ("5", "$.a[0].c.d"),
        ("6", "$.a[0].c.b"),
    ],
);

pub const QUERY_BUGS_2: (&str, &[(&str, &str)]) = (
    "bugs_2.json",
    &[
        ("1", "$.b[0]"),
        ("2", "$.a"),
        ("3", "$.b"),
        ("4", "$.b[0]"),
        ("5", "$.b[0].b"),
    ],
);

pub const QUERY_JOHN: (&str, &[(&str, &str)]) = (
    "john_119.json",
    &[
        ("1", "$.name"),
        ("2", "$.age"),
        ("3", "$.isMale"),
        ("4", "$.phones"),
        ("5", "$.phones[0]"),
        ("6", "$.phones[1]"),
        ("7", "$.address"),
    ],
);

pub const QUERY_JOHN_BIG: (&str, &[(&str, &str)]) = (
    "john_big.json",
    &[
        ("0", "$.person.address"),
        ("1", "$.person.firstName"),
        ("2", "$.person.lastName"),
        ("3", "$.person.phoneNumber[1].type"),
        ("4", "$.person.spouse.person.phoneNumber.*"),
        ("5", "$.person.spouse.person.phoneNumber[0]"),
        ("6", "$.person.spouse.person.phoneNumber[1]"),
        ("7", "$[1]"),
    ],
);

pub const QUERY_NUMBERS: (&str, &[(&str, &str)]) = (
    "numbers_117.json",
    &[
        ("1", "$.numbers[0]"),
        ("2", "$.numbers[*]"),
        ("3", "$.numbers[25]"),
        ("4", "$.numbers[10]"),
    ],
);

pub const QUERY_SMALL: (&str, &[(&str, &str)]) = ("small_14.json", &[("1", "$.x")]);

// ##########
// MB_1
// ##########
pub const QUERY_CANADA: (&str, &[(&str, &str)]) = (
    "canada_(3MB).json",
    &[
        ("1", "$.features[0].properties.name"),
        ("2", "$.features[*].properties.name"),
        ("3", "$.features[0].geometry.coordinates[0]"),
        ("4", "$.features[0].geometry.coordinates[1]"),
        ("5", "$.features[0].geometry.coordinates[*]"),
        ("6", "$.features[*].geometry.coordinates[*]"),
        ("7", "$.features[0].geometry.type"),
        ("8", "$.features[*].geometry.type"),
    ],
);

pub const QUERY_OPENFOOD: (&str, &[(&str, &str)]) = (
    "openfood_(867kB).json",
    &[
        ("1", "$.products[0].brands"),
        ("2", "$.products[*].brands"),
        ("3", "$.products[0].categories"),
        ("4", "$.products[0].categories_hierarchy"),
        ("5", "$.products[0].ingredients_text"),
        ("6", "$.products[0].allergens"),
        ("7", "$.products[0].ecoscore_grade"),
        ("8", "$.products[*].ecoscore_grade"),
        ("9", "$.products[0].packaging"),
        ("10", "$.products[0].countries"),
        ("13", "$.products[0].ecoscore_score"),
        ("14", "$.products[0].categories_properties"),
        ("15", "$.products[0].ciqual_food_name_tags"),
        ("16", "$.products[0].ingredients_n"),
    ],
);

pub const QUERY_PEOPLE: (&str, &[(&str, &str)]) = (
    "people_(1.2MB).json",
    &[
        ("1", "$[0].name"),
        ("2", "$[*].name"),
        ("3", "$[0].email"),
        ("4", "$[*].email"),
        ("5", "$[0].address"),
        ("6", "$[*].address"),
        ("7", "$[0].phone"),
        ("8", "$[*].phone"),
        ("9", "$[0].website"),
        ("10", "$[*].website"),
    ],
);

pub const QUERY_PRETTY_PEOPLE: (&str, &[(&str, &str)]) = (
    "pretty_people_(1.8MB).json",
    &[
        ("1", "$.ctRoot[0].name"),
        ("2", "$.ctRoot[*].name"),
        ("3", "$.ctRoot[0].dob"),
        ("4", "$.ctRoot[*].dob"),
        ("5", "$.ctRoot[0].address.street"),
        ("6", "$.ctRoot[*].address.street"),
        ("7", "$.ctRoot[0].address.town"),
        ("8", "$.ctRoot[*].address.town"),
        ("9", "$.ctRoot[0].address.postode"),
        ("10", "$.ctRoot[*].address.postode"),
        ("11", "$.ctRoot[0].telephone"),
        ("12", "$.ctRoot[*].telephone"),
        ("13", "$.ctRoot[0].pets"),
        ("14", "$.ctRoot[*].pets"),
        ("15", "$.ctRoot[0].score"),
        ("16", "$.ctRoot[*].score"),
        ("17", "$.ctRoot[0].email"),
        ("18", "$.ctRoot[*].email"),
        ("19", "$.ctRoot[0].url"),
        ("20", "$.ctRoot[*].url"),
        ("21", "$.ctRoot[0].description"),
        ("22", "$.ctRoot[*].description"),
        ("23", "$.ctRoot[0].verified"),
        ("24", "$.ctRoot[*].verified"),
        ("25", "$.ctRoot[0].salary"),
        ("26", "$.ctRoot[*].salary"),
    ],
);

pub const QUERY_TWITTER_MINI: (&str, &[(&str, &str)]) = (
    "twitter_(767kB).json",
    &[
        ("1", "$.statuses[0].metadata.result_type"),
        ("2", "$.statuses[0].metadata.iso_language_code"),
        ("3", "$.statuses[0].created_at"),
        ("4", "$.statuses[*].id"),
        ("5", "$.statuses[*].text"),
        ("6", "$.statuses[0].source"),
        ("7", "$.statuses[0].user.id"),
        ("8", "$.statuses[*].user.name"),
        ("9", "$.statuses[0].user.screen_name"),
        ("10", "$.statuses[*].user.followers_count"),
        ("11", "$.statuses[*].user.friends_count"),
        ("12", "$.statuses[0].retweet_count"),
        ("13", "$.statuses[*].favorite_count"),
        ("14", "$.statuses[0].entities.user_mentions"),
        ("15", "$.statuses[*].lang"),
    ],
);

// ##########
// MB_15
// ##########
pub const QUERY_AST: (&str, &[(&str, &str)]) = (
    "ast_(26MB).json",
    &[
        ("1", "$.inner[0].name"),
        ("2", "$.inner[1].name"),
        ("3", "$.inner[*].type.qualType"),
        ("4", "$.inner[2].type.qualType"),
        ("5", "$.inner[0].inner[0].kind"),
        ("6", "$.inner[1].inner[0].kind"),
        ("7", "$..inner[0].kind"),
        ("10", "$.inner[0].isImplicit"),
        ("11", "$.inner[1].isImplicit"),
        ("12", "$..range.begin.offset"),
        ("14", "$..range.begin.col"),
        ("17", "$..isReferenced"),
    ],
);

pub const QUERY_DUMMY_10: (&str, &[(&str, &str)]) = (
    "dummy_(10MB).json",
    &[
        ("1", "$[0].name"),
        ("2", "$[*].name"),
        ("3", "$[0].email"),
        ("4", "$[*].email"),
        ("5", "$[0].address"),
        ("6", "$[*].address"),
        ("7", "$[0].phone"),
        ("8", "$[*].phone"),
        ("9", "$[0].website"),
        ("10", "$[*].website"),
        ("11", "$[1].name"),
        ("12", "$[1].email"),
        ("13", "$..address"),
        ("14", "$..phone"),
        ("15", "$..website"),
        ("16", "$..name"),
    ],
);

pub const QUERY_DUMMY_20: (&str, &[(&str, &str)]) = (
    "dummy_(20MB).json",
    &[
        ("1", "$[*].name"),
        ("2", "$[1].email"),
        ("3", "$[2].address"),
        ("4", "$[3].phone"),
        ("5", "$[4].website"),
        ("6", "$[5].name"),
        ("7", "$[6].email"),
        ("8", "$[7].address"),
        ("9", "$[8].phone"),
        ("10", "$[9].website"),
        ("11", "$[10].name"),
        ("12", "$[11].email"),
        ("13", "$[12].address"),
        ("14", "$[13].phone"),
        ("15", "$[14].website"),
        ("16", "$..name"),
        ("17", "$..email"),
        ("18", "$..address"),
        ("19", "$..phone"),
        ("20", "$..website"),
    ],
);

pub const QUERY_POKEMON_MINI: (&str, &[(&str, &str)]) = (
    "pokemon_(6MB).json",
    &[
        ("1", "$.cfgs[*].Name"),
        ("2", "$.cfgs[1].ID"),
        ("6", "$.cfgs[*].Color"),
        ("7", "$.cfgs[*].Habitat"),
        ("8", "$.cfgs[6].Shape"),
        ("10", "$.cfgs[*].BaseStats"),
        ("11", "$.cfgs[*].Abilities"),
        ("12", "$.cfgs[9].Moves"),
        ("13", "$.cfgs[*].Moves[*].moveName"),
        ("14", "$.cfgs[*].Moves[*].levelLearnedAt"),
        ("16", "$..Genus"),
        ("18", "$..Height"),
        ("19", "$..Weight"),
    ],
);

// ##########
// MB_100
// ##########
pub const QUERY_APP: (&str, &[(&str, &str)]) = (
    "app_(97MB).json",
    &[
        // ("1", "$.['All ASCII'].here"),
        ("2", "$..here"),
        ("3", "$..sqsk"),
        ("4", "$..xddt"),
    ],
);

pub const QUERY_POKEMON: (&str, &[(&str, &str)]) = (
    "pokemon_(173MB).json",
    &[
        ("1", "$.cfg1[0].Name"),
        ("5", "$.cfg1[*].Abilities[0]"),
        ("6", "$.cfg1[*].Moves[1].moveName"),
        ("7", "$.cfg1[*].Name"),
        ("11", "$.cfg1[*].Abilities[1]"),
        ("12", "$..Moves[*].moveName"),
        ("13", "$..Name"),
        ("17", "$.cfg6[*].Abilities[*]"),
        ("18", "$.cfg7[*].Moves[*].moveName"),
    ],
);
