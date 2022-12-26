use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::PathBuf,
};

use crate::{image::Rgb, RgbImage};

#[derive(Debug)]
struct Header {
    #[allow(dead_code)]
    /// The magic number of the file. Should be "P5".
    header: &'static str,

    width: usize,
    height: usize,
    #[allow(dead_code)]
    max_val: usize,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
enum Token {
    Whitespace,
    Comment,
    Value(usize),
    Header,
    End,
    Unexpected,
}

fn lex_header(reader: &mut BufReader<File>) -> Token {
    let mut buf = [0; 1];
    if reader.read_exact(&mut buf).is_err() {
        return Token::End;
    }

    if buf[0] == b'P' {
        if reader.read_exact(&mut buf).is_err() {
            return Token::End;
        }
        if buf[0] == b'5' {
            return Token::Header;
        }
        reader.seek_relative(-1).unwrap();
        buf[0] = b'P';
    }

    if buf[0] == b'#' {
        reader.read_line(&mut String::new()).unwrap();
        Token::Comment
    } else if buf[0].is_ascii_whitespace() {
        Token::Whitespace
    } else if buf[0].is_ascii_digit() {
        let mut number = usize::from(buf[0] - b'0');

        while reader.read_exact(&mut buf).is_ok() && buf[0].is_ascii_digit() {
            number = number * 10 + usize::from(buf[0] - b'0');
        }

        reader.seek_relative(-1).unwrap();
        Token::Value(number)
    } else {
        Token::Unexpected
    }
}

fn parse_headers(buf: &mut BufReader<File>) -> Result<Header, &'static str> {
    let header = lex_header(buf);
    assert!(matches!(header, Token::Header));

    let whitespace = lex_header(buf);
    assert!(matches!(whitespace, Token::Whitespace));

    let mut width = lex_header(buf);
    while width == Token::Comment {
        width = lex_header(buf);
    }
    assert!(matches!(width, Token::Value(_)));

    let whitespace = lex_header(buf);
    assert!(matches!(whitespace, Token::Whitespace));

    let height = lex_header(buf);
    assert!(matches!(height, Token::Value(_)));

    let whitespace = lex_header(buf);
    assert!(matches!(whitespace, Token::Whitespace));

    let max_val = lex_header(buf);
    assert!(matches!(max_val, Token::Value(_)));

    let whitespace = lex_header(buf);
    assert!(matches!(whitespace, Token::Whitespace));

    let width = match width {
        Token::Value(width) => width,
        _ => return Err("Invalid width"),
    };

    let height = match height {
        Token::Value(height) => height,
        _ => return Err("Invalid height"),
    };

    let max_val = match max_val {
        Token::Value(max_val) => max_val,
        _ => return Err("Invalid max value"),
    };

    Ok(Header {
        header: "P5",
        width,
        height,
        max_val,
    })
}

pub fn decode(path: &PathBuf) -> Result<RgbImage, &'static str> {
    // Open file
    let file = File::open(path).map_err(|_| "Could not open file")?;
    let mut reader = BufReader::new(file);
    let header = parse_headers(&mut reader)?;

    let byte_width = 1; // usize::from(header.max_val >= 256) + 1;

    // We multiply by 2 and divide by 3 because the "gray" part takes 2/3 of the image
    let img_height = header.height * 2 / 3;
    let img_width = header.width * byte_width;

    // This is dimension for Cr and Cb
    let channel_width = header.width / 2 * byte_width;
    let channel_height = header.height / 3;

    let mut y = vec![0; img_width * img_height];
    let mut u = vec![0; channel_width * channel_height];
    let mut v = vec![0; channel_width * channel_height];

    reader
        .read_exact(&mut y)
        .map_err(|_| "Could not read gray")?;

    for i in 0..channel_height {
        reader
            .read_exact(&mut u[i * channel_width..(i + 1) * channel_width])
            .map_err(|_| "Could not read u")?;

        reader
            .read_exact(&mut v[i * channel_width..(i + 1) * channel_width])
            .map_err(|_| "Could not read v")?;
    }

    let mut img = RgbImage::with_capacity(img_width, img_height);

    for i in 0..img_height {
        for j in 0..img_width {
            let y = f32::from(y[i * img_width + j]) - 16f32;
            let u = f32::from(u[i / 2 * channel_width + j / 2]) - 128f32;
            let v = f32::from(v[i / 2 * channel_width + j / 2]) - 128f32;

            let r = y + 1.370705f32 * v;
            let g = y - 0.698001f32 * v - 0.337633f32 * u;
            let b = y + 1.732446f32 * u;

            let r = r.max(0f32).min(255f32) as u8;
            let g = g.max(0f32).min(255f32) as u8;
            let b = b.max(0f32).min(255f32) as u8;

            img[i][j] = Rgb::new(r, g, b);
        }
    }

    Ok(img)
}
