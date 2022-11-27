use std::collections::VecDeque;
use std::time::Instant;

use log::{error, info};
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

pub mod vec;
pub mod map;
pub mod fov;
pub mod tex;
pub mod world;

use self::tex::*;
use self::world::*;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const FACTOR: u32 = 4;
const FOV: f32 = 65.;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        WindowBuilder::new()
            .with_title("Ulvestein")
            .with_inner_size(LogicalSize::new(FACTOR * WIDTH, FACTOR * HEIGHT))
            .with_min_inner_size(LogicalSize::new(WIDTH, HEIGHT))
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    let mut last_draw = Instant::now();
    let mut last_fpss = VecDeque::new();

    let mut last_update = last_draw;

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(Frame::from_pixels(&mut pixels));

            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }

            let now = Instant::now();
            let fps = 1. / (now - last_draw).as_secs_f64();
            last_fpss.push_back(fps);
            while last_fpss.len() > 4 {
                last_fpss.pop_front();
            }
            let avg_fps = last_fpss.iter().copied().sum::<f64>() / last_fpss.len() as f64;
            window.set_title(&format!("Ulvestein - FPS {avg_fps:.0}"));
            last_draw = now;
        }

        // Handle input events
        if input.update(&event) {
            let now = Instant::now();
            let delta = (now - last_update).as_secs_f32();

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
                world.fov.change_fov(5.);
            }
            if input.key_pressed_os(VirtualKeyCode::Minus) {
                world.fov.change_fov(-5.);
            }

            world.update(delta, left, right, forwards, backwards, go_left, go_right);
            window.request_redraw();
            last_update = now;
        }
    });
}
