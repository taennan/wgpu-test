pub enum Corner {
    Nw,
    Ne,
    Sw,
    Se,
}

impl From<Corner> for u32 {
    fn from(corner: Corner) -> u32 {
        match corner {
            Corner::Nw => 0,
            Corner::Ne => 1,
            Corner::Sw => 2,
            Corner::Se => 3,
        }
    }
}
