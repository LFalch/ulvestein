use std::f32::consts;

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 320 * 2;
const HEIGHT: u32 = 240 * 2;
const FACTOR: u32 = 2;
const WINDOW_WIDTH: f64 = (WIDTH * FACTOR) as f64;
const WINDOW_HEIGHT: f64 = (HEIGHT * FACTOR) as f64;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        WindowBuilder::new()
            .with_title("Ulvestein")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame_mut());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            let mut redraw = false;

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
                redraw = true;
            }

            let left = input.key_held(VirtualKeyCode::Left);
            let right = input.key_held(VirtualKeyCode::Right);
            let forwards = input.key_held(VirtualKeyCode::Up);
            let backwards = input.key_held(VirtualKeyCode::Down);

            redraw = redraw || (left || right || forwards || backwards);


            if input.key_pressed(VirtualKeyCode::E) {
                redraw = world.set_dist(Dist::Euclid) || redraw;
            }
            if input.key_pressed(VirtualKeyCode::W) {
                redraw = world.set_dist(Dist::Perp) || redraw;
            }
            if input.key_pressed(VirtualKeyCode::Q) {
                redraw = world.set_dist(Dist::PerpQuick) || redraw;
            }

            if redraw {
                world.update(left, right, forwards, backwards);
                window.request_redraw();
            }
        }
    });
}

#[derive(PartialEq, Eq)]
enum Dist {
    Euclid,
    Perp,
    PerpQuick,
}

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    player_x: f32,
    player_y: f32,
    player_angle: f32,
    dist_method: Dist,
    grid: [[u8; 16]; 16],
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            player_x: 4.,
            player_y: 4.,
            player_angle: 0.,
            dist_method: Dist::Euclid,
            grid: [
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 2, 2, 2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 2, 2, 2, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            ]
        }
    }

    fn set_dist(&mut self, dist: Dist) -> bool {
        let changed = self.dist_method != dist;
        self.dist_method = dist;
        changed
    }

    /// Returns distance to wall from origin and wall material
    fn ray_cast(&self, origin_x: f32, origin_y: f32, angle: f32) -> (u8, Distance) {
        let (mut cx, mut cy) = (origin_x, origin_y);
        let (dy, dx) = angle.sin_cos();
        let (dy, dx) = (dy * 0.1, dx * 0.1);

        loop {
            let x = cx.ceil() as isize as usize;
            let y = cy.ceil() as isize as usize;
            let mat = self.grid.get(y).and_then(|a| a.get(x)).copied().unwrap_or(255);
            if mat != 0 {
                break (mat, Distance {
                    x1: origin_x,
                    y1: origin_y,
                    x2: cx,
                    y2: cy,
                });
            }

            cx += dx;
            cy += dy;
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self, left: bool, right: bool, forwards: bool, backwards: bool) {
        const TURN_SPEED: f32 = 0.05;
        const WALK_SPEED: f32 = 0.05;

        if left || right {
            self.player_angle += TURN_SPEED * (right as i8 - left as i8) as f32;
            self.player_angle %= consts::TAU;
        }

        if forwards || backwards {
            let (dy, dx) = self.player_angle.sin_cos();
            let speed = WALK_SPEED * (forwards as i8 - backwards as i8) as f32;
            self.player_x += speed * dx;
            self.player_y += speed * dy;
        }
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (x, angle) in (0..WIDTH).map(|x| (x, self.player_angle - consts::FRAC_PI_4 + consts::FRAC_PI_2 * x as f32 / WIDTH as f32)) {
            let (mat, dist) = self.ray_cast(self.player_x, self.player_y, angle);

            let dist = match self.dist_method {
                Dist::Euclid => dist.euclid(),
                Dist::Perp => dist.perp(self.player_angle),
                Dist::PerpQuick => dist.perp_d(angle - self.player_angle),
            };

            //Calculate height of line to draw on screen
            let line_height = (HEIGHT as f32 / dist) as u32;

            //calculate lowest and highest pixel to fill in current stripe
            let mut draw_start = -(line_height as i32) / 2 + HEIGHT as i32 / 2;
            if draw_start < 0 {
                draw_start = 0;
            }
            let draw_start = draw_start as u32;
            let mut draw_end = line_height / 2 + HEIGHT / 2;
            if draw_end >= HEIGHT {
                draw_end = HEIGHT - 1;
            }

            for y in 0..HEIGHT {
                let below_ceiling = draw_start <= y;
                let over_ground = y <= draw_end;

                let rgba = match (over_ground, below_ceiling) {
                    (true, true) => match mat {
                        1 => [0xff, 0xff, 0xff, 0xff],
                        2 => [0x00, 0xff, 0x00, 0xff],
                        255 => [0x00, 0x00, 0x00, 0xff],
                        _ => unreachable!(),
                    }
                    (true, false) => [0x00, 0x00, 0xff, 0xff],
                    (false, true) => [0xff, 0x00, 0x00, 0xff],
                    (false, false) => unreachable!(),
                };
                
                let i = coords_to_index(x, y);
                frame[i*4..i*4+4].copy_from_slice(&rgba);
            }
        }
    }
}

const fn index_to_coords(i: usize) -> (u32, u32) {
    let x = (i % WIDTH as usize) as u32;
    let y = (i / WIDTH as usize) as u32;

    (x, y)
}

const fn coords_to_index(x: u32, y: u32) -> usize {
    y as usize * WIDTH as usize + x as usize
}

#[test]
fn test() {
    let (x, y) = index_to_coords(124);
    assert_eq!(index_to_coords(124), index_to_coords(coords_to_index(x, y)));
    assert_eq!(124, coords_to_index(x, y));
}

struct Distance {
    x1: f32,
    y1: f32,

    x2: f32,
    y2: f32,
}

impl Distance {
    fn euclid(self) -> f32 {
        (self.x1-self.x2).hypot(self.y1-self.y2)
    }
    fn perp(self, angle: f32) -> f32 {
        let (face_y, face_x) = angle.sin_cos();
        
        (self.x2 - self.x1) * face_x + (self.y2 - self.y1) * face_y
    }
    fn perp_d(self, angle_diff: f32) -> f32 {
        self.euclid() * angle_diff.cos()
    }
}