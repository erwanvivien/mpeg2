use std::path::PathBuf;

use mpeg2::read_files;

use clap::Parser;

/// MPEG2 Decoder
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = mpeg2::MyApp::DEFAULT_PATH.to_string())]
    pathdir: String,

    #[arg(short, long)]
    fps: Option<u64>,

    #[arg(short, long)]
    mode: Option<String>,

    #[arg(short, long)]
    threshold: Option<f32>,
}

fn main() {
    // Parse optional arguments
    let args = Args::parse();
    let Args {
        fps: img_per_second,
        pathdir,
        mode,
        threshold,
    } = args;

    dbg!(img_per_second);

    let meta = mpeg2::meta_decode(&PathBuf::new().join(&pathdir).join("tvid.log"));
    if mode.is_none() && meta.is_err() {
        eprintln!(
            "\n\nError while parsing metadata {:?}\n\n",
            meta.as_ref().err()
        );
        panic!("Error while parsing metadata {:?}", meta.err());
    }

    let meta = meta.ok();

    // Run window
    eframe::run_native(
        mpeg2::MyApp::WINDOW_TITLE,
        Default::default(),
        Box::new(move |cc| {
            Box::new(mpeg2::MyApp::new(
                cc,
                read_files(&pathdir),
                img_per_second,
                mode,
                threshold,
                meta,
            ))
        }),
    );
}
