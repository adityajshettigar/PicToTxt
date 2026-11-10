use std::path::Path;

pub const IMAGE_EXTS: [&str; 8] = ["png", "jpg", "jpeg", "webp", "bmp", "tif", "tiff", "gif"];

pub const IMAGE_MIME_TYPES: [&str; 6] = [
    "image/png",
    "image/jpeg",
    "image/webp",
    "image/bmp",
    "image/tiff",
    "image/gif",
];

fn lower_ext(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
}

/// True when the path has one of the supported image extensions.
pub fn is_valid_image_ext(path: &Path) -> bool {
    lower_ext(path).is_some_and(|e| IMAGE_EXTS.contains(&e.as_str()))
}

/// MIME type for a supported image extension.
pub fn human_mime_from_extension(path: &Path) -> Option<&'static str> {
    match lower_ext(path)?.as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        "bmp" => Some("image/bmp"),
        "tif" | "tiff" => Some("image/tiff"),
        "gif" => Some("image/gif"),
        _ => None,
    }
}

/// Truncate to at most `max` characters (char-safe) for logs and toasts.
pub fn truncate_log(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// Human-readable label for an accelerator string such as `<Primary>c`.
pub fn format_accelerator(accel: &str) -> String {
    match gtk::accelerator_parse(accel) {
        Some((key, mods)) => gtk::accelerator_get_label(key, mods).to_string(),
        None => accel.to_owned(),
    }
}
