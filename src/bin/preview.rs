use std::fs;
use std::collections::HashMap;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use eframe::egui;
use egui_extras::RetainedImage;
use egui_file::FileDialog;
use serde::{Deserialize, Serialize};

use fits_preview::*;

#[derive(Deserialize, Serialize)]
struct Config {
    default_directory: String,
}

#[derive(Default)]
struct PreviewApp {
    // The currently selected directory
    current_directory: Option<PathBuf>,

    // The files in the selected directory
    directory_files: Vec<PathBuf>,
    directory_files_text: Vec<String>,

    // The currently selected file
    selected_file: Option<PathBuf>,
    last_selected_file: Option<PathBuf>,
    metadata: Option<HashMap<String, String>>,

    // Object to select a directory
    select_dir_dialog: Option<FileDialog>,

    // The rendered image
    //texture_display: TextureDisplay,
    image: Option<RetainedImage>,
}

impl PreviewApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        // Create our base app
        let mut app = Self::default();

        // Try to load config file if it exists
        // Use `$HOME/.config/fits_preview/config.toml`
        let home = std::env::var("HOME").expect("failed to find env var `HOME`");
        let config_file = format!("{}/.config/fits_preview/config.toml", home);
        if let Ok(cfg_str) = std::fs::read_to_string(&config_file) {
            if let Ok(config) = toml::from_str::<Config>(&cfg_str) {
                println!("Using config file found at: {}", config_file);
                app.set_directory(&PathBuf::from(config.default_directory));
            }
        }

        app
    }

    fn set_directory(&mut self, dir: &PathBuf) {
        self.directory_files.clear();
        self.directory_files_text.clear();
        self.current_directory = Some(dir.clone());
        // Load new files from directory here, populate `directory_files`
        let mut paths: Vec<_> = fs::read_dir(dir)
            .expect("failed to read_dir()")
            .filter_map(Result::ok)
            .collect();
        paths.sort_by_key(|x| x.path());
        for path in paths {
            let _path = path.path();
            if !_path.is_dir() {
                self.directory_files.push(_path.clone());
                let mut text = _path.to_str().unwrap().to_string();
                text = text.split('/').last().unwrap().to_string();
                self.directory_files_text.push(text);
            }
        }
    }

    fn push_directory_to_config(&mut self, dir: &Path) {
        // Try to load config file if it exists
        // Use `$HOME/.config/fits_preview/config.toml`
        let home = std::env::var("HOME").expect("failed to find env var `HOME`");
        let config_dir = format!("{}/.config/fits_preview", home);
        let config_file = format!("{}/config.toml", &config_dir);
        fs::create_dir_all(&config_dir).expect("failed to create config dir");
        let config = Config {
            default_directory: dir
                .to_str()
                .expect("failed to convert PathBuf to string")
                .to_string(),
        };
        if let Ok(mut file) = std::fs::File::create(config_file) {
            file.write_all(
                toml::to_string(&config)
                    .expect("failed to serialize Config")
                    .as_bytes(),
            )
            .expect("failed to write to file");
        }
    }
}

impl eframe::App for PreviewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Image display
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.last_selected_file != self.selected_file {
                if let Some(pathbuf) = &self.selected_file {
                    let data = std::fs::read(pathbuf).expect("failed to open file");
                    let (kv_pairs, hdu_data) = parse_primary_hdu(&data);
                    let (dx, dy, _) = get_image_dims(&kv_pairs);
                    self.metadata = Some(kv_pairs);

                    let mut rgb: Vec<u8> = vec![];
                    for bytes in hdu_data.chunks(2) {
                        let value =
                            (i16::from_be_bytes([bytes[0], bytes[1]]) as i32 + 32768) as u16;
                        rgb.push((value >> 8) as u8);
                        rgb.push((value >> 8) as u8);
                        rgb.push((value >> 8) as u8);
                    }
                    let image = egui::ColorImage::from_rgb([dx as usize, dy as usize], &rgb);
                    self.image = Some(RetainedImage::from_color_image("color_image", image));
                    self.last_selected_file = self.selected_file.clone();
                }
            }

            // Show the image
            //self.texture_display.plot(ui);
            if let Some(image) = &self.image {
                image.show(ui);
            }
        });

        egui::SidePanel::left("side_panel")
            .default_width(250.0)
            .show(ctx, |ui| {
                // Keyboard input
                if ui.input(|i| i.key_released(egui::Key::ArrowUp)) {
                    if let Some(selected) = &self.selected_file {
                        let j = self
                            .directory_files
                            .iter()
                            .position(|dir| dir == selected)
                            .unwrap_or(0);
                        if j > 0 {
                            self.selected_file = Some(self.directory_files[j - 1].clone());
                        }
                    } else {
                        self.selected_file = Some(self.directory_files[0].clone());
                    }
                } else if ui.input(|i| i.key_released(egui::Key::ArrowDown)) {
                    if let Some(selected) = &self.selected_file {
                        let j = self
                            .directory_files
                            .iter()
                            .position(|dir| dir == selected)
                            .unwrap_or(0);
                        if j < self.directory_files.len() - 1 {
                            self.selected_file = Some(self.directory_files[j + 1].clone());
                        }
                    } else {
                        self.selected_file = Some(self.directory_files[0].clone());
                    }
                }

                // Currently selected directory display
                if let Some(dir) = &self.current_directory {
                    let current_directory_string = format!("{}", dir.display());
                    if (ui.button(current_directory_string)).clicked() {
                        let mut dialog = FileDialog::select_folder(self.current_directory.clone());
                        dialog.open();
                        self.select_dir_dialog = Some(dialog);
                    }
                } else if (ui.button("Choose directory...")).clicked() {
                    let mut dialog = FileDialog::select_folder(self.current_directory.clone());
                    dialog.open();
                    self.select_dir_dialog = Some(dialog);
                }

                // File select UI
                egui::ScrollArea::vertical()
                    .min_scrolled_width(250.0)
                    .show(ui, |ui| {
                        for (file, text) in self
                            .directory_files
                            .iter()
                            .zip(self.directory_files_text.iter())
                        {
                            ui.selectable_value(&mut self.selected_file, Some(file.clone()), text);
                        }
                    });

                // Directory select dialog box
                if let Some(dialog) = &mut self.select_dir_dialog {
                    if dialog.show(ctx).selected() {
                        if let Some(dir) = dialog.path() {
                            println!(
                                "Setting directory to {}",
                                dir.clone().into_os_string().into_string().unwrap()
                            );
                            self.set_directory(&dir);
                            self.push_directory_to_config(&dir);
                        }
                    }
                }
            });

        egui::SidePanel::right("meta_panel")
            .default_width(250.0)
            .show(ctx, |ui| {
                if let Some(metadata) = &self.metadata {
                    let xres = metadata.get("NAXIS1").unwrap().split('/').take(1).collect::<String>().trim().to_string();
                    let yres = metadata.get("NAXIS2").unwrap().split('/').take(1).collect::<String>().trim().to_string();
                    ui.label(&format!("Resolution: {}x{}", xres, yres));
                    ui.separator();
                }
            });
    }
}

fn main() {
    let native_options = eframe::NativeOptions {
        ..Default::default()
    };
    eframe::run_native(
        "FITS Preview Tool",
        native_options,
        Box::new(|cc| Box::new(PreviewApp::new(cc))),
    )
    .expect("failed to launch app");
}
