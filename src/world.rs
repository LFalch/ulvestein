use std::f32::consts;

use log::info;

use crate::{map::Map, tex::{Texture, Colour, Frame}, vec::{Point2, Vector2}, fov::Fov, WIDTH, HEIGHT, FOV};

pub mod thing;

use self::thing::*;

/// Representation of the application state. In this example, a box will bounce around the screen.
pub struct World {
    player_p: Point2,
    player_angle: f32,
    things: Vec<Thing>,
    thing_texes: Vec<Texture>,
    pub map: Map,
    pub fov: Fov,
    pub gun: Texture,
    pub clip: bool,
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    pub fn new() -> Self {
        let (map, x, y, s, things, mut thing_texes) = Map::from_file("map.txt");
        info!("Map name: {}", map.name);

        thing_texes.push(Texture::from_file("tex/player.png"));

        Self {
            map,
            things,
            thing_texes,
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
        let ref player_thing = Thing::new(self.player_p, 0.25, self.thing_texes.len()-1);
        let mut things = Vec::with_capacity(self.things.len()+1);

        let dir = Vector2::unit_from_angle(self.player_angle);

        // Unit vector pointing to the right
        let right_dir = dir.hat();
        const HALF_WIDTH: f32 = (WIDTH / 2) as f32;
        let first_ray = dir / self.fov.tan_half_fov - dir.hat();

        for (x, ray) in (0..WIDTH).map(|x| (x, first_ray + right_dir * (x as f32 / HALF_WIDTH))) {
            let lines = self.map.render_ray_cast(self.player_p, ray);
            let line_len = lines.len();
            let mut i = 0;

            let fisheye_correction_factor = ray.dot(dir) / ray.norm();

            for (dark, u, for_things, dist, mat) in lines.into_iter().rev() {
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
                        (true, false) => Colour::new(0x00, 0x00, 0xff).alpha(0xff),
                        (false, true) => Colour::new(0xff, 0x00, 0x00).alpha(0xff),
                        _ => {
                            let tex = self.map.get_tex(mat, dark);
                            let v = (y - mat_top) as f32 / (mat_bot - mat_top) as f32;

                            tex.get_pixel_f(u, v)
                        }
                    };

                    frame.draw_rgba(x, y as u32, c);
                }

                let (p, dist, last_dist) = for_things;

                let height_factor = 0.5 * self.fov.height_coefficient;
                things.clear();
                i += 1;
                if i != line_len {
                    things.push(player_thing);
                };
                for thing in &self.things {
                    let dist = (thing.pos - p).norm();
                    let i = things.binary_search_by(|t| (t.pos - p).norm().total_cmp(&dist).reverse()).unwrap_or_else(|e| e);
                    things.insert(i, thing);
                }

                for thing in &things {
                    thing.draw_x(&mut frame, x, &self.thing_texes, last_dist, p, dist, height_factor);
                }
            }
        }

        let gun_x = (WIDTH - self.gun.width() as u32) / 2;
        let gun_y = HEIGHT - self.gun.height() as u32;
        self.gun.draw_at(&mut frame, gun_x, gun_y);
    }
}

/// Closest point on a line segment to a circle
pub fn closest_point_of_line_to_circle(line_start: Point2, line_dist: Vector2, circle_center: Point2) -> Point2 {
    let c = circle_center - line_start;

    let d_len = line_dist.norm();

    let c_on_d_len = c.dot(line_dist) / d_len;

    if c_on_d_len < 0. {
        // Closest point is start point
        line_start
    } else if c_on_d_len <= d_len {
        // Closest point is betweeen start and end point
        let c_on_d = c_on_d_len / d_len * line_dist;
        line_start + c_on_d
    } else {
        // Closest point is end point
        line_start + line_dist
    }
}
/// Distance between a line section and a circle
/// 
/// The general formula for distance between a line and cirlcle here would be inadequate
/// since here the line has a finite length so we need to check if the smalleset distance is in that finite line section.
#[inline]
pub fn distance_line_circle(line_start: Point2, line_dist: Vector2, circle_center: Point2) -> Vector2 {
    let closest_point = closest_point_of_line_to_circle(line_start, line_dist, circle_center);

    circle_center.vector_to(closest_point)
}
/// Length of `distance_line_circle`
#[inline]
pub fn dist_line_circle(line_start: Point2, line_dist: Vector2, circle_center: Point2) -> f32 {
    distance_line_circle(line_start, line_dist, circle_center).norm()
}
