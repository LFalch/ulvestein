use image::RgbaImage;
use pixels::Pixels;

use crate::WIDTH;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

impl Colour {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Colour { r, g, b }
    }
    pub fn array(self) -> [u8; 4] {
        [self.r, self.g, self.b, 0xff]
    }
    pub fn alpha(self, a: u8) -> TColour {
        TColour { r: self.r, g: self.g, b: self.b, a }
    }
    /// a represents a value `1/a`
    pub fn scale(self, a: u8) -> Self {
        Colour { r: u8_frac_mul(self.r, a) , g: u8_frac_mul(self.g, a), b: u8_frac_mul(self.b, a) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Texture {
    buffer: Box<[TColour]>,
    width: u16,
}

impl Texture {
    pub fn from_rgba(img: &RgbaImage) -> Self {
        Texture {
            width: img.width() as u16,
            buffer: img.pixels().map(|p| TColour { r: p[0], g: p[1], b: p[2], a: p[3] }).collect()
        }
    }
    pub fn from_file(path: &str) -> Self {
        let img = image::open(path).unwrap().to_rgba8();
        Self::from_rgba(&img)
    }
    pub fn width(&self) -> usize {
        self.width as usize
    }
    pub fn height(&self) -> usize {
        self.buffer.len() / self.width as usize
    }
    /// `u` and `v` are expected to be between 0 and 1
    pub fn get_pixel_f(&self, u: f32, v: f32) -> TColour {
        let u = u.rem_euclid(1.);
        let v = v.rem_euclid(1.);
        let x = (u * self.width as f32).floor() as usize;
        let height = self.height();
        let y = (v * height as f32).floor() as usize;

        self.buffer[y*self.width as usize+x]
    }
    pub fn draw_line_at(&self, frame: &mut Frame, x: u32, y: u32, u: f32, h: u32) {
        for (y, v) in (y..y.saturating_add(h)).map(|sy| (sy, (sy as f32 - y as f32) / h as f32)) {
            frame.draw_rgba(x, y, self.get_pixel_f(u, v));
        }
    }
    /// Draws texture at offset
    pub fn draw_at(&self, frame: &mut Frame, x: u32, y: u32) {
        for (i, &c) in self.buffer.iter().enumerate() {
            let bx = i as u32 % self.width as u32;
            let by = i as u32 / self.width as u32;

            frame.draw_rgba(x+bx, y+by, c);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TColour {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl TColour {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        TColour { r, g, b, a }
    }
    pub fn array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
    pub fn rgb(self) -> Colour {
        Colour { r: self.r, g: self.g, b: self.b }
    }
    pub fn on(self, other: TColour) -> TColour {
        if other.a == 0 || self.a == 255 {
            self
        } else if other.a == 255 {
            let r = u8_frac_mul(self.r, self.a) + u8_frac_mul(other.r, 255 - self.a);
            let g = u8_frac_mul(self.g, self.a) + u8_frac_mul(other.g, 255 - self.a);
            let b = u8_frac_mul(self.b, self.a) + u8_frac_mul(other.b, 255 - self.a);

            TColour::new(r, g, b, 255)
        } else {
            todo!()
        }
    }
}

#[derive(Debug)]
pub struct Frame<'a> {
    buffer: &'a mut [u8],
}

impl<'a> Frame<'a> {
    pub fn from_pixels(pixels: &'a mut Pixels) -> Self {
        Frame { buffer: pixels.get_frame_mut() }
    }
    pub fn draw_rgb(&mut self, x: u32, y: u32, p: Colour) {
        let i = coords_to_index(x, y);
        if let Some(slice) = self.buffer.get_mut(i*4..i*4+4) {
            slice.copy_from_slice(&p.array());
        }
    }
    pub fn draw_rgba(&mut self, x: u32, y: u32, p: TColour) {
        let alpha = p.a;
        if alpha != 0 {
            if alpha == 255 {
                self.draw_rgb(x, y, p.rgb());
            } else {
                let i = coords_to_index(x, y);
                if let Some(orig) = self.buffer.get(i*4..i*4+3) {
                    let orig = Colour::new(orig[0], orig[1], orig[2]).alpha(255);

                    self.draw_rgb(x, y, p.on(orig).rgb());
                }
            }
        }
    }
}

pub const fn u8_frac_mul(a: u8, b: u8) -> u8 {
    ((a as u16 * b as u16) / 255) as u8
}

pub const fn index_to_coords(i: usize) -> (u32, u32) {
    let x = (i % WIDTH as usize) as u32;
    let y = (i / WIDTH as usize) as u32;

    (x, y)
}

pub const fn coords_to_index(x: u32, y: u32) -> usize {
    y as usize * WIDTH as usize + x as usize
}

#[test]
fn test() {
    let (x, y) = index_to_coords(124);
    assert_eq!(index_to_coords(124), index_to_coords(coords_to_index(x, y)));
    assert_eq!(124, coords_to_index(x, y));
}
