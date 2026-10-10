use image::{DynamicImage, GenericImageView, ImageEncoder, ImageFormat, ImageReader};
use oxipng::{Deflaters, InFile, Options, OutFile};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::num::NonZeroU8;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const DEFAULT_QUALITY: u8 = 80;
const MIN_QUALITY: u8 = 1;
const MAX_QUALITY: u8 = 100;
const MAX_JPEG_RECOMPRESS_QUALITY: u8 = 75;
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;
const MAX_IMAGE_DIMENSION: u32 = 16384;
const ZOPFLI_ITERATIONS: u8 = 5;
const LIBDEFLATER_HIGH_LEVEL: u8 = 9;
const LIBDEFLATER_LOW_LEVEL: u8 = 6;

const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "webp", "bmp", "tiff", "gif", "avif",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionOptions {
    pub quality: u8,
    pub format: Option<String>,
    pub scale: Option<u32>,
}

impl Default for CompressionOptions {
    fn default() -> Self {
        Self {
            quality: DEFAULT_QUALITY,
            format: None,
            scale: None,
        }
    }
}

impl CompressionOptions {
    pub fn new(quality: Option<u8>, format: Option<String>, scale: Option<u32>) -> Self {
        let quality = quality
            .unwrap_or(DEFAULT_QUALITY)
            .clamp(MIN_QUALITY, MAX_QUALITY);
        let scale = scale.filter(|&s| s > 0 && s < 100);
        Self { quality, format, scale }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressResult {
    pub path: String,
    pub name: String,
    pub original_size: u64,
    pub compressed_size: u64,
    pub ratio: f64,
    pub success: bool,
    pub error: Option<String>,
    pub width: u32,
    pub height: u32,
}

pub fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|ext| {
            IMAGE_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str())
        })
        .unwrap_or(false)
}

pub fn scan_images_in_dir(dir_path: &str, recursive: bool) -> Result<Vec<ImageInfo>, String> {
    let input_path = Path::new(dir_path);
    if !input_path.exists() {
        return Err(format!("Directory not found: {}", dir_path));
    }
    if !input_path.is_dir() {
        return Err(format!("Not a directory: {}", dir_path));
    }

    let walker = if recursive {
        WalkDir::new(input_path).into_iter()
    } else {
        WalkDir::new(input_path).max_depth(1).into_iter()
    };

    let mut images = Vec::new();
    for entry in walker.filter_entry(|e| !e.file_name().to_string_lossy().starts_with(".~oasis_")) {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() && is_image_file(path) {
            if let Ok(info) = read_image_info(path) {
                images.push(info);
            }
        }
    }

    Ok(images)
}

pub fn scan_image_files(file_paths: &[String]) -> Result<Vec<ImageInfo>, String> {
    let mut images = Vec::new();
    for fp in file_paths {
        let path = Path::new(fp);
        if path.exists() && path.is_file() && is_image_file(path) {
            if let Ok(info) = read_image_info(path) {
                images.push(info);
            }
        }
    }
    Ok(images)
}

fn read_image_info(path: &Path) -> Result<ImageInfo, String> {
    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    let size = metadata.len();

    let reader = ImageReader::open(path)
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?;

    let (width, height) = reader.into_dimensions()
        .map_err(|e| e.to_string())?;

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(ImageInfo {
        path: path.to_string_lossy().to_string(),
        name: path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        size,
        width,
        height,
        format: ext,
    })
}

fn load_image(input_path: &Path) -> Result<(DynamicImage, u64), String> {
    let canonical = input_path
        .canonicalize()
        .map_err(|_| format!("File not found: {:?}", input_path))?;

    let file_size = fs::metadata(&canonical)
        .map_err(|e| e.to_string())?
        .len();
    if file_size > MAX_FILE_SIZE {
        return Err(format!(
            "File too large: {} bytes (max {} bytes)",
            file_size, MAX_FILE_SIZE
        ));
    }

    let img = ImageReader::open(&canonical)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;

    let (w, h) = img.dimensions();
    if w > MAX_IMAGE_DIMENSION || h > MAX_IMAGE_DIMENSION {
        return Err(format!(
            "Image dimensions {}x{} exceed max {}x{}",
            w, h, MAX_IMAGE_DIMENSION, MAX_IMAGE_DIMENSION
        ));
    }

    Ok((img, file_size))
}

