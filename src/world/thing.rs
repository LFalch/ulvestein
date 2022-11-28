use crate::{vec::{Point2, Vector2}, tex::{Frame, Texture}, HEIGHT};

use super::distance_line_circle;

#[derive(Debug, Copy, Clone)]
pub struct Thing {
    pub pos: Point2,
    width: f32,
    tex: usize,
}

impl Thing {
    pub fn new(pos: Point2, width: f32, tex: usize) -> Self {
        Thing { pos, width, tex }
    }
    pub fn draw_x(&self, frame: &mut Frame, x: u32, texes: &[Texture], last_dist: f32, p: Point2, dist: Vector2, height_factor: f32) {
        let f = distance_line_circle(p, dist, self.pos);
        let f_len = f.norm();

        if f_len <= self.width {
            let to_thing = self.pos - p;
            let u = 0.5 + f.dot(-to_thing.hat().set_len(self.width*2.)) / self.width;

            // Calculate height of line to draw on screen
            let line_height = height_factor / (last_dist + to_thing.norm());
            let line_height = if line_height.is_infinite() { i32::MAX } else { line_height as i32 }.abs();

            texes[self.tex].draw_line_at(frame, x, HEIGHT / 2, u, line_height as u32)
        }
    }
}
