use std::f32::consts;

use log::{error, info};
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const FACTOR: u32 = 4;
const WINDOW_WIDTH: f64 = (WIDTH * FACTOR) as f64;
const WINDOW_HEIGHT: f64 = (HEIGHT * FACTOR) as f64;
const FOV: f32 = 90.;

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
            let forwards = input.key_held(VirtualKeyCode::Up) || input.key_held(VirtualKeyCode::W);
            let backwards = input.key_held(VirtualKeyCode::Down) || input.key_held(VirtualKeyCode::S);
            let go_right = input.key_held(VirtualKeyCode::D);
            let go_left = input.key_held(VirtualKeyCode::A);

            redraw = redraw || (left || right || forwards || backwards || go_left || go_right);

            if input.key_pressed_os(VirtualKeyCode::Plus) {
                world.fov += 5.;
                info!("fov: {}", world.fov);
                redraw = true;
            }
            if input.key_pressed_os(VirtualKeyCode::Minus) {
                world.fov -= 5.;
                info!("fov: {}", world.fov);
                redraw = true;
            }

            if redraw {
                world.update(left, right, forwards, backwards, go_left, go_right);
                window.request_redraw();
            }
        }
    });
}

pub mod map;

use self::map::*;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    player_p: Point2,
    player_angle: f32,
    map: Map<16, 16>,
    fov: f32,
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            player_p: Point2::new(6., 6.),
            player_angle: 0.,
            map: Map {grid: [
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
            ]},
            fov: FOV,
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
            if cast.full() {
                self.player_p = cast.into_point();
            }
        }
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        let fov_rad = self.fov * consts::PI / 180.;
        for (x, angle) in (0..WIDTH).map(|x| (x, self.player_angle - 0.5 * fov_rad + fov_rad * x as f32 / WIDTH as f32)) {
            let (mat, dist) = {
                let (y, x) = angle.sin_cos();
                let cast = self.map.ray_cast(self.player_p, Vector2::new(x, y), false);

                (cast.material().unwrap_or(255), (cast.into_point() - self.player_p).norm() * (angle - self.player_angle).cos())
            };

            //Calculate height of line to draw on screen
            let line_height = HEIGHT as f32 / dist;
            let line_height = if line_height.is_infinite() { HEIGHT } else { line_height as u32 };

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
                    (true, true) => rgba(mat),
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
