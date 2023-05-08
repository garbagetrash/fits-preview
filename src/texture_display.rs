use eframe::egui;
use eframe::egui::plot::{Plot, PlotImage, PlotPoint};
use eframe::egui::*;

fn create_image(size: [usize; 2], intensity: &Vec<Vec<u16>>) -> egui::ColorImage {
    let mut max_value = intensity[0][0];
    let mut min_value = intensity[0][0];
    for col in intensity {
        for value in col {
            if *value > max_value {
                max_value = *value;
            }
            if *value < min_value {
                min_value = *value;
            }
        }
    }
    let mut pixel_buf = Vec::with_capacity(4 * size[0] * size[1]);
    for col in intensity {
        for value in col {
            let mono_value = (value >> 8) as u8;
            pixel_buf.push(mono_value);
            pixel_buf.push(mono_value);
            pixel_buf.push(mono_value);
            pixel_buf.push(255);
        }
    }
    // size expects [width, height]
    egui::ColorImage::from_rgba_unmultiplied(size, &pixel_buf)
}

pub struct TextureDisplay {
    dx: usize,
    dy: usize,
    pub image_buffer: Vec<Vec<u16>>,
}

impl Default for TextureDisplay {
    fn default() -> Self {
        Self {
            dx: 0,
            dy: 0,
            image_buffer: vec![vec![]],
        }
    }
}

impl TextureDisplay {
    pub fn new(dx: usize, dy: usize) -> TextureDisplay {
        let mut image_buffer = vec![];
        for _ in 0..dy {
            let row = vec![0; dx];
            image_buffer.push(row);
        }
        TextureDisplay {
            dx,
            dy,
            image_buffer,
        }
    }

    pub fn plot(&mut self, ui: &mut egui::Ui) {
        let texture: egui::TextureHandle = ui.ctx().load_texture(
            "texture demo",
            create_image(
                // size expects [width, height]
                [self.dx, self.dy],
                &self.image_buffer,
            ),
            egui::TextureOptions::LINEAR,
        );
        let image = PlotImage::new(
            &texture,
            PlotPoint::new(0.0, 0.0),
            vec2(texture.aspect_ratio(), 1.0),
        )
        .bg_fill(egui::Color32::BLACK);
        Plot::new("FITS Image")
            .allow_drag(false)
            .allow_scroll(false)
            .show_axes([false, false])
            .set_margin_fraction(vec2(0.0, 0.0))
            .show(ui, |plot_ui| {
                plot_ui.image(image.name("Image"));
            })
            .response;
    }
}
