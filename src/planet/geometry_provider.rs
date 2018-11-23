use nalgebra::{Point2, Point3};
use planet::constants::VERTICES_PER_PATCH;
use planet::Face;

/// Location of a patch in the oriented unit quad.
pub struct PatchLocation {
    pub face: Face,

    /// Offset from the top-left corner of the face
    pub offset: Point2<f64>,

    /// 1 is the entire face
    pub size: f64,
}

/// Geometry of a single patch
pub struct PatchGeometry {
    pub positions: Vec<Point3<f64>>,
}

pub trait GeometryProvider {
    fn provide(&self, patch: PatchLocation) -> PatchGeometry;
}
