use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{decode, flag::FrameMode, metadata_parser::Picture, RgbImage};
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

#[derive(Debug)]
struct MpegFrame {
    pub id: usize,
    pub mode: FrameMode,
    pub duration: Duration,
}

impl MpegFrame {
    pub fn interlaced(&self) -> bool {
        self.mode != FrameMode::PROG
    }

    pub fn repeat_first_field(&self) -> bool {
        self.mode == FrameMode::RFF_TFF || self.mode == FrameMode::RFF_BFF
    }

    pub fn second_field_display_idx(&self) -> i8 {
        match self.mode {
            FrameMode::RFF_TFF | FrameMode::RFF_BFF => 2,
            FrameMode::TFF | FrameMode::BFF => 1,
            _ => 0,
        }
    }
}

pub struct MyApp {
    pathfile: Vec<PathBuf>,
    mode: Option<FrameMode>,
    meta: Option<Vec<Picture>>,

    index: usize,
    loaded_frame: MpegFrame,

    field_display_idx: i8,

    texture_1: egui::TextureHandle,
    texture_2: egui::TextureHandle,

    state: AppState,
    last_update: Instant,
    refresh_rate: Option<Duration>,

    last_fps_update: (Instant, usize),
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
        mode: Option<String>,
        meta: Option<Vec<Picture>>,
    ) -> Self {
        let default_texture_size = [480, 680];

        MyApp {
            pathfile: files,
            mode: mode
                .map(|m| FrameMode::from(m.trim().split_whitespace().collect::<Vec<_>>().iter())),
            meta,

            index: 0,
            loaded_frame: MpegFrame {
                id: 0,
                mode: FrameMode::PROG,
                duration: Duration::from_millis(0),
            },

            field_display_idx: 0,

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

    pub fn incr_index(&mut self) {
        self.index = (self.index + 1) % self.pathfile.len();
    }
    pub fn decr_index(&mut self) {
        self.index = if self.index == 0 {
            self.pathfile.len() - 1
        } else {
            self.index - 1
        };
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let nb_img = self.pathfile.len();

        let (last_update, last_index) = self.last_fps_update;
        let last_update = last_update.elapsed().as_secs_f64();
        if self.state == AppState::Play && last_update >= 1f64 {
            self.last_fps_update = (Instant::now(), self.index as usize);
            self.last_fps = (self.index - last_index) as f64 / last_update;
            if self.loaded_frame.interlaced() {
                self.last_fps *= 2f64;
            }
        }

        let load_new_frame = match self.state {
            AppState::Play => {
                if self.loaded_frame.repeat_first_field() {
                    // 1.5 refresh duration
                    self.last_update.elapsed() >= self.loaded_frame.duration.mul_f32(1.5)
                } else {
                    // 1.0 refresh duration
                    self.last_update.elapsed() >= self.loaded_frame.duration
                }
            }
            AppState::Next | AppState::Previous => self.index != self.loaded_frame.id,
            AppState::Pause => false,
        };

        if load_new_frame {
            // Update the last_update time
            self.last_update = Instant::now();

            // Retrieve the right image path to load
            let path = &self.pathfile[self.index];

            let meta = self
                .meta
                .as_ref()
                .map_or(None, |meta| Some(&meta[self.index]));
            self.loaded_frame = MpegFrame {
                id: self.index,
                mode: self
                    .mode
                    .unwrap_or(meta.map_or(FrameMode::PROG, |meta| meta.picture_type)),
                duration: self
                    .refresh_rate
                    .unwrap_or(meta.map_or(Duration::from_nanos(40_000_000), |meta| meta.duration)),
            };

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
                match self.loaded_frame.mode {
                    FrameMode::PROG => &pixels,
                    FrameMode::RFF_TFF => &top_pixels,
                    FrameMode::RFF_BFF => &bot_pixels,
                    FrameMode::TFF => &top_pixels,
                    FrameMode::BFF => &bot_pixels,
                },
            );

            self.texture_1.set(image, Default::default());

            if self.loaded_frame.interlaced() {
                let image = epaint::ColorImage::from_rgba_unmultiplied(
                    size,
                    match self.loaded_frame.mode {
                        FrameMode::RFF_TFF => &bot_pixels,
                        FrameMode::RFF_BFF => &top_pixels,
                        FrameMode::TFF => &bot_pixels,
                        FrameMode::BFF => &top_pixels,
                        _ => unreachable!(),
                    },
                );

                self.texture_2.set(image, Default::default());
            }

            if self.state == AppState::Play {
                self.incr_index();
                self.field_display_idx = 0;
            }
        }

        // If interlaced, switch field
        if self.loaded_frame.interlaced() && self.state == AppState::Play {
            if self.loaded_frame.repeat_first_field() {
                // First      First      Second
                // [0.0; 0.5[ [0.5; 1.0[ [1.0; 1.5]
                if self.last_update.elapsed() >= self.loaded_frame.duration {
                    self.field_display_idx = 2; // display second field after 1.0 refresh duration
                } else if self.last_update.elapsed() >= self.loaded_frame.duration.div_f32(2.0) {
                    self.field_display_idx = 1; // display first field after 0.5 refresh duration
                }
            } else {
                // First      Second
                // [0.0; 0.5[ [0.5; 1.0[
                if self.last_update.elapsed() >= self.loaded_frame.duration.div_f32(2.0) {
                    self.field_display_idx = 1; // display second field after 0.5 refresh duration
                }
            }
        }

        match self.state {
            // Request a repaint after the refresh rate (takes into account the time it took to load the image)
            AppState::Play => {
                let delay = if self.loaded_frame.interlaced() {
                    self.loaded_frame
                        .duration
                        .mul_f32((self.field_display_idx as f32 + 1.0) * 0.5)
                        .checked_sub(self.last_update.elapsed())
                        .unwrap_or_default()
                } else {
                    self.loaded_frame
                        .duration
                        .checked_sub(self.last_update.elapsed())
                        .unwrap_or_default()
                };
                ctx.request_repaint_after(delay)
            }
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
                    "Frame {}/{}",
                    (self.index + 1) % nb_img,
                    nb_img,
                )));

                if self.state == AppState::Play {
                    ui.add(egui::Label::new(format!("{:.2} fps", self.last_fps,)));
                }

                ui.add(egui::Label::new(format!(
                    "Mode {:?}",
                    self.loaded_frame.mode
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
                    if self.loaded_frame.interlaced() {
                        if self.field_display_idx <= 0 {
                            self.field_display_idx = self.loaded_frame.second_field_display_idx();
                            self.decr_index();
                        } else {
                            self.field_display_idx -= 1;
                        }
                    } else {
                        self.decr_index();
                    }
                    self.state = AppState::Previous;
                    ctx.request_repaint();
                }

                if next.clicked() {
                    if self.loaded_frame.interlaced() {
                        if self.field_display_idx >= self.loaded_frame.second_field_display_idx() {
                            self.field_display_idx = 0;
                            self.incr_index();
                        } else {
                            self.field_display_idx += 1;
                        }
                    } else {
                        self.incr_index();
                    }
                    self.state = AppState::Next;
                    ctx.request_repaint();
                }
            });

            if !self.loaded_frame.interlaced() {
                ui.image(&self.texture_1, self.texture_1.size_vec2());
            } else {
                if self.field_display_idx < self.loaded_frame.second_field_display_idx() {
                    ui.image(&self.texture_1, self.texture_1.size_vec2());
                } else {
                    ui.image(&self.texture_2, self.texture_2.size_vec2());
                }
            }
        });
    }
}
