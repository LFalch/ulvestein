use std::ops::{Add, Sub, Mul, Neg, Div};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self { Vector2 {x, y} }
    #[inline(always)]
    pub fn norm(&self) -> f32 {
        self.x.hypot(self.y)
    }
    #[inline(always)]
    pub fn unit_from_angle(angle: f32) -> Self {
        let (y, x) = angle.sin_cos();
        Self::new(x, y)
    }
    #[inline(always)]
    pub fn direction_angle(self) -> f32 {
        self.y.atan2(self.x)
    }
    #[inline(always)]
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }
    /// (x, y) -> (-y, x)
    ///
    /// Turns the vector ninety degrees
    #[inline(always)]
    pub fn hat(self) -> Self {
        Vector2 { x: -self.y, y: self.x }
    }
    pub fn set_len(self, len: f32) -> Self {
        let scale = len / self.norm();
        if scale.is_finite() {
            scale * self
        } else {
            Vector2::new(0., 0.)
        }
    }
    pub fn proj(self, other: Self) -> Vector2 {
        let divisor = 1. / other.dot(other);
        if divisor.is_finite() {
            self.dot(other) * other * divisor
        } else {
            Vector2::new(0., 0.)
        }
    }
}

impl Neg for Vector2 {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self::Output {
        Vector2::new(-self.x, -self.y)
    }
}

impl Add for Vector2 {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Vector2::new(self.x+rhs.x, self.y+rhs.y)
    }
}

impl Sub for Vector2 {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Vector2::new(self.x-rhs.x, self.y-rhs.y)
    }
}

impl Mul<Vector2> for f32 {
    type Output = Vector2;
    #[inline(always)]
    fn mul(self, rhs: Vector2) -> Self::Output {
        Vector2::new(rhs.x*self, rhs.y*self)
    }
}

impl Mul<f32> for Vector2 {
    type Output = Vector2;
    #[inline(always)]
    fn mul(self, rhs: f32) -> Self::Output {
        Vector2::new(self.x*rhs, self.y*rhs)
    }
}

impl Div<f32> for Vector2 {
    type Output = Vector2;
    #[inline(always)]
    fn div(self, rhs: f32) -> Self::Output {
        Vector2::new(self.x/rhs, self.y/rhs)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    pub const ORIGIN: Self = Self::new(0., 0.);
    pub const fn new(x: f32, y: f32) -> Self {
        Point2 { x, y }
    }
    #[inline(always)]
    pub fn vector_to(self, other: Self) -> Vector2 {
        other - self
    }
}

impl Add<Vector2> for Point2 {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Vector2) -> Self::Output {
        Point2::new(self.x+rhs.x, self.y+rhs.y)
    }
}

impl Add<Point2> for Vector2 {
    type Output = Point2;
    #[inline(always)]
    fn add(self, rhs: Point2) -> Self::Output {
        Point2::new(self.x+rhs.x, self.y+rhs.y)
    }
}

impl Sub<Vector2> for Point2 {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Vector2) -> Self::Output {
        Point2::new(self.x-rhs.x, self.y-rhs.y)
    }
}

impl Sub<Point2> for Vector2 {
    type Output = Point2;
    #[inline(always)]
    fn sub(self, rhs: Point2) -> Self::Output {
        Point2::new(self.x-rhs.x, self.y-rhs.y)
    }
}

impl Sub for Point2 {
    type Output = Vector2;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Vector2::new(self.x-rhs.x, self.y-rhs.y)
    }
}
