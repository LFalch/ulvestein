#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mat {
    id: u8,
}

impl Mat {
    pub fn from_id(id: u8) -> Self {
        Mat { id }
    }
    pub fn id(&self) -> u8 {
        self.id
    }
    pub const fn invalid() -> Self {
        Mat { id: 255 }
    }
    pub fn is_solid(&self) -> bool {
        self.id != 0
    }
    pub fn is_opaque(&self) -> bool {
        self.id != 0
    }
    pub fn is_reflecive(&self) -> bool {
        self.id == 26
    }
}