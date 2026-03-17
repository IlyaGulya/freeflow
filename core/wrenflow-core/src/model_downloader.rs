//! Model downloader — fetches ONNX model files from HuggingFace.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use wrenflow_domain::model_management::*;

/// Check if a model is already fully downloaded.
pub fn is_model_present(model: &ModelInfo, model_dir: &Path) -> bool {
    model.expected_files.iter().all(|f| model_dir.join(f).exists())
}

/// Download model files from HuggingFace.
/// Supports cancellation via `cancel_flag`.
/// Queries actual file sizes via HEAD requests (no hardcoded sizes).
pub async fn download_model(
    model: &ModelInfo,
    model_dir: &Path,
    listener: Arc<dyn ModelDownloadListener>,
    cancel_flag: Arc<AtomicBool>,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(model_dir).map_err(|e| format!("Create dir: {e}"))?;

    if is_model_present(model, model_dir) {
        log::info!("Model {} already present at {:?}", model.id, model_dir);
        listener.on_state_changed(LocalModelState::Ready);
        return Ok(model_dir.to_path_buf());
    }

    let client = reqwest::Client::builder()
        .user_agent("wrenflow/0.1")
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| format!("HTTP client: {e}"))?;

    let files = &model.expected_files;
    let total_files = files.len();

    // Query total size via HEAD requests
    let mut total_bytes: u64 = 0;
    for filename in files {
        let dest = model_dir.join(filename);
        if dest.exists() {
            if let Ok(meta) = std::fs::metadata(&dest) {
                total_bytes += meta.len();
            }
            continue;
        }
        let url = format!("https://huggingface.co/{}/resolve/main/{}", model.repo_id, filename);
        if let Ok(resp) = client.head(&url).send().await {
            if let Some(len) = resp.content_length() {
                total_bytes += len;
            }
        }
    }
    let total_bytes = if total_bytes > 0 { Some(total_bytes) } else { None };

    let mut bytes_so_far: u64 = 0;

    for (i, filename) in files.iter().enumerate() {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("Cancelled".to_string());
        }

        let dest = model_dir.join(filename);

        // Skip if already exists
        if dest.exists() {
            if let Ok(meta) = std::fs::metadata(&dest) {
                bytes_so_far += meta.len();
            }
            listener.on_progress(DownloadProgress {
                bytes_downloaded: bytes_so_far,
                total_bytes,
                current_file: filename.clone(),
                files_completed: i + 1,
                files_total: total_files,
            });
            continue;
        }

        let url = format!("https://huggingface.co/{}/resolve/main/{}", model.repo_id, filename);
        log::info!("Downloading {} → {:?}", url, dest);

        listener.on_progress(DownloadProgress {
            bytes_downloaded: bytes_so_far,
            total_bytes,
            current_file: filename.clone(),
            files_completed: i,
            files_total: total_files,
        });

        let response = client.get(&url)
            .send()
            .await
            .map_err(|e| format!("Download {filename}: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("Download {filename}: HTTP {}", response.status()));
        }

        // Write to temp file, rename on completion (atomic)
        let tmp_dest = model_dir.join(format!("{filename}.tmp"));
        let mut file = std::fs::File::create(&tmp_dest)
            .map_err(|e| format!("Create {filename}: {e}"))?;

        use std::io::Write;
        use tokio_stream::StreamExt;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            if cancel_flag.load(Ordering::Relaxed) {
                let _ = std::fs::remove_file(&tmp_dest);
                return Err("Cancelled".to_string());
            }

            let chunk = chunk.map_err(|e| format!("Read {filename}: {e}"))?;
            file.write_all(&chunk).map_err(|e| format!("Write {filename}: {e}"))?;
            bytes_so_far += chunk.len() as u64;

            listener.on_progress(DownloadProgress {
                bytes_downloaded: bytes_so_far,
                total_bytes,
                current_file: filename.clone(),
                files_completed: i,
                files_total: total_files,
            });
        }

        // Rename temp to final
        std::fs::rename(&tmp_dest, &dest)
            .map_err(|e| format!("Rename {filename}: {e}"))?;

        listener.on_progress(DownloadProgress {
            bytes_downloaded: bytes_so_far,
            total_bytes,
            current_file: filename.clone(),
            files_completed: i + 1,
            files_total: total_files,
        });
    }

    log::info!("All model files downloaded to {:?}", model_dir);
    Ok(model_dir.to_path_buf())
}
