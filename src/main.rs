use std::{path::PathBuf, str::FromStr};

use mpeg2::read_files;

use clap::Parser;

/// MPEG2 Decoder
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = mpeg2::MyApp::DEFAULT_PATH.to_string())]
    pathdir: String,

    #[arg(short, long, default_value_t = 25)]
    fps: u64,
}

fn main() {
    // Parse optional arguments
    let args = Args::parse();
    let Args {
        fps: img_per_second,
        pathdir,
    } = args;

    dbg!(img_per_second);

    let meta = mpeg2::meta_decode(&PathBuf::from_str("./tools/mpeg2dec/tvid.log").unwrap());
    if meta.is_err() {
        panic!("Error while parsing metadata {:?}", meta.err());
    }

    let meta = meta.unwrap();
    let app = mpeg2::MyApp::new(read_files(&pathdir), img_per_second, meta);

    // Run window
    eframe::run_native(
        mpeg2::MyApp::WINDOW_TITLE,
        Default::default(),
        Box::new(move |_| Box::new(app)),
    );
}
