#[derive(Clone, Copy, Debug)]
pub struct Corner {
    v: u16,
    h: u16,
}

impl Corner {
    pub fn new(v: u16, h: u16) -> Self {
        Self { v, h }
    }

    pub fn vertical(&self) -> u16 {
        self.v
    }

    pub fn horizontal(&self) -> u16 {
        self.h
    }
}

impl From<(u16, u16)> for Corner {
    fn from((v, h): (u16, u16)) -> Self {
        Self { v, h }
    }
}

impl From<u16> for Corner {
    fn from(val: u16) -> Self {
        Self { v: val, h: val }
    }
}