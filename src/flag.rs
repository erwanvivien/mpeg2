use std::slice::Iter;

#[derive(PartialEq)]
enum Flag {
    None,
    Progressive,
    RepeatFirstField,
    TopFieldFirst,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
#[allow(non_camel_case_types)]
pub enum FrameMode {
    PROG,
    RFF_TFF,
    RFF_BFF,
    TFF,
    BFF,
}

impl From<&str> for Flag {
    fn from(w: &str) -> Self {
        match w {
            "PROG" => Flag::Progressive,
            "RFF" => Flag::RepeatFirstField,
            "TFF" => Flag::TopFieldFirst,
            _ => Flag::None,
        }
    }
}

impl<'a> From<Iter<'_, &'a str>> for FrameMode {
    fn from(it: Iter<&'a str>) -> Self {
        let flags = it.map(|w| Flag::from(*w)).collect::<Vec<_>>();

        if flags.contains(&Flag::RepeatFirstField) {
            if flags.contains(&Flag::TopFieldFirst) {
                FrameMode::RFF_TFF
            } else {
                FrameMode::RFF_BFF
            }
        } else if flags.contains(&Flag::TopFieldFirst) {
            FrameMode::TFF
        } else if flags.contains(&Flag::Progressive) {
            FrameMode::PROG
        } else {
            FrameMode::BFF
        }
    }
}