fn determine_output_format(output: &Path, format: &Option<String>) -> Result<ImageFormat, String> {
    if let Some(fmt) = format {
        match fmt.to_lowercase().as_str() {
            "jpeg" | "jpg" => Ok(ImageFormat::Jpeg),
            "png" => Ok(ImageFormat::Png),
            "webp" => Ok(ImageFormat::WebP),
            "avif" => Ok(ImageFormat::Avif),
            _ => Err(format!("Unsupported format: {}", fmt)),
        }
    } else if let Some(ext) = output.extension().and_then(|ext| ext.to_str()) {
        match ext.to_ascii_lowercase().as_str() {
            "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
            "png" => Ok(ImageFormat::Png),
            "webp" => Ok(ImageFormat::WebP),
            "avif" => Ok(ImageFormat::Avif),
            "gif" => Ok(ImageFormat::Gif),
            "bmp" => Ok(ImageFormat::Bmp),
            "tif" | "tiff" => Ok(ImageFormat::Tiff),
            _ => Ok(ImageFormat::Jpeg),
        }
    } else {
        Ok(ImageFormat::Jpeg)
    }
}

fn save_image(
    img: &DynamicImage,
    output: &Path,
    format: ImageFormat,
    options: &CompressionOptions,
    input_path: &Path,
) -> Result<(), String> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    match format {
        ImageFormat::Jpeg => {
            use image::codecs::jpeg::JpegEncoder;

            let actual_quality = if input_path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase() == "jpg" || s.to_lowercase() == "jpeg")
                .unwrap_or(false)
            {
                std::cmp::min(options.quality, MAX_JPEG_RECOMPRESS_QUALITY)
            } else {
                options.quality
            };

            let mut file = std::fs::File::create(output).map_err(|e| e.to_string())?;
            let mut encoder = JpegEncoder::new_with_quality(&mut file, actual_quality);
            encoder
                .encode_image(img)
                .map_err(|e| e.to_string())?;
        }
        ImageFormat::Png => {
            let temp_path = output.with_extension("temp.png");
            img.save_with_format(&temp_path, image::ImageFormat::Png)
                .map_err(|e| e.to_string())?;

            struct TempFileGuard(PathBuf);
            impl Drop for TempFileGuard {
                fn drop(&mut self) {
                    let _ = fs::remove_file(&self.0);
                }
            }
            let _guard = TempFileGuard(temp_path.clone());

            let mut oxipng_options = Options::from_preset(4);
            oxipng_options.force = true;

            if options.quality >= 90 {
                oxipng_options.deflate = Deflaters::Zopfli {
                    iterations: NonZeroU8::new(ZOPFLI_ITERATIONS).unwrap(),
                };
            } else if options.quality >= 70 {
                oxipng_options.deflate = Deflaters::Libdeflater {
                    compression: LIBDEFLATER_HIGH_LEVEL,
                };
            } else {
                oxipng_options.deflate = Deflaters::Libdeflater {
                    compression: LIBDEFLATER_LOW_LEVEL,
                };
            }

            let input = InFile::Path(temp_path.clone());
            let out = OutFile::Path {
                path: Some(output.to_path_buf()),
                preserve_attrs: false,
            };
            oxipng::optimize(&input, &out, &oxipng_options)
                .map_err(|e| format!("PNG optimization error: {}", e))?;
        }
        ImageFormat::WebP => {
            // image crate's WebP encoder is LOSSLESS only, which inflates photo
            // files dramatically. Use libwebp (via the `webp` crate) for proper
            // quality-based lossy encoding instead.
            let encoder = webp::Encoder::from_image(img)
                .map_err(|e| format!("WebP encoder error: {}", e))?;
            let quality = options.quality.min(100) as f32;
            let mem = encoder.encode(quality);
            fs::write(output, &*mem).map_err(|e| e.to_string())?;
        }
        ImageFormat::Avif => {
            use image::codecs::avif::AvifEncoder;
            let rgba = img.to_rgba8();
            let (w, h) = (rgba.width(), rgba.height());
            let mut file = std::fs::File::create(output).map_err(|e| e.to_string())?;
            let speed: u8 = 6;
            let enc = AvifEncoder::new_with_speed_quality(&mut file, speed, options.quality);
            enc.write_image(rgba.as_raw(), w, h, image::ExtendedColorType::Rgba8)
                .map_err(|e| e.to_string())?;
        }
        ImageFormat::Gif | ImageFormat::Bmp | ImageFormat::Tiff => {
            img.save_with_format(output, format)
                .map_err(|e| e.to_string())?;
        }
        _ => {
            return Err(format!("Unsupported format: {:?}", format));
        }
    }

    Ok(())
}

