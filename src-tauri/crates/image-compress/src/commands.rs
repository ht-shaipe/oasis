use crate::compress::{
    compress_single_image, scan_image_files, scan_images_in_dir,
    CompressResult, CompressionOptions, ImageInfo,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanDirParams {
    pub dir_path: String,
    pub recursive: bool,
}

#[tauri::command]
pub async fn image_compress_scan_dir(params: ScanDirParams) -> Result<Vec<ImageInfo>, String> {
    tokio::task::spawn_blocking(move || {
        scan_images_in_dir(&params.dir_path, params.recursive)
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
pub async fn image_compress_scan_files(file_paths: Vec<String>) -> Result<Vec<ImageInfo>, String> {
    tokio::task::spawn_blocking(move || {
        scan_image_files(&file_paths)
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

#[derive(Debug, Deserialize)]
pub struct CompressParams {
    pub input_paths: Vec<String>,
    pub output_dir: Option<String>,
    pub quality: Option<u8>,
    pub format: Option<String>,
    pub scale: Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct BatchCompressResult {
    pub results: Vec<CompressResult>,
    pub total_original: u64,
    pub total_compressed: u64,
    pub total_ratio: f64,
    pub success_count: usize,
    pub fail_count: usize,
}

#[tauri::command]
pub async fn image_compress_batch(
    app: tauri::AppHandle,
    params: CompressParams,
) -> Result<BatchCompressResult, String> {
    let options = CompressionOptions::new(params.quality, params.format.clone(), params.scale);
    let input_paths = params.input_paths.clone();
    let output_dir = params.output_dir.clone();
    let total = input_paths.len();
    let counter = Arc::new(AtomicUsize::new(0));

    let app_handle = app.clone();
    let results = tokio::task::spawn_blocking(move || {
            let opts = options.clone();
            let _output = output_dir.clone();

            let paths: Vec<String> = input_paths.clone();
            let output = output_dir.clone();

        let mut results_vec: Vec<CompressResult> = Vec::with_capacity(paths.len());

        paths
            .par_iter()
            .map(|p| {
                let r = compress_single_image(p, output.as_deref(), &opts);
                let done = counter.fetch_add(1, Ordering::Relaxed) + 1;
                let _ = app_handle.emit("image-compress-progress", CompressProgress {
                    index: done - 1,
                    total,
                    result: r.clone(),
                });
                r
            })
            .collect_into_vec(&mut results_vec);

        results_vec
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?;

    let total_original: u64 = results.iter().map(|r| r.original_size).sum();
    let total_compressed: u64 = results.iter().map(|r| r.compressed_size).sum();
    let total_ratio = if total_original > 0 {
        ((total_original as f64 - total_compressed as f64) / total_original as f64) * 100.0
    } else {
        0.0
    };
    let success_count = results.iter().filter(|r| r.success).count();
    let fail_count = results.len() - success_count;

    Ok(BatchCompressResult {
        results,
        total_original,
        total_compressed,
        total_ratio,
        success_count,
        fail_count,
    })
}

#[derive(Debug, Serialize, Clone)]
pub struct CompressProgress {
    pub index: usize,
    pub total: usize,
    pub result: CompressResult,
}

#[derive(Debug, Deserialize)]
pub struct SingleCompressParams {
    pub input_path: String,
    pub output_dir: Option<String>,
    pub quality: Option<u8>,
    pub format: Option<String>,
    pub scale: Option<u32>,
}

#[tauri::command]
pub async fn image_compress_single(params: SingleCompressParams) -> CompressResult {
    let options = CompressionOptions::new(params.quality, params.format, params.scale);
    let input_path = params.input_path.clone();
    let output_dir = params.output_dir.clone();
    tokio::task::spawn_blocking(move || {
        compress_single_image(&input_path, output_dir.as_deref(), &options)
    })
    .await
    .unwrap_or_else(|e| CompressResult {
        path: params.input_path.clone(),
        name: Path::new(&params.input_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        original_size: 0,
        compressed_size: 0,
        ratio: 0.0,
        success: false,
        error: Some(format!("Task error: {}", e)),
        width: 0,
        height: 0,
    })
}
