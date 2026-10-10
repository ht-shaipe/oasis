use image::{imageops::FilterType, Rgba, RgbaImage};
use tauri::image::Image;

const TRAY_PX: u32 = 44;

pub fn load_tray_icon() -> Result<Image<'static>, String> {
    let decoded = image::load_from_memory(include_bytes!("../icons/icon.png"))
        .map_err(|err| format!("tray icon decode failed: {err}"))?;
    let resized = decoded
        .resize_exact(TRAY_PX, TRAY_PX, FilterType::Lanczos3)
        .to_rgba8();

    let template = to_template_icon(&resized);

    Ok(Image::new_owned(template.into_raw(), TRAY_PX, TRAY_PX))
}

fn to_template_icon(img: &RgbaImage) -> RgbaImage {
    let (width, height) = img.dimensions();
    let mut out = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let Rgba([r, g, b, a]) = img.get_pixel(x, y);
            if *a < 30 {
                continue;
            }
            let luminance = (0.299 * *r as f32 + 0.587 * *g as f32 + 0.114 * *b as f32) as u8;
            let alpha = if luminance > 200 { 0 } else { 255 };
            out.put_pixel(x, y, Rgba([0, 0, 0, alpha]));
        }
    }

    out
}
