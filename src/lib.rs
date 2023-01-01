mod display;
mod image;
mod metadata_parser;
mod pgm_parser;

use std::{fs, path::PathBuf};

use regex::Regex;

pub use crate::metadata_parser::meta_decode;
pub use crate::pgm_parser::decode;

pub use display::MyApp;
pub use image::RgbImage;

pub fn read_files(dir: &String) -> Vec<PathBuf> {
    // Retrieve image paths from directory
    let mut files = fs::read_dir(dir)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    // Regex to extract the number of the image
    let basename_regex = Regex::new(r"(\d+).pgm").unwrap();

    // Files were sorted by name, we want to sort them by number
    files.sort_by(|a, b| {
        // Converts PathBuf to str
        let a = a.to_str().unwrap();
        let b = b.to_str().unwrap();

        // Extract the number as string from path
        let a = basename_regex.captures(a).unwrap().get(1).unwrap().as_str();
        let b = basename_regex.captures(b).unwrap().get(1).unwrap().as_str();

        // Convert the number to usize
        let a = a.parse::<usize>().unwrap();
        let b = b.parse::<usize>().unwrap();

        a.cmp(&b)
    });

    files
}
