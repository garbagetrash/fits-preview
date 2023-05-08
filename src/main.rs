use fits_preview::*;
use image::{save_buffer_with_format, ColorType, ImageFormat};

fn main() -> std::io::Result<()> {
    let data =
        std::fs::read("/home/styty/Pictures/Astrophotos/test/Light/L/HD_200775_Light_020.fits")?;

    let (kv_pairs, truncated_hdu_data) = parse_primary_hdu(&data);
    let (dx, dy, _) = get_image_dims(&kv_pairs);

    // Write to PNG
    save_buffer_with_format(
        "output.png",
        &truncated_hdu_data,
        dx,
        dy,
        ColorType::L16,
        ImageFormat::Png,
    )
    .expect("failed to save as PNG");

    Ok(())
}
