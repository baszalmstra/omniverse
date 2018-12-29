use crate::planet::PatchLocation;
use crate::planet::PatchGeometry;

pub trait AsyncGeometryProvider {
    fn queue(&self, patch: PatchLocation);
}

struct Token {
    pub priority: u32,
}

struct ThreadpoolGeometryProvider {
    
}
