use csv::ReaderBuilder;
use std::fs::File;
use std::io::BufReader;

/// Expected csv structure:
///      ID,Path,Result,SkipPercentage
///      1,$..freeShipping,230089,0.00000000000000000
///      2,$..additionalFeatures[*].feature,61098,0.00000000000000000
///      3,$..includedItemList[*],9096,0.00000000000000000
///      4,$.products[*].videoChapters,769,0.09227438889806847
///      ...
/// Note: Query data with the "_scaled" suffix uses just one single query that targets different
/// scopes of the JSON

const QUERY_DATA_FOLDER: &str = "res/query";
const QUERY_DATA_FOLDER_TEST: &str = "../../res/query";

// DEBUG
pub const QUERY_NSPL_MINI: &str = "nspl_mini"; // cargo run --bin rq -- -v $.data[*][10] res/json/nspl_mini.json > run.log

// 1 GB
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

// 25 GB
pub const QUERY_NESTED_COL: &str = "nested_col_(27.7GB)";

pub fn read_queries(file_name: &str) -> (String, Vec<(String, String)>) {
    let csv_path = format!("{QUERY_DATA_FOLDER}/{file_name}.csv");
    read_csv(file_name, &csv_path)
}

pub fn read_queries_test(file_name: &str) -> (String, Vec<(String, String)>) {
    let csv_path = format!("{QUERY_DATA_FOLDER_TEST}/{file_name}.csv");
    read_csv(file_name, &csv_path)
}

/// Reads the query data from a csv. It extracts the QUERY_ID and QUERY_TEXT field which are
/// expected to be the first 2 columns.
pub fn read_csv(file_name: &str, csv_path: &str) -> (String, Vec<(String, String)>) {
    let file = File::open(&csv_path).expect(format!("Cannot open CSV file {csv_path}").as_str());
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(BufReader::new(file));

    let mut queries = Vec::new();

    for result in rdr.records() {
        let record = result.expect("Failed to parse CSV record");
        let id = record.get(0).expect("Missing ID").to_string();
        let path = record.get(1).expect("Missing path").to_string();
        queries.push((id, path));
    }

    let json_name = format!("{}.json", remove_suffix(file_name, "_scaled"));
    (json_name, queries)
}

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

pub const QUERY_BESTBUY_SHORT: (&str, &[(&str, &str)]) = (
    "bestbuy_short_(103MB).json",
    &[
        ("101", "$.products[*].videoChapters"),
        ("102", "$.products[*].videoChapters[1].chapter"),
        ("103", "$.products[*].shipping[*]"),
        ("104", "$.products[*].shipping[*].ground"),
        ("105", "$.products[*].shipping[*].nextDay"),
        ("106", "$.products[*].shipping[*].secondDay"),
        ("107", "$.products[*].shipping[*].vendorDelivery"),
        ("108", "$.products[*].shippingLevelsOfService[*]"),
        ("109", "$.products[*].shippingLevelsOfService[*].serviceLevelId"),
        ("110", "$.products[*].shippingLevelsOfService[*].serviceLevelName"),
        ("111", "$.products[*].shippingLevelsOfService[*].unitShippingPrice"),
        ("112", "$.products[*].categoryPath[2]"),
        ("113", "$.products[*].categoryPath[*].id"),
        ("114", "$.products[*].categoryPath[*].name"),
        ("115", "$.products[*].quantityLimit"),
        ("117", "$.products[*].frequentlyPurchasedWith[*]"),
        ("118", "$.products[*].includedItemList[*]"),
        ("121", "$.products[*].homeDelivery"),
        ("123", "$.products[*].freeShipping"),
        ("124", "$.products[*].additionalFeatures[*]"),
        ("125", "$.products[*].additionalFeatures[*].feature"),
        ("126", "$.products[*].dollarSavings"),
        ("127", "$.products[*].lengthInMinutes"),
        ("128", "$.products[*].screenFormat"),
        ("200", "$..freeShipping"),
        ("201", "$..additionalFeatures[*]"),
        ("202", "$..additionalFeatures[*].feature"),
        ("203", "$..dollarSavings"),
        ("204", "$..lengthInMinutes"),
        ("205", "$..screenFormat"),
        ("300", "$.products[4].categoryPath[2]"),
        ("301", "$.products[4].categoryPath[*].id"),
        ("302", "$.products[4].categoryPath[*].name"),
        ("303", "$.products[4].quantityLimit"),
    ],
);

pub const QUERY_CROSSREF0: (&str, &[(&str, &str)]) = (
    "crossref0_(320MB).json",
    &[
        ("1", "$.items[*].URL"),
        ("2", "$.items[*].resource.primary.URL"),
        ("3", "$.items[*].member"),
        ("4", "$.items[*].author[*].given"),
        ("5", "$.items[*].author[*].family"),
        ("6", "$.items[*].author[*].sequence"),
        ("7", "$.items[*].score"),
        ("8", "$.items[0].prefix"),
        ("9", "$.items[0].DOI"),
        ("10", "$.items[1].URL"),
        ("11", "$.items[1].author[*].given"),
        ("12", "$.items[2].URL"),
        ("13", "$.items[2].resource.primary.URL"),
        ("14", "$..URL"),
        ("15", "$..author[*].given"),
        ("16", "$..author[*].family"),
        ("17", "$..author[*].affiliation[0].name"),
        ("18", "$..title[*]"),
    ],
);

