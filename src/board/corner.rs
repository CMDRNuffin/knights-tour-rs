use crate::aliases::BoardIndex as Idx;

#[derive(Clone, Copy, Debug)]
pub struct Corner {
    v: Idx,
    h: Idx,
}

impl Corner {
    pub fn vertical(&self) -> Idx {
        self.v
    }

    pub fn horizontal(&self) -> Idx {
        self.h
    }
}

impl From<(Idx, Idx)> for Corner {
    fn from((v, h): (Idx, Idx)) -> Self {
        Self { v, h }
    }
}

impl From<Idx> for Corner {
    fn from(val: Idx) -> Self {
        Self { v: val, h: val }
    }
}
