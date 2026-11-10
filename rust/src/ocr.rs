use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::{anyhow, Result};
use image::RgbImage;
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use rten::Model;

use crate::{image_input, util};

const DETECTION_MODEL: &[u8] = include_bytes!("../assets/models/detection.rten");
const RECOGNITION_MODEL: &[u8] = include_bytes!("../assets/models/recognition.rten");

const MAX_ERR_CHARS: usize = 120;

static ENGINE: OnceLock<OcrEngine> = OnceLock::new();

/// Work item handed to the OCR worker thread.
pub enum Job {
    Path(PathBuf),
    Bytes(Vec<u8>),
}

/// Result sent back to the UI thread.
pub enum OcrResult {
    Ok { text: String },
    Err(String),
}

fn engine() -> Result<&'static OcrEngine> {
    if let Some(engine) = ENGINE.get() {
        return Ok(engine);
    }

    let detection_model = Model::load_static_slice(DETECTION_MODEL)
        .map_err(|e| anyhow!("Cannot load detection model: {e}"))?;
    let recognition_model = Model::load_static_slice(RECOGNITION_MODEL)
        .map_err(|e| anyhow!("Cannot load recognition model: {e}"))?;

    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })
    .map_err(|e| anyhow!("Cannot start OCR engine: {e}"))?;

    // If another thread won the race, keep its engine.
    let _ = ENGINE.set(engine);
    ENGINE
        .get()
        .ok_or_else(|| anyhow!("OCR engine unavailable"))
}

pub fn extract_text(img: &RgbImage) -> Result<String> {
    let eng = engine()?;
    let src = ImageSource::from_bytes(img.as_raw(), img.dimensions())
        .map_err(|e| anyhow!("Unsupported image data: {e}"))?;
    let input = eng
        .prepare_input(src)
        .map_err(|e| anyhow!("Cannot prepare image: {e}"))?;
    let text = eng
        .get_text(&input)
        .map_err(|e| anyhow!("Text extraction failed: {e}"))?;
    Ok(text.trim().to_owned())
}

fn run(job: Job) -> OcrResult {
    let image = match job {
        Job::Path(path) => image_input::load_path(&path),
        Job::Bytes(bytes) => image_input::load_bytes(&bytes),
    };

    match image.and_then(|img| extract_text(&img)) {
        Ok(text) if text.is_empty() => OcrResult::Err("No text found in image".to_owned()),
        Ok(text) => OcrResult::Ok { text },
        Err(e) => {
            let msg = util::truncate_log(&format!("{e:#}"), MAX_ERR_CHARS);
            eprintln!("picto-txt: {msg}");
            OcrResult::Err(msg)
        }
    }
}

/// Run OCR on a worker thread. The GTK main loop is never blocked.
pub fn spawn(job: Job) -> async_channel::Receiver<OcrResult> {
    let (tx, rx) = async_channel::bounded(1);
    std::thread::spawn(move || {
        let result = run(job);
        let _ = tx.send_blocking(result);
    });
    rx
}
