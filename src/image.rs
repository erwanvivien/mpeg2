use std::ops::{Index, IndexMut};

#[repr(C)]
#[derive(Clone)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

pub struct RgbImage {
    data: Vec<(u8, u8, u8)>,
    width: usize,
    height: usize,
}

impl RgbImage {
    pub fn with_capacity(width: usize, height: usize) -> Self {
        let data = vec![(0, 0, 0); width * height];

        Self {
            data,
            width,
            height,
        }
    }
}

impl RgbImage {
    pub fn to_ppm(&self) -> String {
        let mut s = String::with_capacity(self.width * self.height * 3 / 2);

        // # Header
        // Magic value for PPM
        s.push_str("P3\n");
        // Width and height
        s.push_str(&format!("{} {}\n", self.width, self.height));
        // Max value
        s.push_str("255\n");

        // # Body
        let mut column = 0;
        for (r, g, b) in &self.data {
            let str = format!("{} {} {} ", r, g, b);
            if column + str.len() > 70 {
                s.push('\n');
                column = 0;
            }
            s.push_str(&str);

            column += str.len();
        }

        s
    }

    pub fn get_rgb(&self) -> Vec<u8> {
        self.data.iter().flat_map(|p| [p.0, p.1, p.2]).collect()
    }

    pub fn get_rgba(&self) -> Vec<u8> {
        self.data
            .iter()
            .flat_map(|p| [p.0, p.1, p.2, 255])
            .collect()
    }

    pub fn get_data(&self) -> &Vec<(u8, u8, u8)> {
        &self.data
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

impl Index<usize> for RgbImage {
    type Output = [Rgb];

    fn index(&self, index: usize) -> &Self::Output {
        let start = index * self.width;
        unsafe {
            std::slice::from_raw_parts(self.data.as_ptr().add(start) as *const Rgb, self.width)
        }
    }
}

impl IndexMut<usize> for RgbImage {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let start = index * self.width;
        unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_mut_ptr().add(start) as *mut Rgb,
                self.width,
            )
        }
    }
}
