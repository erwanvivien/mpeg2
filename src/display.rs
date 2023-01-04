use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{
    decode,
    metadata_parser::{Picture, PictureType},
    RgbImage,
};
use eframe::{
    egui::{self, ColorImage, Context},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AppState {
    Play,
    Pause,
    Next,
    Previous,
}

pub struct MyApp {
    pathfile: Vec<PathBuf>,
    meta: Vec<Picture>,

    index: isize,
    loaded_index: isize,
    first_field: bool,
    texture_1: egui::TextureHandle,
    texture_2: egui::TextureHandle,

    state: AppState,
    last_update: Instant,
    refresh_rate: Option<Duration>,

    last_fps_update: (Instant, isize),
    last_fps: f64,

    global_vec: Vec<u8>,
    rgb_image: RgbImage,
}

impl MyApp {
    pub const WINDOW_TITLE: &str = "Image Viewer";

    #[cfg(not(target_os = "windows"))]
    pub const DEFAULT_PATH: &str = "videos/pendulum";
    #[cfg(target_os = "windows")]
    pub const DEFAULT_PATH: &str = r"videos/pendulum";

    pub fn new(
        cc: &eframe::CreationContext<'_>,
        files: Vec<PathBuf>,
        img_per_second: Option<u64>,
        meta: Vec<Picture>,
    ) -> Self {
        let default_texture_size = [480, 620];

        MyApp {
            pathfile: files,
            meta,

            index: 0,
            loaded_index: -1,
            first_field: true,
            texture_1: cc.egui_ctx.load_texture(
                "texture-1",
                ColorImage::new(default_texture_size, egui::Color32::BLACK),
                Default::default(),
            ),
            texture_2: cc.egui_ctx.load_texture(
                "texture-2",
                ColorImage::new(default_texture_size, egui::Color32::BLACK),
                Default::default(),
            ),

            state: AppState::Play,
            last_update: Instant::now(),
            refresh_rate: img_per_second
                .map(|img_per_second| Duration::from_nanos(1_000_000_000 / img_per_second)),

            last_fps_update: (Instant::now(), 0),
            last_fps: 0f64,

            global_vec: Vec::new(),
            rgb_image: RgbImage::with_capacity(0, 0),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let nb_img = self.pathfile.len() as isize;
        let index = self.index.rem_euclid(nb_img) as usize;
        let current_meta = &self.meta[index];

        let (last_update, last_index) = self.last_fps_update;
        let last_update = last_update.elapsed().as_secs_f64();
        if last_update >= 1f64 {
            self.last_fps_update = (Instant::now(), self.index);
            self.last_fps = ((self.index - last_index) as f64) / last_update;
            if current_meta.picture_type != PictureType::Progressive {
                self.last_fps *= 2f64;
            }
        }

        let refresh_duration = self.refresh_rate.unwrap_or(current_meta.duration);

        let load_new_frame = match self.state {
            AppState::Play => self.last_update.elapsed() >= refresh_duration,
            AppState::Next | AppState::Previous => self.index != self.loaded_index,
            AppState::Pause => false,
        };

        // Switch to second field after half the refresh duration
        if current_meta.picture_type != PictureType::Progressive
            && self.state == AppState::Play
            && self.last_update.elapsed() >= refresh_duration.div_f32(2.0)
            && self.first_field
        {
            self.first_field = false;
        }

        if load_new_frame {
            // Update the last_update time
            self.last_update = Instant::now();

            // Retrieve the right image path to load
            let path = &self.pathfile[index];

            // Print the path of the image to load (only in debug mode)
            #[cfg(debug_assertions)]
            dbg!(&path);

            // Load the image and convert to RGBA pixels
            decode(path, &mut self.rgb_image).unwrap();

            let img = &self.rgb_image;
            let size = [img.width(), img.height()];
            let pixels = img.get_rgba();
            if self.global_vec.len() != pixels.len() * 2 {
                self.global_vec = vec![0; pixels.len() * 2];
            }

            // Skip field + vertical nearest neighbour upscaling
            let row_nb_bytes = img.width() * 4;
            let idx = (0..img.height())
                .step_by(2)
                .chain((1..img.height()).step_by(2));

            for (i, idx) in idx.enumerate() {
                let start = idx * row_nb_bytes;
                let end = (idx + 1) * row_nb_bytes;
                let row = &pixels[start..end];

                let i = i * 2;
                self.global_vec[i * row_nb_bytes..(i + 1) * row_nb_bytes].copy_from_slice(row);
                self.global_vec[(i + 1) * row_nb_bytes..(i + 2) * row_nb_bytes]
                    .copy_from_slice(row);
            }

            let top_pixels = &self.global_vec[..pixels.len()];
            let bot_pixels = &self.global_vec[pixels.len()..];

            // Convert the image to a ColorImage
            let image = epaint::ColorImage::from_rgba_unmultiplied(
                size,
                match current_meta.picture_type {
                    PictureType::Progressive => &pixels,
                    PictureType::RepeatFirstField => top_pixels,
                    PictureType::TopFieldFirst => top_pixels,
                    PictureType::BottomFieldFirst => bot_pixels,
                },
            );

            self.texture_1.set(image, Default::default());

            if current_meta.picture_type != PictureType::Progressive {
                let image = epaint::ColorImage::from_rgba_unmultiplied(
                    size,
                    match current_meta.picture_type {
                        PictureType::RepeatFirstField => top_pixels,
                        PictureType::TopFieldFirst => bot_pixels,
                        PictureType::BottomFieldFirst => top_pixels,
                        _ => unreachable!(),
                    },
                );

                self.texture_2.set(image, Default::default());
            }

            self.loaded_index = self.index;

            if self.state == AppState::Play {
                self.index += 1;
                self.first_field = true;
            }
        }

        match self.state {
            // Request a repaint after the refresh rate (takes into account the time it took to load the image)
            AppState::Play => ctx.request_repaint_after(
                refresh_duration
                    .div_f32(2.0)
                    .checked_sub(Instant::now() - self.last_update)
                    .unwrap_or(Duration::from_secs(0)),
            ),
            AppState::Next | AppState::Previous => self.state = AppState::Pause,
            _ => (),
        }

        // Display the image
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let prev = ui.button("Prev");

                let play_pause = ui.button(if self.state == AppState::Play {
                    "Pause"
                } else {
                    "Play"
                });

                let next = ui.button("Next");

                ui.add(egui::Label::new(format!(
                    "Frame {}/{} - {:.2} fps",
                    (self.index + 1) % nb_img,
                    nb_img,
                    self.last_fps,
                )));

                if play_pause.clicked() {
                    self.state = match self.state {
                        AppState::Play => AppState::Pause,
                        AppState::Pause => AppState::Play,
                        _ => self.state,
                    };
                    ctx.request_repaint();
                };

                if prev.clicked() {
                    ctx.request_repaint();
                    if current_meta.picture_type != PictureType::Progressive {
                        if self.first_field {
                            self.index -= 1;
                        }
                        self.first_field = !self.first_field;
                    } else {
                        self.index -= 1;
                    }
                    self.state = AppState::Previous;
                }

                if next.clicked() {
                    ctx.request_repaint();
                    if current_meta.picture_type != PictureType::Progressive {
                        if !self.first_field {
                            self.index += 1;
                        }
                        self.first_field = !self.first_field;
                    } else {
                        self.index += 1;
                    }
                    self.state = AppState::Next;
                }
            });

            if self.first_field {
                ui.image(&self.texture_1, self.texture_1.size_vec2());
            } else {
                ui.image(&self.texture_2, self.texture_2.size_vec2());
            }
        });
    }
}
