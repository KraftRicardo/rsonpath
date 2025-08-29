// use rsonpath::lookup_table::extra::pair_data;
// use rsonpath::lookup_table::luts::lut_hash_map::LutHashMap;
// use rsonpath::lookup_table::luts::lut_hash_map_double::LutHashMapDouble;
// use rsonpath::lookup_table::luts::lut_hash_map_group::LutHashMapGroup;
// use rsonpath::lookup_table::luts::lut_perfect_naive::LutPerfectNaive;
// use rsonpath::lookup_table::luts::lut_phf::LutPHF;
// use rsonpath::lookup_table::luts::lut_phf_double::LutPHFDouble;
// use rsonpath::lookup_table::luts::lut_phf_group::LutPHFGroup;
// use rsonpath::lookup_table::luts::lut_ptr_hash_double::LutPtrHashDouble;
// use rsonpath::lookup_table::{
//     speed::query_data::{QUERY_BESTBUY_SHORT, QUERY_JOHN_BIG, QUERY_POKEMON_MINI, QUERY_TWITTER_SHORT},
//     LookUpTable,
// };
//
// // Macro to generate individual test functions for each (lut_type, json_file) combination
// macro_rules! build_test {
//     ($lut_type:ident, $test_name:ident, $json_file:ident) => {
//         #[test]
//         fn $test_name() {
//             let json_path = format!("../../{}", $json_file);
//             let lut = $lut_type::build(&json_path, 0).expect(&format!(
//                 "Fail @ building {}. Input = {}",
//                 stringify!($lut_type),
//                 json_path
//             ));
//
//             compare_valid(&lut, &json_path);
//         }
//     };
// }
//
// fn compare_valid(lut: &dyn LookUpTable, json_path: &str, cutoff: usize) {
//     let (keys, values) = pair_data::get_pairs_absolute(json_path, cutoff).expect("Fail @ finding pairs.");
//
//     let mut count_incorrect: u64 = 0;
//     for (i, key) in keys.iter().enumerate() {
//         if values[i] != lut.get(key).expect("Fail at getting value.") {
//             count_incorrect += 1;
//         }
//     }
//     assert_eq!(count_incorrect, 0);
// }
//
// // Example usage:
// build_test!(LutHashMap, hash_map_john_big, QUERY_JOHN_BIG);
// build_test!(LutHashMap, hash_map_pokemon, QUERY_POKEMON_MINI);
// build_test!(LutHashMap, hash_map_twitter_short, QUERY_TWITTER_SHORT);
// build_test!(LutHashMap, hash_map_crossref0, QUERY_BESTBUY_SHORT);
//
// // LutHashMapDouble
// build_test!(LutHashMapDouble, hash_map_double_john_big, QUERY_JOHN_BIG);
// build_test!(LutHashMapDouble, hash_map_double_pokemon, QUERY_POKEMON_MINI);
// build_test!(LutHashMapDouble, hash_map_double_twitter_short, QUERY_TWITTER_SHORT);
// build_test!(LutHashMapDouble, hash_map_double_crossref0, QUERY_BESTBUY_SHORT);
//
// // LutHashMapGroup
// build_test!(LutHashMapGroup, hash_map_group_john_big, QUERY_JOHN_BIG);
// build_test!(LutHashMapGroup, hash_map_group_pokemon, QUERY_POKEMON_MINI);
// build_test!(LutHashMapGroup, hash_map_group_twitter_short, QUERY_TWITTER_SHORT);
// build_test!(LutHashMapGroup, hash_map_group_crossref0, QUERY_BESTBUY_SHORT);
//
// // LutPerfectNaive
// build_test!(LutPerfectNaive, perfect_naive_john_big, QUERY_JOHN_BIG);
// build_test!(LutPerfectNaive, perfect_naive_pokemon, QUERY_POKEMON_MINI);
// build_test!(LutPerfectNaive, perfect_naive_twitter_short, QUERY_TWITTER_SHORT);
// build_test!(LutPerfectNaive, perfect_naive_crossref0, QUERY_BESTBUY_SHORT);
//
// // LutPHF
// build_test!(LutPHF, phf_john_big, QUERY_JOHN_BIG);
// build_test!(LutPHF, phf_pokemon, QUERY_POKEMON_MINI);
// build_test!(LutPHF, phf_twitter_short, QUERY_TWITTER_SHORT);
// build_test!(LutPHF, phf_crossref0, QUERY_BESTBUY_SHORT);
//
// // LutPHFDouble
// build_test!(LutPHFDouble, phf_double_john_big, QUERY_JOHN_BIG);
// build_test!(LutPHFDouble, phf_double_pokemon, QUERY_POKEMON_MINI);
// build_test!(LutPHFDouble, phf_double_twitter_short, QUERY_TWITTER_SHORT);
// build_test!(LutPHFDouble, phf_double_crossref0, QUERY_BESTBUY_SHORT);
//
// // LutPHFGroup
// build_test!(LutPHFGroup, phf_group_john_big, QUERY_JOHN_BIG);
// build_test!(LutPHFGroup, phf_group_pokemon, QUERY_POKEMON_MINI);
// build_test!(LutPHFGroup, phf_group_twitter_short, QUERY_TWITTER_SHORT);
// build_test!(LutPHFGroup, phf_group_crossref0, QUERY_BESTBUY_SHORT);
//
// // LutPtrHashDouble
// build_test!(LutPtrHashDouble, ptr_hash_john_big, QUERY_JOHN_BIG);
// build_test!(LutPtrHashDouble, ptr_hash_pokemon, QUERY_POKEMON_MINI);
// build_test!(LutPtrHashDouble, ptr_hash_twitter_short, QUERY_TWITTER_SHORT);
// build_test!(LutPtrHashDouble, ptr_hash_crossref0, QUERY_BESTBUY_SHORT);
