use std::ops::{Add, Sub, Mul};

#[derive(Debug, Copy, Clone)]
pub struct Vector2{
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub const fn new(x: f32, y: f32) -> Self { Vector2 {x, y} }
    pub fn norm(&self) -> f32 {
        self.x.hypot(self.y)
    }
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }
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
}

pub type Point2 = Vector2;

impl Add for Vector2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Vector2::new(self.x+rhs.x, self.y+rhs.y)
    }
}

impl Sub for Vector2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Vector2::new(self.x-rhs.x, self.y-rhs.y)
    }
}

impl Mul<Vector2> for f32 {
    type Output = Vector2;
    fn mul(self, rhs: Vector2) -> Self::Output {
        Vector2::new(rhs.x*self, rhs.y*self)
    }
}

impl Mul<f32> for Vector2 {
    type Output = Vector2;
    fn mul(self, rhs: f32) -> Self::Output {
        Vector2::new(self.x*rhs, self.y*rhs)
    }
}