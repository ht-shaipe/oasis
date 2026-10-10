pub mod commands;
pub mod compress;

pub use compress::{
    compress_single_image, scan_images_in_dir, CompressionOptions, ImageInfo,
};
