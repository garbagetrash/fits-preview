use std::fs;
use std::path::PathBuf;

use eframe::egui;
use egui_file::FileDialog;

use fits_preview::texture_display::TextureDisplay;
use fits_preview::*;

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

    // Object to select a directory
    select_dir_dialog: Option<FileDialog>,

    // The rendered image
    texture_display: TextureDisplay,
}

impl PreviewApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        Self::default()
    }

    fn set_directory(&mut self, dir: PathBuf) {
        self.current_directory = Some(dir.clone());
        // Load new files from directory here, populate `directory_files`
        let paths = fs::read_dir(dir).expect("failed to read_dir()");
        for path in paths {
            let _path = path.unwrap().path();
            if !_path.is_dir() {
                self.directory_files.push(_path.clone());
                let mut text = _path.to_str().unwrap().to_string();
                text = text.split('/').last().unwrap().to_string();
                self.directory_files_text.push(text);
            }
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
                    self.texture_display = TextureDisplay::new(dx as usize, dy as usize);

                    let mut temp = vec![];
                    let mut image_buffer = vec![];
                    for bytes in hdu_data.chunks(2) {
                        temp.push((i16::from_be_bytes([bytes[0], bytes[1]]) as i32 + 32768) as u16);
                        if temp.len() == dx as usize {
                            image_buffer.push(temp);
                            temp = vec![];
                        }
                    }
                    self.texture_display.image_buffer = image_buffer;
                    self.last_selected_file = self.selected_file.clone();
                }
            }

            // Show the image
            self.texture_display.plot(ui);
        });

        egui::SidePanel::left("side_panel")
            .default_width(350.0)
            .resizable(false)
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
                egui::ScrollArea::vertical().show(ui, |ui| {
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
                            self.set_directory(dir);
                        }
                    }
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