pub const QUERY_GOOGLE_SHORT: (&str, &[(&str, &str)]) = (
    "google_map_short_(107MB).json",
    &[
        ("0", "$[*]..bounds"),
        ("1", "$[*]..bounds.northeast"),
        ("2", "$[*]..bounds.northeast.lat"),
        ("3", "$[*]..bounds.northeast.lng"),
        ("4", "$[*]..copyrights"),
        ("5", "$[*]..summary"),
        ("6", "$[*]..warnings"),
        ("7", "$[*]..waypoint_order"),
        ("8", "$[*].routes[*]"),
        ("9", "$[*].routes[*]..legs"),
        ("10", "$[*].routes[*]..points"),
        ("11", "$[*].routes[*]..steps[*]"),
        ("12", "$[*].routes[*].bounds"),
        ("13", "$[*].routes[*].bounds.northeast"),
        ("14", "$[*].routes[*].bounds.northeast.lat"),
        ("15", "$[*].routes[*].bounds.northeast.lng"),
        ("16", "$[*].routes[*].legs[*].start_location.lat"),
        ("17", "$[*].routes[*].legs[*].steps[1]"),
        ("18", "$[*].routes[*].legs[*].steps[1].distance.text"),
        ("19", "$[*].routes[*].legs[*].traffic_speed_entry"),
        ("20", "$[*].routes[*].overview_polyline"),
        ("21", "$[*].routes[*].overview_polyline.points"),
        ("22", "$[*].routes[*].summary"),
        ("23", "$[*].routes[*].warnings"),
        ("24", "$[*].routes[*].waypoint_order"),
        ("25", "$[1]"),
        ("26", "$[10].routes[*].bounds"),
        ("27", "$[100].routes[*].bounds"),
        ("100", "$[*].routes[*].legs[*]"),
        ("101", "$[*].routes[*].legs[*].steps[*]"),
        ("102", "$[*].routes[*].legs[*].steps[*].distance"),
        ("103", "$[*].routes[*].legs[*].steps[*].distance.text"),
        ("104", "$[*].routes[*].legs[*].steps[*].distance.value"),
        ("108", "$[*].routes[*].legs[*].steps[*].duration"),
        ("109", "$[*].routes[*].legs[*].steps[*].polyline"),
        ("110", "$[*].routes[*].legs[*].steps[*].polyline.points"),
        ("111", "$[*].routes[*].legs[*].steps[*].end_location"),
        ("112", "$[*].routes[*].legs[*].steps[*].html_instructions"),
        ("113", "$[*].routes[*].legs[*].steps[*].travel_mode"),
        ("114", "$[*].routes[*].legs[*].steps[*].start_location"),
        ("115", "$[*].routes[*].legs[*].steps[*].start_location.lat"),
        ("116", "$[*].routes[*].legs[*].steps[*].start_location.lng"),
        ("117", "$[*].routes[*].legs[*].steps[*].maneuver"),
        ("118", "$[*].routes[*].legs[*]..lat"),
        ("119", "$[*].routes[*].legs[*]..lng"),
        ("200", "$[*].available_travel_modes"),
        ("202", "$[*].routes[*]"),
        ("203", "$[*].routes[*].legs[*]"),
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

pub const QUERY_TWITTER_SHORT: (&str, &[(&str, &str)]) = (
    "twitter_short_(80MB).json",
    &[
        ("1", "$[*].geo"),
        ("2", "$[*].id"),
        ("3", "$[*].source"),
        ("4", "$[*].timestamp_ms"),
        ("5", "$[*].user.created_at"),
        ("6", "$[*].retweeted_status.id"),
        ("7", "$[*].retweeted_status.filter_level"),
        ("8", "$[1].retweeted_status.user.following"),
        ("9", "$[0].retweeted_status.user.name"),
        ("10", "$[1].retweeted_status[*]"),
        ("14", "$[1].retweeted_status[*]..id"),
        ("16", "$[1].retweeted_status[*].user.lang"),
        ("17", "$..entities.hashtags[*]"),
        ("18", "$..entities.symbols[*]"),
        ("19", "$..entities.symbols[1]"),
        ("20", "$..urls[*].display_url"),
        ("21", "$[*].entities..symbols[*]"),
        ("22", "$[*].entities..url"),
        ("23", "$[*]..id"),
        ("24", "$[*].entities..symbols[*]"),
        ("25", "$[*].entities..url"),
        ("26", "$[*].entities.symbols[*]"),
        ("27", "$[*].entities.symbols[1]"),
        ("28", "$[*].entities.urls[*].display_url"),
        ("29", "$[*].timestamp_ms"),
    ],
);

pub const QUERY_WALMART_SHORT: (&str, &[(&str, &str)]) = (
    "walmart_short_(95MB).json",
    &[
        ("1", "$.items[*].itemId"),
        ("2", "$.items[*].name"),
        ("3", "$.items[*].msrp"),
        ("4", "$.items[*].salePrice"),
        ("5", "$.items[*].upc"),
        ("6", "$.items[*].categoryPath"),
        ("7", "$.items[*].shortDescription"),
        ("8", "$.items[*].longDescription"),
        ("9", "$.items[0].thumbnailImage"),
        ("10", "$.items[0].productTrackingUrl"),
        ("11", "$.items[0].freeShipToStore"),
        ("12", "$.items[0].stock"),
        ("13", "$..addToCartUrl"),
        ("14", "$..isbn"),
        ("15", "$..availableOnline"),
        ("16", "$..freeShippingOver50Dollars"),
        ("17", "$..categoryNode"),
        ("18", "$..marketplace"),
        ("19", "$.category"),
        ("20", "$.format"),
    ],
);
