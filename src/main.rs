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

    // Run window
    eframe::run_native(
        mpeg2::MyApp::WINDOW_TITLE,
        Default::default(),
        Box::new(move |_| Box::new(mpeg2::MyApp::new(read_files(&pathdir), img_per_second))),
    );
}
