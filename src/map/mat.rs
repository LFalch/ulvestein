#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mat {
    id: u8,
}

impl Mat {
    pub(super) const fn air() -> Self {
        Mat { id: 0 }
    }
    pub(super) const fn is_air(self) -> bool {
        self.id == 0
    }
    pub(super) fn from_len(index: usize) -> Self {
        debug_assert_ne!(index, 0, "air should be constructed with ::air()");
        Mat { id: index as u8 }
    }
    pub(super) fn index(&self) -> usize {
        self.id as usize - 1
    }
}
