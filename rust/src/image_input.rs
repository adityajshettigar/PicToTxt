use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use gtk::prelude::*;
use gtk::{gdk, gio, glib};
use image::RgbImage;

use crate::util;

/// What was found on the clipboard.
pub enum Pasted {
    Path(PathBuf),
    Bytes(Vec<u8>),
    Nothing,
}

/// Decode an image file into RGB8.
pub fn load_path(path: &Path) -> Result<RgbImage> {
    let kind = util::human_mime_from_extension(path).unwrap_or("image");
    let img = image::open(path).with_context(|| format!("Cannot read {kind} file"))?;
    Ok(img.to_rgb8())
}

/// Decode in-memory encoded image data (e.g. PNG) into RGB8.
pub fn load_bytes(bytes: &[u8]) -> Result<RgbImage> {
    let img = image::load_from_memory(bytes).context("Cannot decode pasted image")?;
    Ok(img.to_rgb8())
}

/// Encode a texture as PNG bytes; portable across GTK4 minor versions.
pub fn texture_to_png(texture: &gdk::Texture) -> Vec<u8> {
    texture.save_to_png_bytes().to_vec()
}

/// Filter a list of files down to supported image paths, in order.
pub fn image_paths(files: &[gio::File]) -> Vec<PathBuf> {
    files
        .iter()
        .filter_map(|f| f.path())
        .filter(|p| util::is_valid_image_ext(p))
        .collect()
}

/// Read the clipboard: file list first, then texture.
pub async fn read_clipboard(clipboard: &gdk::Clipboard) -> Pasted {
    let formats = clipboard.formats();

    if formats.contains_type(gdk::FileList::static_type()) {
        match clipboard
            .read_value_future(gdk::FileList::static_type(), glib::Priority::DEFAULT)
            .await
        {
            Ok(value) => {
                if let Ok(list) = value.get::<gdk::FileList>() {
                    if let Some(first) = image_paths(&list.files()).into_iter().next() {
                        return Pasted::Path(first);
                    }
                }
            }
            Err(e) => eprintln!("picto-txt: clipboard file list: {e}"),
        }
    }

    if formats.contains_type(gdk::Texture::static_type()) {
        match clipboard.read_texture_future().await {
            Ok(Some(texture)) => return Pasted::Bytes(texture_to_png(&texture)),
            Ok(None) => {}
            Err(e) => eprintln!("picto-txt: clipboard texture: {e}"),
        }
    }

    Pasted::Nothing
}
