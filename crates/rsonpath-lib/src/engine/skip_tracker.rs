use lazy_static::lazy_static;
use log::debug;
use std::collections::HashMap;
use std::fs::{metadata, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::lookup_table::{SkipMode, QUERY_REPETITIONS, SKIP_MODE};
use crate::result::MatchCount;

const ORDER: Ordering = Ordering::Relaxed;

// Used for SkipMode::TRACK, SkipMode::TRACK_TIMED
lazy_static! {
    static ref TRACK_COUNT_LUT: Mutex<HashMap<usize, usize>> = Mutex::new(HashMap::new());
    static ref TRACK_COUNT_ITE: Mutex<HashMap<usize, usize>> = Mutex::new(HashMap::new());
    static ref TRACK_TIME_NANOS: Mutex<HashMap<usize, u64>> = Mutex::new(HashMap::new());
}

// Used for SkipMode::COUNT
static TOTAL_SKIP_COUNT_LUT: AtomicU64 = AtomicU64::new(0);
static TOTAL_SKIP_COUNT_ITE: AtomicU64 = AtomicU64::new(0);
static TOTAL_SKIP_DISTANCE_LUT: AtomicU64 = AtomicU64::new(0);
static TOTAL_SKIP_DISTANCE_ITE: AtomicU64 = AtomicU64::new(0);

/// Note: when calling this function is usually not clear whether it was a LUT or ITE skip!
/// So ignore the skip tag (LUT or ITE)
#[inline]
pub fn track_timed_distance(distance: usize, time_nanos: u64) {
    // Accumulate frequency
    let mut frequency_map = TRACK_COUNT_ITE.lock().expect("Failed to lock TRACK_COUNT_ITE");
    *frequency_map.entry(distance).or_insert(0) += 1;

    // Accumulate time
    let mut time_map = TRACK_TIME_NANOS.lock().expect("Failed to lock TRACK_TIME_NANOS.");
    *time_map.entry(distance).or_insert(0) += time_nanos;
}

#[inline]
pub fn track_distance_lut(distance: usize) {
    // println!("Track: {distance}");

    if SKIP_MODE == SkipMode::COUNT {
        TOTAL_SKIP_COUNT_LUT.fetch_add(1, Ordering::Relaxed);
    } else if SKIP_MODE == SkipMode::TRACK {
        let mut map = TRACK_COUNT_LUT.lock().expect("Failed to lock TRACK_COUNT_LUT");
        *map.entry(distance).or_insert(0) += 1;
    }

    TOTAL_SKIP_DISTANCE_LUT.fetch_add(distance as u64, ORDER);
}

#[inline]
pub fn track_distance_ite(distance: usize) {
    // println!("Track: {distance}");

    if SKIP_MODE == SkipMode::COUNT {
        TOTAL_SKIP_COUNT_ITE.fetch_add(1, Ordering::Relaxed);
    } else if SKIP_MODE == SkipMode::TRACK {
        let mut map = TRACK_COUNT_ITE.lock().expect("Unable to get TRACK_COUNT_ITE lock.");
        *map.entry(distance).or_insert(0) += 1;
    }

    TOTAL_SKIP_DISTANCE_ITE.fetch_add(distance as u64, ORDER);
}

// SkipMode::TRACK
#[inline]
pub fn save_track_to_csv(file_path: &str) -> std::io::Result<()> {
    debug!("Saving to {file_path}");
    let path = Path::new(file_path);
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Headline
    writeln!(writer, "DISTANCE,FREQUENCY,SKIP_TYPE")?;

    // LUT
    let lut_map = TRACK_COUNT_LUT.lock().expect("Unable to get TRACK_COUNT_LUT lock.");
    for (distance, frequency) in lut_map.iter() {
        writeln!(writer, "{distance},{frequency},lut",)?;
    }

    // ITE
    let ite_map = TRACK_COUNT_ITE.lock().expect("Unable to get TRACK_COUNT_ITE lock.");
    for (distance, frequency) in ite_map.iter() {
        writeln!(writer, "{distance},{frequency},ite")?;
    }

    drop(lut_map);
    drop(ite_map);
    reset();

    Ok(())
}

// SkipMode::TRACK_TIMED
#[inline]
pub fn save_track_timed_to_csv(file_path: &str) -> std::io::Result<()> {
    debug!("Saving to {file_path}");
    let path = Path::new(file_path);
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Headline
    writeln!(writer, "DISTANCE,FREQUENCY,SKIP_TYPE,TIME_NANOS,REPETITIONS")?;

    let time_nanos_map = TRACK_TIME_NANOS.lock().expect("Unable to get TRACK_TIME_NANOS lock.");

    // LUT
    let lut_map = TRACK_COUNT_LUT.lock().expect("Unable to get TRACK_COUNT_LUT lock.");
    for (distance, frequency_repeated) in lut_map.iter() {
        let time = time_nanos_map.get(distance).expect("Fail") / QUERY_REPETITIONS as u64;
        let frequency = frequency_repeated / QUERY_REPETITIONS;
        writeln!(writer, "{distance},{frequency},lut,{time},{QUERY_REPETITIONS}")?;
    }

    // ITE
    let ite_map = TRACK_COUNT_ITE.lock().expect("Unable to get TRACK_COUNT_ITE lock");
    for (distance, frequency_repeated) in ite_map.iter() {
        let time = time_nanos_map.get(distance).expect("Fail") / QUERY_REPETITIONS as u64;
        let frequency = frequency_repeated / QUERY_REPETITIONS;
        writeln!(writer, "{distance},{frequency},ite,{time},{QUERY_REPETITIONS}")?;
    }

    drop(time_nanos_map);
    drop(lut_map);
    drop(ite_map);
    reset();

    Ok(())
}

// SkipMode::COUNT
#[inline]
pub fn save_count_to_csv(
    json_path: &str,
    csv_path: &str,
    filename: &str,
    query_id: &str,
    query_text: &str,
    result: MatchCount,
    write_query_csv: bool,
) {
    let lut_count = TOTAL_SKIP_COUNT_LUT.load(ORDER);
    let ite_count = TOTAL_SKIP_COUNT_ITE.load(ORDER);
    let total_count = lut_count + ite_count;
    let lut_distance = TOTAL_SKIP_DISTANCE_LUT.load(ORDER);
    let ite_distance = TOTAL_SKIP_DISTANCE_ITE.load(ORDER);
    let total_distance = lut_distance + ite_distance;
    let json_size: u64 = metadata(json_path).expect("Fail @ reading file metadata").len();

    let percentage_total_skip: f64 = if json_size > 0 {
        (total_distance as f64) / (json_size as f64)
    } else {
        0.0
    };
    let percentage_lut_skip: f64 = if json_size > 0 {
        (lut_distance as f64) / (json_size as f64)
    } else {
        0.0
    };
    let percentage_ite_skip: f64 = if json_size > 0 {
        (ite_distance as f64) / (json_size as f64)
    } else {
        0.0
    };

    let path = Path::new(csv_path);
    let file_existed = path.exists();
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("Fail @ opening file");
    let mut writer = BufWriter::new(file);

    // Add the header if the file does not exist
    if !file_existed {
        writeln!(
            writer,
            "FILENAME,QUERY_ID,QUERY_TEXT,LUT_PERCENT_SKIP,ITE_PERCENT_SKIP,TOTAL_PERCENT_SKIP,LUT_COUNT,ITE_COUNT,TOTAL_COUNT,LUT_DISTANCE,ITE_DISTANCE,TOTAL_DISTANCE,FILE_SIZE",
        )
        .expect("Fail @ writing head");
    }

    // Write data to CSV
    writeln!(
        writer,
        "{filename},{query_id},{query_text},{percentage_lut_skip:.6},{percentage_ite_skip:.6},{percentage_total_skip:.6},{lut_count},{ite_count},{total_count},{lut_distance},{ite_distance},{total_distance},{json_size}",
    )
    .expect("Fail @ writing line");

    println!("TOTAL_SKIP_PERCENT = {percentage_total_skip}");

    writer.flush().expect("Fail @ writing csv");
    reset();

    // ####################
    // Note: This code is not necessary, but can come in handy when you just came up with your
    // queries and did not run them yet, meaning you do not have the result and skip percentage
    // value. If this code is enabled it will write a new csv with updated result and skip
    // percentage values. It currently saves the file at "res/query2/{filename}.csv".
    // Attention: Using _scaled will write the same csv as the non _scaled so you need to manually
    // fix that.
    // ####################
    if write_query_csv {
        let query_2_str = format!("res/query2/{filename}.csv");
        let query_2_path = Path::new(&query_2_str);
        let file_existed = query_2_path.exists();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(query_2_path)
            .expect("Fail @ opening file");
        let mut writer = BufWriter::new(file);

        // Add the header if the file does not exist
        if !file_existed {
            writeln!(writer, "QUERY_ID,QUERY_TEXT,RESULT,SKIP_PERCENTAGE",).expect("Fail @ writing head");
        }
        writeln!(writer, "{query_id},{query_text},{result},{percentage_lut_skip}").expect("Fail @ writing line");
    }
}

#[inline]
pub fn reset() {
    TOTAL_SKIP_COUNT_LUT.store(0, ORDER);
    TOTAL_SKIP_COUNT_ITE.store(0, ORDER);

    TOTAL_SKIP_DISTANCE_LUT.store(0, ORDER);
    TOTAL_SKIP_DISTANCE_ITE.store(0, ORDER);

    TRACK_COUNT_LUT
        .lock()
        .expect("Unable to get TRACK_COUNT_LUT lock")
        .clear();
    TRACK_COUNT_ITE
        .lock()
        .expect("Unable to get TRACK_COUNT_ITE lock")
        .clear();
    TRACK_TIME_NANOS
        .lock()
        .expect("Unable to get TRACK_TIME_NANOS lock")
        .clear();
}
