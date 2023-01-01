use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{decode, metadata_parser::Picture};
use eframe::{
    egui::{self, Context},
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
    texture: Option<egui::TextureHandle>,
    pathfile: Vec<PathBuf>,
    meta: Vec<Picture>,

    index: isize,

    state: AppState,
    last_update: Instant,
    refresh_rate: Option<Duration>,

    start_time: Instant,
    never_paused: bool,
}

impl MyApp {
    pub const WINDOW_TITLE: &str = "Image Viewer";

    #[cfg(not(target_os = "windows"))]
    pub const DEFAULT_PATH: &str = "videos/pendulum";
    #[cfg(target_os = "windows")]
    pub const DEFAULT_PATH: &str = r"videos/pendulum";

    pub fn new(files: Vec<PathBuf>, img_per_second: Option<u64>, meta: Vec<Picture>) -> Self {
        MyApp {
            pathfile: files,
            index: 0,
            meta,
            last_update: Instant::now(),
            state: AppState::Play,
            texture: None,
            refresh_rate: img_per_second
                .map(|img_per_second| Duration::from_nanos(1_000_000_000 / img_per_second)),

            start_time: Instant::now(),
            never_paused: true,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let nb_img = self.pathfile.len() as isize;
        let index = self.index.rem_euclid(nb_img) as usize;
        let current_meta = &self.meta[index];

        if self.never_paused && self.index % 100 == 0 {
            println!(
                "fps = {}",
                (self.index as f64) / self.start_time.elapsed().as_secs_f64()
            );
        }

        let refresh_duration = self.refresh_rate.unwrap_or(current_meta.duration);

        let should_refresh = self.texture.is_none()
            || (self.state == AppState::Play && self.last_update.elapsed() > refresh_duration)
            || self.state == AppState::Next
            || self.state == AppState::Previous;

        if should_refresh {
            // Update the last_update time
            self.last_update = Instant::now();

            // Retrieve the right image path to load
            let path = &self.pathfile[index];

            // Print the path of the image to load (only in debug mode)
            #[cfg(debug_assertions)]
            dbg!(&path);

            // Load the image and convert to RGBA pixels
            let img = decode(path).unwrap();
            let size = [img.width(), img.height()];
            let pixels = img.get_rgba();

            // Convert the image to a ColorImage
            let image = epaint::ColorImage::from_rgba_unmultiplied(size, &pixels);
            if let Some(texture) = &mut self.texture {
                // Other loads
                texture.set(image, Default::default());
            } else {
                // On first load
                self.texture = Some(ctx.load_texture("my-image", image, Default::default()));
            }

            if self.state == AppState::Play {
                self.index += 1;
            }
        }

        if matches!(self.state, AppState::Next | AppState::Previous) {
            self.state = AppState::Pause;
        }

        // Request a repaint after the refresh rate (takes into account the time it took to load the image)
        if self.state == AppState::Play {
            ctx.request_repaint_after(self.last_update + refresh_duration - Instant::now());
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

                if play_pause.clicked() {
                    self.state = match self.state {
                        AppState::Play => AppState::Pause,
                        AppState::Pause => AppState::Play,
                        _ => self.state,
                    };
                    ctx.request_repaint();
                    self.never_paused = false;
                };

                if prev.clicked() {
                    ctx.request_repaint();
                    self.index -= 1;
                    self.state = AppState::Previous;
                    self.never_paused = false;
                }
                if next.clicked() {
                    ctx.request_repaint();
                    self.state = AppState::Next;
                    self.index += 1;
                    self.never_paused = false;
                }
            });

            if let Some(texture) = &self.texture {
                ui.image(texture, texture.size_vec2());
            }
        });
    }
}