pub fn compress_single_image(
    input_path: &str,
    output_dir: Option<&str>,
    options: &CompressionOptions,
) -> CompressResult {
    let input = Path::new(input_path);
    let name = input
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let load_result = load_image(input);
    let (img, original_size) = match load_result {
        Ok(r) => r,
        Err(e) => {
            return CompressResult {
                path: input_path.to_string(),
                name,
                original_size: 0,
                compressed_size: 0,
                ratio: 0.0,
                success: false,
                error: Some(e),
                width: 0,
                height: 0,
            };
        }
    };

    let extension = if let Some(fmt) = &options.format {
        match fmt.to_lowercase().as_str() {
            "jpeg" | "jpg" => "jpg",
            "png" => "png",
            "webp" => "webp",
            "avif" => "avif",
            _ => input
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("jpg"),
        }
    } else {
        input
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("jpg")
    };

    let file_stem = input
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let (output_path, in_place) = if let Some(dir) = output_dir {
        let output_filename = format!("{}.{}", file_stem, extension);
        (PathBuf::from(dir).join(&output_filename), false)
    } else {
        let temp_name = format!(".~oasis_{}.{}", file_stem, extension);
        (input.parent().unwrap_or(Path::new(".")).join(&temp_name), true)
    };

    let output_format = match determine_output_format(&output_path, &options.format) {
        Ok(f) => f,
        Err(e) => {
            return CompressResult {
                path: input_path.to_string(),
                name,
                original_size,
                compressed_size: 0,
                ratio: 0.0,
                success: false,
                error: Some(e),
                width: 0,
                height: 0,
            };
        }
    };

    let (orig_w, orig_h) = img.dimensions();
    let resized_img = if let Some(scale) = options.scale {
        let new_width = ((img.width() as f64) * (scale as f64) / 100.0).round() as u32;
        let new_height = ((img.height() as f64) * (scale as f64) / 100.0).round() as u32;
        if new_width > 0 && new_height > 0 {
            img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
        } else {
            img
        }
    } else {
        img
    };

    if let Err(e) = save_image(&resized_img, &output_path, output_format, options, input) {
        let _ = fs::remove_file(&output_path);
        return CompressResult {
            path: input_path.to_string(),
            name,
            original_size,
            compressed_size: 0,
            ratio: 0.0,
            success: false,
            error: Some(e),
            width: 0,
            height: 0,
        };
    }

    let compressed_size = fs::metadata(&output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    // Never let "compression" grow a file: if the recompressed output is not
    // smaller than the original, keep the original untouched and report 0 gain.
    // (Happens with already well-compressed inputs, e.g. mozjpeg 4:2:0 JPEGs
    // re-encoded by the 4:4:4 image-crate encoder, or losslessly tiny PNGs.)
    if in_place && compressed_size >= original_size {
        let _ = fs::remove_file(&output_path);
        return CompressResult {
            path: input_path.to_string(),
            name,
            original_size,
            compressed_size: original_size,
            ratio: 0.0,
            success: true,
            error: None,
            width: orig_w,
            height: orig_h,
        };
    }

    let final_compressed_size = if in_place {
        let original_path = input.to_path_buf();
        let _ = fs::remove_file(&original_path);
        if let Err(e) = fs::rename(&output_path, &original_path) {
            let _ = fs::remove_file(&output_path);
            return CompressResult {
                path: input_path.to_string(),
                name,
                original_size,
                compressed_size: 0,
                ratio: 0.0,
                success: false,
                error: Some(format!("Failed to replace original file: {}", e)),
                width: 0,
                height: 0,
            };
        }
        fs::metadata(&original_path).map(|m| m.len()).unwrap_or(compressed_size)
    } else {
        compressed_size
    };

    let ratio = if original_size > 0 {
        ((original_size as f64 - final_compressed_size as f64) / original_size as f64) * 100.0
    } else {
        0.0
    };

    CompressResult {
        path: input_path.to_string(),
        name,
        original_size,
        compressed_size: final_compressed_size,
        ratio,
        success: true,
        error: None,
        width: resized_img.width(),
        height: resized_img.height(),
    }
}

pub fn batch_compress(
    image_paths: &[String],
    output_dir: Option<&str>,
    options: &CompressionOptions,
) -> Vec<CompressResult> {
    if let Some(dir) = output_dir {
        if let Err(e) = fs::create_dir_all(dir) {
            return image_paths
                .iter()
                .map(|p| CompressResult {
                    path: p.clone(),
                    name: Path::new(p)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    original_size: 0,
                    compressed_size: 0,
                    ratio: 0.0,
                    success: false,
                    error: Some(format!("Failed to create output dir: {}", e)),
                    width: 0,
                    height: 0,
                })
                .collect();
        }
    }

    let output = output_dir.map(|s| s.to_string());
    let opts = options.clone();

    let paths: Vec<String> = image_paths.to_vec();
    paths
        .par_iter()
        .map(|p| compress_single_image(p, output.as_deref(), &opts))
        .collect()
}

pub fn generate_output_path(
    input_path: &Path,
    output_dir: Option<&Path>,
    format: &Option<String>,
) -> PathBuf {
    let file_stem = input_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let extension = if let Some(fmt) = format {
        match fmt.to_lowercase().as_str() {
            "jpeg" | "jpg" => "jpg",
            "png" => "png",
            "webp" => "webp",
            "avif" => "avif",
            _ => input_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("jpg"),
        }
    } else {
        input_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("jpg")
    };

    let output_filename = format!("{}.{}", file_stem, extension);
    output_dir.unwrap_or(input_path.parent().unwrap_or(Path::new("."))).join(output_filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn make_test_image(path: &Path, w: u32, h: u32) {
        let mut img = image::RgbaImage::new(w, h);
        for (x, y, p) in img.enumerate_pixels_mut() {
            *p = Rgba([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8, 255]);
        }
        img.save(path).expect("write test image");
    }

    #[test]
    fn compress_png_in_place_works() {
        let dir = std::env::temp_dir();
        let input = dir.join("oasis_ic_test.png");
        make_test_image(&input, 300, 200);
        assert!(std::fs::metadata(&input).unwrap().len() > 0);

        let opts = CompressionOptions::new(Some(70), None, None);
        let result = compress_single_image(input.to_str().unwrap(), None, &opts);

        assert!(result.success, "compression failed: {:?}", result.error);
        assert!(result.compressed_size > 0);
        // in-place: original path still exists and now holds the compressed bytes
        assert_eq!(std::fs::metadata(&input).unwrap().len(), result.compressed_size);
        let _ = std::fs::remove_file(&input);
    }

    #[test]
    fn compress_with_scale_and_webp_works() {
        let dir = std::env::temp_dir();
        let input = dir.join("oasis_ic_scale.png");
        make_test_image(&input, 400, 300);
        let out_dir = dir.to_str().unwrap().to_string();
        let opts = CompressionOptions::new(Some(80), Some("webp".to_string()), Some(50));
        let result = compress_single_image(input.to_str().unwrap(), Some(&out_dir), &opts);

        assert!(result.success, "webp/scale compression failed: {:?}", result.error);
        assert_eq!(result.width, 200);
        assert_eq!(result.height, 150);
        let out = dir.join("oasis_ic_scale.webp");
        assert!(out.exists() && std::fs::metadata(&out).unwrap().len() > 0);
        let _ = std::fs::remove_file(&input);
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn scan_files_reads_image_info() {
        let dir = std::env::temp_dir();
        let input = dir.join("oasis_ic_scan.png");
        make_test_image(&input, 100, 100);
        let infos = scan_image_files(&[input.to_str().unwrap().to_string()]).unwrap();
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].width, 100);
        assert_eq!(infos[0].height, 100);
        let _ = std::fs::remove_file(&input);
    }

    fn make_noise_image_rgb(path: &Path, w: u32, h: u32) {
        use image::{Rgb, RgbImage};
        let mut img = RgbImage::new(w, h);
        for (x, y, p) in img.enumerate_pixels_mut() {
            let v = (((x as u32 * 31) ^ (y as u32 * 17))
                .wrapping_mul(2654435761)
                >> 8) % 256;
            *p = Rgb([v as u8, (v.wrapping_mul(7)) as u8, (v.wrapping_mul(13)) as u8]);
        }
        img.save(path).expect("write noise image");
    }

    // Re-encoding a low-quality JPEG at the recompress cap (75) would produce a
    // LARGER file; the in-place guard must reject that and keep the original.
    #[test]
    fn compress_never_grows_in_place() {
        let dir = std::env::temp_dir();
        let src_png = dir.join("oasis_ic_nogrow_src.png");
        make_noise_image_rgb(&src_png, 256, 256);
        let img = image::open(&src_png).unwrap();
        let _ = std::fs::remove_file(&src_png);

        let input = dir.join("oasis_ic_nogrow.jpg");
        let mut f = std::fs::File::create(&input).unwrap();
        let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut f, 40);
        enc.encode_image(&img).unwrap();
        drop(f);

        let before = std::fs::metadata(&input).unwrap().len();
        let opts = CompressionOptions::new(Some(90), None, None);
        let result = compress_single_image(input.to_str().unwrap(), None, &opts);

        assert!(result.success, "compression failed: {:?}", result.error);
        assert_eq!(
            result.compressed_size, before,
            "guard must keep original when output would grow"
        );
        assert_eq!(result.ratio, 0.0);
        assert_eq!(std::fs::metadata(&input).unwrap().len(), before);
        let _ = std::fs::remove_file(&input);
    }

    // The old path used image crate's LOSSLESS webp encoder, which inflated
    // photo-like files. Lossy libwebp must shrink a noise PNG decisively.
    #[test]
    fn webp_lossy_shrinks_noise_image() {
        let dir = std::env::temp_dir();
        let input = dir.join("oasis_ic_webp.png");
        make_noise_image_rgb(&input, 256, 256);
        let before = std::fs::metadata(&input).unwrap().len();

        let opts = CompressionOptions::new(Some(80), Some("webp".to_string()), None);
        let result =
            compress_single_image(input.to_str().unwrap(), Some(dir.to_str().unwrap()), &opts);

        assert!(result.success, "webp compression failed: {:?}", result.error);
        let out = dir.join("oasis_ic_webp.webp");
        let after = std::fs::metadata(&out).unwrap().len();
        assert!(
            after < before,
            "lossy webp should shrink noise png ({} -> {})",
            before,
            after
        );
        let _ = std::fs::remove_file(&input);
        let _ = std::fs::remove_file(&out);
    }
}
