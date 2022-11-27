use std::f32::consts;

use log::info;

use crate::{map::Map, tex::{Texture, Colour, Frame}, vec::{Point2, Vector2}, fov::Fov, WIDTH, HEIGHT, FOV};

/// Representation of the application state. In this example, a box will bounce around the screen.
pub struct World {
    player_p: Point2,
    player_angle: f32,
    pub map: Map,
    pub fov: Fov,
    pub gun: Texture,
    pub clip: bool,
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    pub fn new() -> Self {
        let (map, x, y, s) = Map::from_file("map.txt");
        info!("Map name: {}", map.name);
        Self {
            map,
            player_p: Point2::new(x as f32 + 0.5, y as f32 + 0.5),
            player_angle: s.into_unit_vector().direction_angle(),
            fov: Fov::new_from_degrees(FOV),
            clip: true,
            gun: Texture::from_file("tex/gun.png"),
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    pub fn update(&mut self, delta: f32, left: bool, right: bool, forwards: bool, backwards: bool, go_left: bool, go_right: bool) {
        const TURN_SPEED: f32 = 105.  /* degrees */ / 180. * consts::PI;
        const WALK_SPEED: f32 = 2.3;

        if left || right {
            self.player_angle += delta * TURN_SPEED * (right as i8 - left as i8) as f32;
            self.player_angle %= consts::TAU;
        }

        if (forwards ^ backwards) || (go_left ^ go_right) {
            let dv = Vector2::unit_from_angle(self.player_angle);
            let dp = dv * (forwards as i8 - backwards as i8) as f32 + dv.hat() * (go_right as i8 - go_left as i8) as f32;
            let dp = dp.set_len(delta * WALK_SPEED);

            let orig_p = self.player_p;

            self.player_p = self.player_p + dp;

            if self.clip {
                self.player_p = self.player_p - self.map.move_ray_cast(orig_p, dp);
            }
        }
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    pub fn draw(&self, mut frame: Frame) {
        let dir = Vector2::unit_from_angle(self.player_angle);

        // Unit vector pointing to the right
        let right_dir = dir.hat();
        const HALF_WIDTH: f32 = (WIDTH / 2) as f32;
        let first_ray = dir / self.fov.tan_half_fov - dir.hat();

        for (x, ray) in (0..WIDTH).map(|x| (x, first_ray + right_dir * (x as f32 / HALF_WIDTH))) {
            let lines = self.map.render_ray_cast(self.player_p, ray);

            let fisheye_correction_factor = ray.dot(dir) / ray.norm();

            for (dark, u, dist, mat) in lines.into_iter().rev() {
                // Calculate height of line to draw on screen
                let line_height = self.fov.height_coefficient / dist / fisheye_correction_factor;
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
                        (true, true) => {
                            let tex = self.map.get_tex(mat, dark);
                            let v = (y - mat_top) as f32 / (mat_bot - mat_top) as f32;

                            tex.get_pixel_f(u, v)
                        }
                        (true, false) => Colour::new(0x00, 0x00, 0xff).alpha(0xff),
                        (false, true) => Colour::new(0xff, 0x00, 0x00).alpha(0xff),
                        (false, false) => Colour::new(0xff, 0xff, 0xff).alpha(0x07),
                    };

                    frame.draw_rgba(x, y as u32, c);
                }
            }
        }

        let gun_x = (WIDTH - self.gun.width() as u32) / 2;
        let gun_y = HEIGHT - self.gun.height() as u32;
        self.gun.draw_at(&mut frame, gun_x, gun_y);
    }
}
