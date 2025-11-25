#[derive(Debug)]
pub struct Molecule {
    pub addr: String,
    pub port: u32,
}

impl Molecule {
    pub fn new(addr: String, port: u32) -> Self {
        Self { addr, port }
    }
}
