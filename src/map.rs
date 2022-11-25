use std::{path::Path, fs::File, io::{BufReader, BufRead}, collections::HashMap};

use super::WIDTH;
use crate::{vec::*, Texture};

mod mat;
mod ray_caster;

pub use ray_caster::*;
pub use mat::*;

#[derive(Debug, Clone)]
pub struct Map {
    pub name: Box<str>,
    textures: Vec<(Texture, Texture)>,
    properties: Vec<Properties>,
    grid: Vec<Mat>,
    width: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Properties {
    solid: bool,
    transparent: bool,
    reflective: bool,
    door: bool,
}

impl Map {
    pub fn from_file<P: AsRef<Path>>(path: P) -> (Self, i32, i32, Side) {
        let f = BufReader::new(File::open(path).unwrap());
        let mut lines = f.lines();

        let name = lines.next().unwrap().unwrap().trim().to_owned().into_boxed_str();
        assert_eq!(lines.next().unwrap().unwrap().trim(), "");

        let mut textures = Vec::new();
        let mut properties = Vec::new();
        let mut material_map = HashMap::new();
        material_map.insert(' ', Mat::air());
        material_map.insert('<', Mat::air());
        material_map.insert('>', Mat::air());
        material_map.insert('^', Mat::air());
        material_map.insert('v', Mat::air());

        loop {
            match lines.next().unwrap().unwrap().trim() {
                "" => break,
                s => {
                    let mut elements = s.split_whitespace();
                    // TODO: check char length
                    let c = elements.next().unwrap().chars().next().unwrap();
                    let texture_dark = elements.next_back().unwrap();
                    let texture = elements.next_back().unwrap();

                    let (mut solid, mut transparent, mut reflective, mut door) = (true, false, false, false);

                    for property in elements {
                        match property {
                            "door" => door = true,
                            "solid" => solid = true,
                            "nonsolid" | "walkthrough" => solid = false,
                            "transparent" | "seethrough" => transparent = true,
                            "opaque" => transparent = false,
                            "reflective" => {
                                transparent = true;
                                reflective = true;
                            }
                            _ => panic!("uknown property {property} of texture {texture}"),
                        }
                    }

                    let texture = Texture::from_file(texture);
                    let texture_dark = Texture::from_file(texture_dark);
                    textures.push((texture, texture_dark));
                    properties.push(Properties {solid, transparent, reflective, door});

                    material_map.insert(c, Mat::from_len(textures.len()));
                }
            }
        }

        let mut grid = Vec::new();
        let mut width = 0;
        let mut player = None;

        for line in lines {
            let line = line.unwrap();
            let line = line.trim();
            let mut len = 0;

            for c in line.chars() {
                let mat = material_map[&c];
                grid.push(mat);

                if mat.is_air() {
                    let w = if width == 0 { i32::MAX } else { width };
                    let l = grid.len() as i32 - 1;
                    let (i, j) = (l % w, l / w);

                    match c {
                        '>' => player = Some((i, j, Side::Right)),
                        '<' => player = Some((i, j, Side::Left)),
                        '^' => player = Some((i, j, Side::Up)),
                        'v' => player = Some((i, j, Side::Down)),
                        _ => (),
                    }
                }
                
                len += 1;
            }
            if width == 0 {
                width = len;
            } else if width != len {
                panic!("this line was {len} long, but previous lines were all {width}");
            }
        }

        let (i, j, s) = player.expect("no player on map");

        (Self {
            name,
            textures,
            properties,
            grid,
            width,
        }, i, j, s)
    }

    pub fn get_tex(&self, mat: Mat, dark: bool) -> &Texture {
        let (light, non_light) = &self.textures[mat.index()];
        if dark {
            non_light
        } else {
            light
        }
    }
    pub fn get(&self, x: i32, y: i32) -> Option<Mat> {
        let x = x as isize as usize;
        let y = y as isize as usize;
        let w = self.width as isize as usize;

        let index = y.checked_mul(w)?.checked_add(x)?;
        self.grid.get(index).copied()
    }
    fn props(&self, mat: &Mat) -> Properties {
        if mat.is_air() { Properties { solid: false, transparent: true, reflective: false, door: false } } else {
            self.properties[mat.index()]
        }
    }

    /// Return the vector going into a solid material to be **clip**ped off
    pub fn move_ray_cast(&self, orig_p: Point2, dp: Vector2) -> Vector2 {
        let (clip, side) = ray_cast(orig_p, dp, true, 8,
            |x, y| self.get(x, y),
            |m| self.props(m).solid,
            |m| self.props(m).solid,
            |_| false,
            |m| !self.props(m).solid,
            false,
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
            |m| self.props(m).solid || !self.props(m).transparent,
            |m| !self.props(m).transparent,
            |m| self.props(m).reflective,
            |m| self.props(m).transparent,
            true,
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
