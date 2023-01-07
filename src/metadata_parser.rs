// SEQ <frame_period>
// PIC <offset> <temp_ref> [PROG] [RFF] [TFF]
// PIC ...
// SEQ ...

use std::io::BufRead;
use std::time::Duration;
use std::{fs::File, io::BufReader, path::PathBuf};

use crate::flag::FrameMode;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Picture {
    pub duration: Duration,
    pub picture_type: FrameMode,
    id: usize,
}

pub fn meta_decode(path: &PathBuf) -> Result<Vec<Picture>, String> {
    let file = File::open(path).map_err(|_| format!("Could not open `tvid.log` at {:?}", path))?;
    let mut reader = BufReader::new(file);

    let mut sequence_frame_period = None;
    let mut line = String::new();

    let mut pictures = Vec::new();
    let mut last = 0;

    while let Ok(read_count) = reader.read_line(&mut line) {
        if read_count == 0 {
            break;
        }

        let words = line.trim().split_whitespace().collect::<Vec<_>>();

        if line.starts_with("SEQ") {
            let frame_period = words
                .get(1)
                .unwrap_or(&"1080000")
                .parse::<usize>()
                .map_err(|_| "Could not parse frame_period")?;

            sequence_frame_period = Some(frame_period);
            last = pictures.len()
        } else if line.starts_with("PIC") {
            if words.len() < 3 {
                return Err("line PIC doesn't contain enough fields".into());
            }

            let temp_ref = words[2]
                .parse::<usize>()
                .map_err(|_| "Could not parse temp_ref")?;

            let frame_mode = FrameMode::from(words[3..].iter());

            let frame_period =
                sequence_frame_period.expect("You should have a SEQ before PIC") as f64;

            let picture = Picture {
                id: temp_ref + last,
                duration: Duration::from_millis((27_000_000f64 / frame_period) as u64),
                picture_type: frame_mode,
            };

            pictures.push(picture);
        }

        line.clear();
    }

    pictures.sort_by_key(|p| p.id);

    Ok(pictures)
}
