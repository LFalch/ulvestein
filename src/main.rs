use std::collections::VecDeque;
use std::f32::consts;
use std::time::Instant;

use image::RgbaImage;
use log::{error, info};
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

pub mod vec;
pub mod map;

use self::map::*;
use self::vec::*;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const FACTOR: u32 = 4;
const WINDOW_WIDTH: f64 = WIDTH as f64;
const WINDOW_HEIGHT: f64 = HEIGHT as f64;
const FOV: f32 = 65.;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        WindowBuilder::new()
            .with_title("Ulvestein")
            .with_inner_size(LogicalSize::new(FACTOR as f64 * WINDOW_WIDTH, FACTOR as f64 * WINDOW_HEIGHT))
            .with_min_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    let mut last = Instant::now();
    let mut last_fpss = VecDeque::new();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(Frame { buffer: pixels.get_frame_mut() });

            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }

            let now = Instant::now();
            let fps = 1. / (now - last).as_secs_f64();
            last_fpss.push_back(fps);
            while last_fpss.len() > 4 {
                last_fpss.pop_front();
            }
            let avg_fps = last_fpss.iter().copied().sum::<f64>() / last_fpss.len() as f64;
            window.set_title(&format!("Ulvestein - FPS {avg_fps:.0}"));
            last = now;
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            let left = input.key_held(VirtualKeyCode::Left);
            let right = input.key_held(VirtualKeyCode::Right);
            let forwards = input.key_held(VirtualKeyCode::Up) || input.key_held(VirtualKeyCode::W);
            let backwards = input.key_held(VirtualKeyCode::Down) || input.key_held(VirtualKeyCode::S);
            let go_right = input.key_held(VirtualKeyCode::D);
            let go_left = input.key_held(VirtualKeyCode::A);

            if input.key_pressed(VirtualKeyCode::N) {
                info!("noclip {}", if world.clip { "on" } else { "off" });
                world.clip = !world.clip;
            }
            if input.key_pressed_os(VirtualKeyCode::Plus) {
                world.fov += 5.;
                info!("fov: {}", world.fov);
            }
            if input.key_pressed_os(VirtualKeyCode::Minus) {
                world.fov -= 5.;
                info!("fov: {}", world.fov);
            }

            world.update(left, right, forwards, backwards, go_left, go_right);
            window.request_redraw();
        }
    });
}

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    player_p: Point2,
    player_angle: f32,
    map: Map<16, 16>,
    fov: f32,
    textures: Box<[Texture]>,
    gun: Texture,
    clip: bool,
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            player_p: Point2::new(6., 6.),
            player_angle: 0.,
            map: Map::new([
                [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
                [2, 0, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
                [2, 0, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
                [2, 0, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0, 7, 0, 0, 2],
                [2, 15, 15, 15, 21, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
                [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
                [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
                [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
                [2, 0, 0, 0, 0, 0, 0, 0, 22, 0, 0, 0, 0, 0, 0, 2],
                [2, 0, 0, 0, 0, 0, 0, 22, 22, 22, 0, 0, 0, 0, 0, 2],
                [2, 0, 0, 0, 0, 0, 0, 0, 22, 0, 0, 0, 0, 0, 0, 2],
                [2, 0, 26, 26, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
                [2, 0, 0, 26, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
                [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
                [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
                [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 2],
            ]),
            fov: FOV,
            clip: true,
            gun: Texture::from_file("gun.png"),
            textures: {
                let mut img = image::open("spritesheet.png").unwrap();
                let mut textures = Vec::new();

                for y in 0..9 {
                    for x in 0..6 {
                        let cropped = img.crop(x*64, y*64, 64, 64);
                        let img = cropped.as_rgba8().unwrap();
                        textures.push(Texture::from_rgba(img));
                    }
                }

                textures.into_boxed_slice()
            },
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self, left: bool, right: bool, forwards: bool, backwards: bool, go_left: bool, go_right: bool) {
        const TURN_SPEED: f32 = 0.05;
        const WALK_SPEED: f32 = 0.05;

        if left || right {
            self.player_angle += TURN_SPEED * (right as i8 - left as i8) as f32;
            self.player_angle %= consts::TAU;
        }

        if forwards || backwards || go_left || go_right {
            let (dy, dx) = self.player_angle.sin_cos();
            let dv = Vector2::new(dx, dy);
            let dp = dv * (forwards as i8 - backwards as i8) as f32 + dv.hat() * (go_right as i8 - go_left as i8) as f32;
            let dp = dp.set_len(WALK_SPEED);
            let cast = self.map.ray_cast(self.player_p, dp, true);
            if self.clip {
                if cast.full() {
                    self.player_p = cast.into_point();
                }
            } else {
                self.player_p = self.player_p + dp;
            }
        }
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, mut frame: Frame) {
        let fov_rad = self.fov * consts::PI / 180.;
        for (x, angle) in (0..WIDTH).map(|x| (x, self.player_angle - 0.5 * fov_rad + fov_rad * x as f32 / WIDTH as f32)) {
            let (mat, dist, u, dark) = {
                let (y, x) = angle.sin_cos();
                let cast = self.map.ray_cast(self.player_p, Vector2::new(x, y), false);

                let p = cast.into_point();

                // Find the corresponding texture x-coordinate
                let frac_x = p.x.fract();
                // Use the dark texture if we came from the side
                let dark = frac_x == 0.;
                // Do some nonsense to make sure the image doesn't flip
                let u = if dark {
                    if x < 0. {
                        1. - p.y.fract()
                    } else {
                        p.y.fract()
                    }
                } else {
                    if y < 0. {
                        frac_x
                    } else {
                        1. - frac_x
                    }
                };

                (cast.material().unwrap_or(Mat::invalid()), (p - self.player_p).norm() * (angle - self.player_angle).cos(), u, dark)
            };

            // Calculate height of line to draw on screen, TODO: change
            let line_height = HEIGHT as f32 / dist;
            let line_height = if line_height.is_infinite() { i32::MAX } else { line_height as i32 };

            // doing the halving for each term eliminates overflow and looks smoother
            const HALF_HEIGHT: i32 = HEIGHT as i32 / 2;
            let half_line_height = line_height / 2;

            let mat_top = HALF_HEIGHT - half_line_height;
            let mat_bot = HALF_HEIGHT + half_line_height;

            for y in 0..HEIGHT as i32 {
                let below_ceiling = mat_top <= y;
                let over_ground = y <= mat_bot;

                let c = match (over_ground, below_ceiling) {
                    (true, true) => match self.textures.get(2 * (mat.id() as usize - 1) + dark as usize) {
                        Some(tex) => {
                            let v = (y - mat_top) as f32 / (mat_bot - mat_top) as f32;

                            tex.get_pixel_f(u, v).rgb()
                        }
                        _ => continue
                    }
                    (true, false) => Colour::new(0x00, 0x00, 0xff),
                    (false, true) => Colour::new(0xff, 0x00, 0x00),
                    (false, false) => Colour::new(0xff, 0xff, 0xff),
                };

                frame.draw_rgb(x, y as u32, c);
            }
        }

        let gun_x = (WIDTH - self.gun.width as u32) / 2;
        let gun_y = HEIGHT - self.gun.height() as u32;
        self.gun.draw_at(&mut frame, gun_x, gun_y)
    }
}


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

#[derive(Debug, Clone)]
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
    pub fn height(&self) -> usize {
        self.buffer.len() / self.width as usize
    }
    /// `u` and `v` are expected to be between 0 and 1
    pub fn get_pixel_f(&self, u: f32, v: f32) -> TColour {
        let x = (u * (self.width - 1) as f32) as usize;
        let height = self.height();
        let y = (v * (height - 1) as f32) as usize;

        self.buffer[y*self.width as usize+x]
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
}

#[derive(Debug)]
pub struct Frame<'a> {
    buffer: &'a mut [u8],
}

impl Frame<'_> {
    fn draw_rgb(&mut self, x: u32, y: u32, p: Colour) {
        let i = coords_to_index(x, y);
        if let Some(slice) = self.buffer.get_mut(i*4..i*4+4) {
            slice.copy_from_slice(&p.array());
        }
    }
    fn draw_rgba(&mut self, x: u32, y: u32, p: TColour) {
        let alpha = p.a;
        let p = p.rgb();
        if alpha != 0 {
            if alpha == 255 {
                self.draw_rgb(x, y, p);
            } else {
                let i = coords_to_index(x, y);
                if let Some(orig) = self.buffer.get(i*4..i*4+3) {
                    let (or, og, ob) = (orig[0], orig[1], orig[2]);

                    let r = u8_frac_mul(p.r, alpha) + u8_frac_mul(or, 255 - alpha);
                    let g = u8_frac_mul(p.g, alpha) + u8_frac_mul(og, 255 - alpha);
                    let b = u8_frac_mul(p.b, alpha) + u8_frac_mul(ob, 255 - alpha);

                    self.draw_rgb(x, y, Colour::new(r, g, b));
                }
            }
        }
    }
}


pub const fn u8_frac_mul(a: u8, b: u8) -> u8 {
    ((a as u16 * b as u16) / 255) as u8
}
