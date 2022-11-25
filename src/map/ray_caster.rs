use crate::vec::{Point2, Vector2};

pub fn ray_cast<M, FG, FN, FT, FR, FP>(from: Point2, dist: Vector2, finite: bool, node_limit: usize, get_mat: FG, is_node: FN,
    is_terminator: FT, is_reflector: FR, is_pass_througher: FP, skip_first_check: bool) -> CastPoints<M>
where FG: Fn(i32, i32) -> Option<M>, FN: Fn(&M) -> bool, FT: Fn(&M) -> bool, FR: Fn(&M) -> bool, FP: Fn(&M) -> bool {
    let dest = from + dist;

    let mut cur = from;
    let (mut gx, mut gy) = (cur.x.floor() as i32, cur.y.floor() as i32);
    let x_dir = Direction::new(dist.x);
    let y_dir = Direction::new(dist.y);
    
    // If you're on a grid boundary, make sure you are only stuck on the wall if you're going towards it
    if cur.x.fract() == 0. && x_dir == Direction::Neg {
        gx -= 1;
    }
    if cur.y.fract() == 0. && y_dir == Direction::Neg {
        gy -= 1;
    }


    let mut points = Vec::with_capacity(2);

    let mut side = Side::from_vec(dist);

    let mut do_mat_check = !skip_first_check;

    loop {
        if points.len() >= node_limit {
            break;
        }

        if finite && (cur - dest).dot(dist) / dist.norm() >= 0. {
            points.push(CastPoint::dest(dest));
            break;
        }

        if do_mat_check {
            if cur.x < 0. || cur.y < 0. {
                points.push(CastPoint::void(cur, side));
                break; 
            }

            let mat = get_mat(gx, gy);

            if let Some(mat) = mat {
                if is_node(&mat) {
                    if is_terminator(&mat) {
                        points.push(CastPoint::terminated(cur, mat, side));
                        break;
                    } else if is_reflector(&mat) {
                        points.push(CastPoint::reflect(cur, mat, side));

                        let mut dist = if finite { dest - cur } else { dist };
                        match side {
                            Side::Left | Side::Right => dist.x = -dist.x,
                            Side::Up | Side::Down => dist.y = -dist.y,
                        }

                        let cps = ray_cast(cur, dist, finite, node_limit-points.len(), get_mat, is_node, is_terminator, is_reflector, is_pass_througher, false);
                        points.extend(cps);

                        break;
                    } else if is_pass_througher(&mat) {
                        points.push(CastPoint::pass(cur, mat, side));
                    }
                }
            } else {
                points.push(CastPoint::void(cur, side));
                break;
            }
        }
        do_mat_check = true;

        let nearest_corner = Point2::new(x_dir.on(gx as f32), y_dir.on(gy as f32));
        let distance = nearest_corner - cur;

        // Time until we hit the next corner in the x and y direction respectively
        let time = (distance.x/dist.x, distance.y/dist.y);

        if time.0 < time.1 {
            side = Side::along_x(dist.x.is_sign_positive());
            // Going along x
            cur.x = nearest_corner.x;
            cur.y += time.0 * dist.y;

            gx = x_dir.on_i32(gx);
        } else {
            side = Side::along_y(dist.y.is_sign_positive());
            // Going along y
            cur.y = nearest_corner.y;
            cur.x += time.1 * dist.x;

            gy = y_dir.on_i32(gy);
        }
    }

    let target;

    if finite {
        target = Some(dest);
        if let Some(CastPointType::Void(_)) = points.last().map(|p| &p.cast_type) {
            points.push(CastPoint::dest(dest));
        }
    } else {
        target = None;
    }

    CastPoints {
        origin: from,
        target,
        inner: points,
    }
}

#[derive(Debug, Clone)]
pub struct CastPoints<M> {
    inner: Vec<CastPoint<M>>,
    pub origin: Point2,
    pub target: Option<Point2>,
}

impl<M> CastPoints<M> {
    pub fn clip(&self) -> (Vector2, Option<Side>) {
        let target = self.target.expect("clip only makes sense on finite casts");

        let mut point = Point2::new(f32::NAN, f32::NAN);
        let mut side = None;
        for cp in &self.inner {
            point = cp.point;
            match cp.cast_type {
                CastPointType::Reflection(_, s) | CastPointType::Pass(_, s) | CastPointType::Termination(_, s) => {
                    side = Some(s);
                    break;
                }
                CastPointType::Void(s) => side = Some(s),
                CastPointType::Destination => side = None,
            }
        }

        (target-point, side)
    }
}

impl<M> IntoIterator for CastPoints<M> {
    type Item = CastPoint<M>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct CastPoint<M> {
    pub point: Point2,
    pub cast_type: CastPointType<M>,
}

impl<M> CastPoint<M> {
    const fn terminated(point: Point2, mat: M, side: Side) -> Self {
        CastPoint { point, cast_type: CastPointType::Termination(mat, side) }
    }
    const fn dest(point: Point2) -> Self {
        CastPoint { point, cast_type: CastPointType::Destination }
    }
    const fn void(point: Point2, side: Side) -> Self {
        CastPoint { point, cast_type: CastPointType::Void(side) }
    }
    const fn reflect(point: Point2, mat: M, side: Side) -> Self {
        CastPoint { point, cast_type: CastPointType::Reflection(mat, side) }
    }
    const fn pass(point: Point2, mat: M, side: Side) -> Self {
        CastPoint { point, cast_type: CastPointType::Pass(mat, side) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CastPointType<M> {
    /// Indicates that the cast was reflected by a reflective material
    Reflection(M, Side),
    /// Encountered a solid, see-through material here, also end point if edge is reached
    Pass(M, Side),
    /// Encountered the void, end point if non-finite
    Void(Side),
    /// Ray cast hit a solid, opaue material, end point
    Termination(M, Side),
    /// Reached its destination, only finite casts, end point
    Destination,
}

#[repr(i8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Side {
    Right,
    Down,
    Left,
    Up,
}

impl Side {
    const fn along_x(x_positive: bool) -> Self {
        if x_positive {
            Self::Left
        } else {
            Self::Right
        }
    }
    const fn along_y(y_positive: bool) -> Self {
        if y_positive {
            Self::Up
        } else {
            Self::Down
        }
    }
    fn from_vec(dist: Vector2) -> Side {
        if dist.x.abs() > dist.y.abs() {
            Self::along_x(dist.x.is_sign_positive())
        } else {
            Self::along_y(dist.y.is_sign_positive())
        }
    }

    pub const fn flip(self) -> Self {
        match self {
            Side::Right => Side::Left,
            Side::Down => Side::Up,
            Side::Left => Side::Right,
            Side::Up => Side::Down,
        }
    } 

    pub const fn into_unit_vector(self) -> Vector2 {
        match self {
            Side::Right => Vector2::new(1., 0.),
            Side::Down => Vector2::new(0., 1.),
            Side::Left => Vector2::new(-1., 0.),
            Side::Up => Vector2::new(0., -1.),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    fn on_i32(self, n: i32) -> i32 {
        match self {
            Direction::Pos => n + 1,
            Direction::Neg => n - 1,
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
