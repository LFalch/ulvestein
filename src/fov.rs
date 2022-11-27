use log::info;

use super::{WIDTH, HEIGHT};

#[derive(Debug, Copy, Clone)]
pub struct Fov {
    pub fov: f32,
    pub fov_vert: f32,
    /// Tangent of half of the FOV (used for finding the coordinates of the first ray)
    pub tan_half_fov: f32,
    /// Projected height of wall with height of 1 at distance of 1
    pub height_coefficient: f32,
}

impl Fov {
    pub fn new_from_degrees(fov_deg: f32) -> Self {
        let fov = fov_deg.to_radians();
        let fov_vert = 2. * (HEIGHT as f32 / WIDTH as f32 * (0.5 * fov).sin()).atan();
        let tan_half_fov = (0.5 * fov).tan();
        // Happens to also be the same as the distance to the projection plane
        let height_coefficient = 0.5 * WIDTH as f32 / (0.5 * fov).sin();

        Fov {
            fov,
            fov_vert,
            tan_half_fov,
            height_coefficient,
        }
    }
    pub fn change_fov(&mut self, deg_diff: f32) {
        *self = Self::new_from_degrees(self.fov.to_degrees() + deg_diff);
        info!("fov: {:.0} - {:.0}", self.fov.to_degrees(), self.fov_vert.to_degrees());
    }
}
