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
        let mut output = Self::default();
        output.selected_file = Some(PathBuf::from(
            r"/home/styty/Pictures/Astrophotos/test/Light/L/HD_200775_Light_020.fits",
        ));
        output.current_directory = Some(PathBuf::from(
            r"/home/styty/Pictures/Astrophotos/test/Light/L/",
        ));
        output
    }
}

impl eframe::App for PreviewApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        // Image display
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.last_selected_file != self.selected_file {
                if let Some(pathbuf) = &self.selected_file {
                    let data = std::fs::read(pathbuf).expect("failed to open file");
                    let (kv_pairs, hdu_data) = parse_primary_hdu(&data);
                    let (dx, dy, _) = get_image_dims(&kv_pairs);
                    self.texture_display = TextureDisplay::new(dx as usize, dy as usize);

                    let mut row = 0;
                    let mut col = 0;
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

        egui::SidePanel::left("side_panel").show(ctx, |ui| {

            // Currently selected directory display
            if let Some(dir) = &self.current_directory {
                ui.heading(format!("Current Directory: {:?}", dir));
            }

            // Open directory button
            if (ui.button("Open Directory")).clicked() {
                let mut dialog = FileDialog::select_folder(self.current_directory.clone());
                dialog.open();
                self.select_dir_dialog = Some(dialog);
            }

            // File select UI
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (file, text) in self.directory_files.iter().zip(self.directory_files_text.iter()) {
                    ui.selectable_value(&mut self.selected_file, Some(file.clone()), text);
                }
            });

            // Directory select dialog box
            if let Some(dialog) = &mut self.select_dir_dialog {
                if dialog.show(ctx).selected() {
                    if let Some(dir) = dialog.path() {
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
