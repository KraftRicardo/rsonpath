use std::path::Path;

#[inline]
#[must_use]
pub fn extract_filename(path: &str) -> String {
    let path = Path::new(path);
    let filename = path.file_stem().expect("Failed to extract filename");
    filename.to_string_lossy().into_owned()
}

#[inline]
#[must_use]
pub fn get_filetype_from_path(path: &str) -> String {
    let path = Path::new(path);
    match path.extension() {
        Some(ext) => ext.to_string_lossy().into_owned(),
        None => String::new(),
    }
}

#[inline]
#[must_use]
pub fn get_filename(path: &str) -> &str {
    Path::new(path).file_stem().and_then(|name| name.to_str()).unwrap_or("")
}

#[inline]
#[must_use]
pub fn get_next_valid_name(path: &str) -> String {
    let mut new_path = path.to_string();
    let mut counter = 1;

    while Path::new(&new_path).exists() {
        new_path = format!("{}_({}).csv", path.trim_end_matches(".csv"), counter);
        counter += 1;
    }

    new_path
}
