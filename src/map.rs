use super::WIDTH;
use crate::vec::*;

mod mat;

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

    // Taken from topskud
    pub fn ray_cast(&self, from: Point2, dist: Vector2, finite: bool) -> RayCast {
        let dest = from + dist;

        let mut cur = from;
        let mut to_wall = Vector2::new(0., 0.);
        let (mut gx, mut gy) = (cur.x.floor() as i32, cur.y.floor() as i32);
        let x_dir = Direction::new(dist.x);
        let y_dir = Direction::new(dist.y);

        loop {
            if finite && (cur - dest).dot(dist) / dist.norm() >= 0. {
                break RayCast::n_full(dest);
            }

            let mat = self.get(gx, gy);

            if let Some(mat) = mat {
                if mat.is_solid() {
                    break RayCast::n_half(mat, cur, dest-cur, to_wall);
                }
                if cur.x < 0. || cur.y < 0. {
                    break RayCast::n_off_edge(cur, dest-cur); 
                }
            } else {
                break RayCast::n_off_edge(cur, dest-cur);
            }

            let nearest_corner = Point2::new(x_dir.on(gx as f32), y_dir.on(gy as f32));
            let distance = nearest_corner - cur;

            let time = (distance.x/dist.x, distance.y/dist.y);

            if time.0 < time.1 {
                to_wall.x = dist.x.signum();
                to_wall.y = 0.;
                // Going along x
                cur.x = nearest_corner.x;
                cur.y += time.0 * dist.y;

                gx = if let Some(n) = x_dir.on_i32(gx) {
                    n
                } else {
                    break RayCast::n_off_edge(cur, dest-cur);
                }
            } else {
                if time.0 - time.1 < std::f32::EPSILON {
                    to_wall.x = dist.x.signum();
                    to_wall.y = dist.y.signum();
                } else {
                    to_wall.x = 0.;
                    to_wall.y = dist.y.signum();
                }
                // Going along y
                cur.y = nearest_corner.y;
                cur.x += time.1 * dist.x;

                gy = if let Some(n) = y_dir.on_i32(gy) {
                    n
                } else {
                    break RayCast::n_off_edge(cur, dest-cur);
                }
            }
        }
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

#[derive(Debug, Copy, Clone)]
enum Direction {
    Pos,
    Neg,
}

impl Direction {
    #[inline]
    fn new(n: f32) -> Self {
        if n.is_sign_negative() {
            Direction::Neg
        } else {
            Direction::Pos
        }
    }
    #[inline]
    fn on_i32(self, n: i32) -> Option<i32> {
        match self {
            Direction::Pos => Some(n + 1),
            Direction::Neg => (n as u32).checked_sub(1).map(|i| i as i32),
        }
    }
    #[inline]
    fn on(self, n: f32) -> f32 {
        match self {
            Direction::Pos => n + 1.,
            Direction::Neg => n,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RayCast {
    result: RayCastResult,
    point: Point2,
    clip: Vector2,
}

#[derive(Debug, Copy, Clone)]
enum RayCastResult {
    Full,
    Half(Mat, Vector2),
    OffEdge,
}

impl RayCast {
    const fn n_full(point: Point2) -> Self {
        RayCast{
            result: RayCastResult::Full,
            point,
            clip: Vector2::new(0., 0.)
        }
    }
    const fn n_half(mat: Mat, point: Point2, clip: Vector2, to_wall: Vector2) -> Self {
        RayCast{
            result: RayCastResult::Half(mat, to_wall),
            point,
            clip,
        }
    }
    const fn n_off_edge(point: Point2, clip: Vector2) -> Self {
        RayCast{
            result: RayCastResult::OffEdge,
            point,
            clip,
        }
    }

    pub const fn full(self) -> bool {
        match self.result {
            RayCastResult::Full => true,
            _ => false,
        }
    }
    pub const fn half(self) -> bool {
        match self.result {
            RayCastResult::Half(_, _) => true,
            _ => false,
        }
    }
    pub const fn half_vec(self) -> Option<Vector2> {
        match self.result {
            RayCastResult::Half(_, v) => Some(v),
            _ => None,
        }
    }
    pub const fn material(self) -> Option<Mat> {
        match self.result {
            RayCastResult::Half(m, _) => Some(m),
            _ => None,
        }
    }
    pub fn into_point(self) -> Point2 {
        let Self{point, ..} = self;
        point
    }
    pub fn clip(self) -> Vector2 {
        let Self{clip, ..} = self;
        clip
    }
}
