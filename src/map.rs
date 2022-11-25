use super::WIDTH;
use crate::vec::*;

mod mat;
mod ray_caster;

pub use ray_caster::*;
pub use mat::*;

pub struct Map<const N: usize, const M: usize> {
    pub grid: [[Mat; N]; M],
}

impl<const N: usize, const M: usize> Map<N, M> {
    pub fn new(arg: [[u8; N]; M]) -> Map<N, M> {
        Map { grid: arg.map(|a| a.map(|id| Mat::from_id(id))) }
    }

    fn get(&self, x: i32, y: i32) -> Option<Mat> {
        self.grid.get(y as u32 as usize).and_then(|a| a.get(x as u32 as usize)).copied()
    }

    /// Return the vector going into a solid material to be **clip**ped off
    pub fn move_ray_cast(&self, orig_p: Point2, dp: Vector2) -> Vector2 {
        let (clip, side) = ray_cast(orig_p, dp, true, 8,
            |x, y| self.get(x, y),
            |n| n.is_solid(),
            |n| n.is_solid(),
            |_| false,
            |n| !n.is_solid(),
        ).clip();

        const PUSH: f32 = 0.005;

        if let Some(side) = side {
            let wall_dir = side.flip().into_unit_vector();
            let to_wall = clip.proj(wall_dir);
            to_wall + PUSH * wall_dir
        } else { clip }
    }

    /// Returns a vector of (dark, u, distance, material) in order of increasing distance
    /// that show what the ray encountered travelling in this direction
    ///
    /// Since rays do not stop at every node, this is a list and should be drawn in reverse order
    pub fn render_ray_cast(&self, orig_p: Point2, dp: Vector2) -> Vec<(bool, f32, f32, Mat)> {
        let cast = ray_cast(orig_p, dp, false, 8,
            |x, y| self.get(x, y),
            |n| n.is_solid(),
            |n| n.is_opaque(),
            |n| n.is_reflective(),
            |n| !n.is_solid(),
        );

        let mut last_point = orig_p;
        let mut total_distance = 0.;

        cast.into_iter().filter_map(|cp| {
                total_distance += (cp.point - last_point).norm();
                last_point = cp.point;
                let dist = total_distance;

                match cp.cast_type {
                    CastPointType::Void(_) => None,
                    // TODO: fix reflection
                    CastPointType::Reflection(mat, side)
                    | CastPointType::Pass(mat, side)
                    | CastPointType::Termination(mat, side) => {
                        let dark = matches!(side, Side::Left | Side::Right);
                        let u = match side {
                            Side::Left => cp.point.y.fract(),
                            Side::Up => 1. - cp.point.x.fract(),
                            Side::Right => 1. - cp.point.y.fract(),
                            Side::Down => cp.point.x.fract(),
                        };

                        Some((dark, u, dist, mat))
                    }
                    CastPointType::Destination => unreachable!(),
                }
            }).collect::<Vec<_>>()
    }
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
