use std::{
    ops::Div,
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{decode, flag::FrameMode, metadata_parser::Picture, RgbImage};
use eframe::{
    egui::{self, ColorImage, Context},
    Frame,
};
use ndarray::{s, Array2};

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
    threshold: f32,
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

    rgb_image: RgbImage,
    prev_pixels: Array2<u8>,
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
        threshold: Option<f32>,
        meta: Option<Vec<Picture>>,
    ) -> Self {
        let default_texture_size = [480, 680];

        MyApp {
            pathfile: files,
            mode: mode
                .map(|m| FrameMode::from(m.trim().split_whitespace().collect::<Vec<_>>().iter())),

            threshold: threshold.unwrap_or(0.05),
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

            rgb_image: RgbImage::with_capacity(0, 0),
            prev_pixels: Array2::zeros((0, 0)),
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

            let mut pixels =
                Array2::from_shape_vec((img.height(), img.width() * 4), img.get_rgba()).unwrap();

            // [height / 2, width * 4]
            let mut top_field = pixels.clone();
            top_field
                .slice_mut(s![1isize..;2, ..])
                .assign(&pixels.slice(s![..;2, ..]));

            let mut bot_field = pixels.clone();
            bot_field
                .slice_mut(s![..;2, ..])
                .assign(&pixels.slice(s![1isize..;2, ..]));

            if self.loaded_frame.interlaced() && !self.prev_pixels.is_empty() {
                let curr_top_field = pixels.slice(s![..;2, ..]);
                let curr_bot_field = pixels.slice(s![1isize..;2, ..]);

                let prev_top_field = self.prev_pixels.slice(s![..;2, ..]);
                let prev_bot_field = self.prev_pixels.slice(s![1isize..;2, ..]);

                const BLOCK_SIZE: usize = 8;
                const CHUNK_SIZE: (usize, usize) = (BLOCK_SIZE / 2, BLOCK_SIZE * 4);

                let error_size = (img.height() / BLOCK_SIZE, img.width() / BLOCK_SIZE);

                let errors_vec = prev_top_field
                    .exact_chunks(CHUNK_SIZE)
                    .into_iter()
                    .zip(curr_top_field.exact_chunks(CHUNK_SIZE))
                    .map(|(prev, curr)| {
                        prev.iter()
                            .zip(curr.iter())
                            .map(|(prev, curr)| (*prev as f32 - *curr as f32).abs())
                            .sum::<f32>()
                            .div(CHUNK_SIZE.0 as f32 * CHUNK_SIZE.1 as f32 * 255f32)
                    })
                    .collect::<Vec<f32>>();

                let mut error = Array2::from_shape_vec(error_size, errors_vec).unwrap();

                let errors_vec = prev_bot_field
                    .exact_chunks(CHUNK_SIZE)
                    .into_iter()
                    .zip(curr_bot_field.exact_chunks(CHUNK_SIZE))
                    .map(|(prev, curr)| {
                        prev.iter()
                            .zip(curr.iter())
                            .map(|(prev, curr)| (*prev as f32 - *curr as f32).abs())
                            .sum::<f32>()
                            .div(CHUNK_SIZE.0 as f32 * CHUNK_SIZE.1 as f32 * 255f32)
                    })
                    .collect();

                let error_bot = Array2::from_shape_vec(error_size, errors_vec).unwrap();

                error.zip_mut_with(&error_bot, |e_top, e_bot| {
                    *e_top = e_top.max(*e_bot);
                });

                error.indexed_iter().for_each(|((i, j), err)| {
                    // Weave zone if error is low enough
                    if *err <= self.threshold {
                        let row_start = j * CHUNK_SIZE.1;
                        let row_end = (j + 1) * CHUNK_SIZE.1;

                        let line_start = i * CHUNK_SIZE.0;
                        let line_end = (i + 1) * CHUNK_SIZE.0;

                        let prev_bot =
                            prev_bot_field.slice(s![line_start..line_end, row_start..row_end]);
                        let curr_top =
                            curr_bot_field.slice(s![line_start..line_end, row_start..row_end]);

                        let line_s = i * BLOCK_SIZE;
                        let line_e = (i + 1) * BLOCK_SIZE;

                        // Weave T(current) + B(previous)
                        top_field
                            .slice_mut(s![line_s..line_e;2, row_start..row_end])
                            .assign(&curr_top);
                        top_field
                            .slice_mut(s![(line_s + 1)..line_e;2, row_start..row_end])
                            .assign(&prev_bot);

                        bot_field
                            .slice_mut(s![line_s..line_e;2, row_start..row_end])
                            .assign(&curr_top);
                        bot_field
                            .slice_mut(s![(line_s + 1)..line_e;2, row_start..row_end])
                            .assign(&prev_bot)
                    }
                });
            }

            // Convert the image to a ColorImage
            let image = epaint::ColorImage::from_rgba_unmultiplied(
                [img.width(), img.height()],
                match self.loaded_frame.mode {
                    FrameMode::PROG => &pixels,
                    FrameMode::RFF_TFF => &top_field,
                    FrameMode::RFF_BFF => &bot_field,
                    FrameMode::TFF => &top_field,
                    FrameMode::BFF => &bot_field,
                }
                .as_standard_layout()
                .as_slice()
                .unwrap(),
            );

            self.texture_1.set(image, Default::default());

            if self.loaded_frame.interlaced() {
                let image = epaint::ColorImage::from_rgba_unmultiplied(
                    [img.width(), img.height()],
                    match self.loaded_frame.mode {
                        FrameMode::RFF_TFF => &bot_field,
                        FrameMode::RFF_BFF => &top_field,
                        FrameMode::TFF => &bot_field,
                        FrameMode::BFF => &top_field,
                        _ => unreachable!(),
                    }
                    .as_standard_layout()
                    .as_slice()
                    .unwrap(),
                );

                self.texture_2.set(image, Default::default());
            }

            std::mem::swap(&mut pixels, &mut self.prev_pixels);

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
