use std::{
    path::PathBuf,
    sync::{mpsc::Sender, Arc, Mutex},
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

#[derive(Debug, Clone)]
struct MpegFrame {
    pub id: isize,
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

    index: isize,
    loaded_frame: [Arc<Mutex<MpegFrame>>; 2],

    field_display_idx: i8,

    texture_1: [Arc<Mutex<egui::TextureHandle>>; 2],
    texture_2: [Arc<Mutex<egui::TextureHandle>>; 2],

    indices: Arc<Mutex<[usize; 2]>>,

    state: AppState,
    last_update: Instant,

    last_fps_update: (Instant, isize),
    last_fps: f64,

    img_thread: std::thread::JoinHandle<()>,
    sender: Sender<usize>,

    init: Instant,
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

        let texture1 = cc.egui_ctx.load_texture(
            "texture-1",
            ColorImage::new(default_texture_size, egui::Color32::BLACK),
            Default::default(),
        );
        let texture2 = cc.egui_ctx.load_texture(
            "texture-2",
            ColorImage::new(default_texture_size, egui::Color32::BLACK),
            Default::default(),
        );

        let texture3 = cc.egui_ctx.load_texture(
            "texture-3",
            ColorImage::new(default_texture_size, egui::Color32::BLACK),
            Default::default(),
        );
        let texture4 = cc.egui_ctx.load_texture(
            "texture-4",
            ColorImage::new(default_texture_size, egui::Color32::BLACK),
            Default::default(),
        );

        // Create mutex to share the image between the main thread and the image loading thread
        let texture_1 = [
            Arc::new(Mutex::new(texture1)),
            Arc::new(Mutex::new(texture2)),
        ];
        let texture_2 = [
            Arc::new(Mutex::new(texture3)),
            Arc::new(Mutex::new(texture4)),
        ];

        let loaded_frame = [
            Arc::new(Mutex::new(MpegFrame {
                id: 0,
                mode: FrameMode::PROG,
                duration: Duration::from_nanos(40_000_000),
            })),
            Arc::new(Mutex::new(MpegFrame {
                id: 0,
                mode: FrameMode::PROG,
                duration: Duration::from_nanos(40_000_000),
            })),
        ];

        let indices = Arc::new(Mutex::new([0, 0]));

        let mode =
            mode.map(|m| FrameMode::from(m.trim().split_whitespace().collect::<Vec<_>>().iter()));

        let (sender, receiver) = std::sync::mpsc::channel::<usize>();
        let (img_texture_1, img_texture_2) = (texture_1.clone(), texture_2.clone());
        let files_thread = files.clone();
        let loaded_frame_thread = loaded_frame.clone();
        let indices_thread = indices.clone();

        let img_thread = std::thread::spawn(move || {
            let mut rgb_image = RgbImage::with_capacity(0, 0);
            let mut global_vec = Vec::new();

            let files = files_thread;
            let loaded_frame = loaded_frame_thread;

            let mut indices = indices_thread;

            loop {
                let res = receiver.recv();
                if res.is_err() {
                    continue;
                }

                let index = res.unwrap();
                if indices.lock().unwrap().contains(&index) {
                    continue;
                }

                let (img_texture_1, img_texture_2) =
                    (&img_texture_1[index % 2], &img_texture_2[index % 2]);

                let path = &files[index];
                let meta = meta.as_ref().map_or(None, |meta| Some(&meta[index]));

                let refresh_rate = img_per_second
                    .map(|img_per_second| Duration::from_nanos(1_000_000_000 / img_per_second));

                let tmp_frame = MpegFrame {
                    id: index as isize,
                    mode: mode.unwrap_or(meta.map_or(FrameMode::PROG, |meta| meta.picture_type)),
                    duration: refresh_rate.unwrap_or(
                        meta.map_or(Duration::from_nanos(40_000_000), |meta| meta.duration),
                    ),
                };
                *loaded_frame[index % 2].lock().unwrap() = tmp_frame.clone();

                // Print the path of the image to load (only in debug mode)
                #[cfg(debug_assertions)]
                dbg!(&path);

                // Load the image and convert to RGBA pixels
                decode(path, &mut rgb_image).unwrap();

                let img = &rgb_image;
                let size = [img.width(), img.height()];
                let pixels = img.get_rgba();
                if global_vec.len() != pixels.len() * 2 {
                    global_vec = vec![0; pixels.len() * 2];
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
                    global_vec[i * row_nb_bytes..(i + 1) * row_nb_bytes].copy_from_slice(row);
                    global_vec[(i + 1) * row_nb_bytes..(i + 2) * row_nb_bytes].copy_from_slice(row);
                }

                let top_pixels = &global_vec[..pixels.len()];
                let bot_pixels = &global_vec[pixels.len()..];

                // Convert the image to a ColorImage
                let image = epaint::ColorImage::from_rgba_unmultiplied(
                    size,
                    match &tmp_frame.mode {
                        FrameMode::RFF_TFF => &top_pixels,
                        FrameMode::RFF_BFF => &bot_pixels,
                        FrameMode::TFF => &top_pixels,
                        FrameMode::BFF => &bot_pixels,
                        _ => &pixels,
                    },
                );

                img_texture_1.lock().unwrap().set(image, Default::default());

                if tmp_frame.interlaced() {
                    let image = epaint::ColorImage::from_rgba_unmultiplied(
                        size,
                        match tmp_frame.mode {
                            FrameMode::RFF_TFF => &bot_pixels,
                            FrameMode::RFF_BFF => &top_pixels,
                            FrameMode::TFF => &bot_pixels,
                            FrameMode::BFF => &top_pixels,
                            _ => unreachable!(),
                        },
                    );

                    img_texture_2.lock().unwrap().set(image, Default::default());
                }

                indices.lock().unwrap()[index % 2] = index;
            }
        });

        sender.send(0).unwrap();

        MyApp {
            pathfile: files,

            index: 0,
            loaded_frame,

            field_display_idx: 0,

            texture_1,
            texture_2,

            indices,

            state: AppState::Play,
            last_update: Instant::now() - Duration::from_secs(5),

            last_fps_update: (Instant::now(), 0),
            last_fps: 0f64,

            img_thread,
            sender,

            init: Instant::now(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let nb_img = self.pathfile.len() as isize;
        let index = self.index.rem_euclid(nb_img) as usize;

        let loaded_frame = &self.loaded_frame[index % 2].lock().unwrap().clone();

        // Update FPS
        {
            let (last_update, last_index) = self.last_fps_update;
            let last_update = last_update.elapsed().as_secs_f64();
            if self.state == AppState::Play && last_update >= 1f64 {
                self.last_fps_update = (Instant::now(), self.index);
                self.last_fps = ((self.index - last_index) as f64) / last_update;
                if loaded_frame.interlaced() {
                    self.last_fps *= 2f64;
                }
            }
        }

        let load_new_frame = match self.state {
            AppState::Play => {
                if loaded_frame.repeat_first_field() {
                    // 1.5 refresh duration
                    self.last_update.elapsed() >= loaded_frame.duration.mul_f32(1.5)
                } else {
                    // 1.0 refresh duration
                    self.last_update.elapsed() >= loaded_frame.duration
                }
            }
            AppState::Next | AppState::Previous => self.index != loaded_frame.id,
            AppState::Pause => false,
        };

        if load_new_frame {
            self.last_update = Instant::now();

            self.sender.send((index + 1) % (nb_img as usize)).unwrap();

            if self.state == AppState::Play {
                self.index += 1;
                self.field_display_idx = 0;
            }
        }

        // If interlaced, switch field
        if loaded_frame.interlaced() && self.state == AppState::Play {
            if loaded_frame.repeat_first_field() {
                // First      First      Second
                // [0.0; 0.5[ [0.5; 1.0[ [1.0; 1.5]
                if self.last_update.elapsed() >= loaded_frame.duration {
                    self.field_display_idx = 2; // display second field after 1.0 refresh duration
                } else if self.last_update.elapsed() >= loaded_frame.duration.div_f32(2.0) {
                    self.field_display_idx = 1; // display first field after 0.5 refresh duration
                }
            } else {
                // First      Second
                // [0.0; 0.5[ [0.5; 1.0[
                if self.last_update.elapsed() >= loaded_frame.duration.div_f32(2.0) {
                    self.field_display_idx = 1; // display second field after 0.5 refresh duration
                }
            }
        }

        match self.state {
            // Request a repaint after the refresh rate (takes into account the time it took to load the image)
            AppState::Play => {
                let delay = if loaded_frame.interlaced() {
                    loaded_frame
                        .duration
                        .mul_f32((self.field_display_idx as f32 + 1.0) * 0.5)
                } else {
                    loaded_frame.duration
                }
                .checked_sub(self.last_update.elapsed())
                .unwrap_or_default();

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

                ui.add(egui::Label::new(format!("Mode {:?}", loaded_frame.mode)));

                if play_pause.clicked() {
                    self.state = match self.state {
                        AppState::Play => AppState::Pause,
                        AppState::Pause => AppState::Play,
                        _ => self.state,
                    };
                    ctx.request_repaint();
                };

                if prev.clicked() {
                    if loaded_frame.interlaced() {
                        if self.field_display_idx <= 0 {
                            self.field_display_idx = loaded_frame.second_field_display_idx();
                            self.index -= 1;
                        } else {
                            self.field_display_idx -= 1;
                        }
                    } else {
                        self.index -= 1;
                    }
                    self.state = AppState::Previous;
                    ctx.request_repaint();
                }

                if next.clicked() {
                    if loaded_frame.interlaced() {
                        if self.field_display_idx >= loaded_frame.second_field_display_idx() {
                            self.field_display_idx = 0;
                            self.index += 1;
                        } else {
                            self.field_display_idx += 1;
                        }
                    } else {
                        self.index += 1;
                    }
                    self.state = AppState::Next;
                    ctx.request_repaint();
                }
            });

            let texture_1 = self.texture_1[index % 2].lock().unwrap().clone();
            let texture_2 = self.texture_2[index % 2].lock().unwrap().clone();
            if !loaded_frame.interlaced() {
                ui.image(&texture_1, texture_1.size_vec2());
            } else {
                // let elapsed = Instant::now().duration_since(self.init);
                // dbg!((index, elapsed.as_millis(), self.field_display_idx));

                if self.field_display_idx < loaded_frame.second_field_display_idx() {
                    ui.image(&texture_1, texture_1.size_vec2());
                } else {
                    ui.image(&texture_2, texture_2.size_vec2());
                }
            }
        });
    }
}
