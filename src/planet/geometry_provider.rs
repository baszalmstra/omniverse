use crate::planet::Face;
use nalgebra::{Point2, Point3, Vector3};
use crate::planet::quad_tree;

/// Location of a patch in the oriented unit quad.
#[derive(Debug, Copy, Clone)]
pub struct PatchLocation {
    pub face: Face,

    /// Offset from the top-left corner of the face
    pub offset: Point2<f64>,

    /// 1 is the entire face
    pub size: f64,

    /// The lod level of this patch, higher means more detail
    pub lod_level: usize
}

impl PatchLocation {
    pub fn split(&self, index: quad_tree::Child) -> PatchLocation {
        let size = self.size * 0.5;
        let face = self.face;
        match index {
            quad_tree::Child::TopLeft => PatchLocation {
                face,
                offset: Point2::new(self.offset.x, self.offset.y),
                size,
                lod_level: self.lod_level + 1,
            },
            quad_tree::Child::TopRight => PatchLocation {
                face,
                offset: Point2::new(self.offset.x + size, self.offset.y),
                size,
                lod_level: self.lod_level + 1,
            },
            quad_tree::Child::BottomLeft => PatchLocation {
                face,
                offset: Point2::new(self.offset.x, self.offset.y + size),
                size,
                lod_level: self.lod_level + 1,
            },
            quad_tree::Child::BottomRight => PatchLocation {
                face,
                offset: Point2::new(self.offset.x + size, self.offset.y + size),
                size,
                lod_level: self.lod_level + 1,
            },
        }
    }

    pub fn top_left(&self) -> PatchLocation {
        self.split(quad_tree::Child::TopLeft)
    }

    pub fn top_right(&self) -> PatchLocation {
        self.split(quad_tree::Child::TopRight)
    }

    pub fn bottom_left(&self) -> PatchLocation {
        self.split(quad_tree::Child::BottomLeft)
    }

    pub fn bottom_right(&self) -> PatchLocation {
        self.split(quad_tree::Child::BottomRight)
    }
}

impl Into<PatchLocation> for Face {
    fn into(self) -> PatchLocation {
        PatchLocation {
            face: self,
            offset: Point2::new(0.0, 0.0),
            size: 1.0,
            lod_level: 0
        }
    }
}

/// Geometry of a single patch
pub struct PatchGeometry {
    pub positions: Vec<Point3<f64>>,
    pub normals: Vec<Vector3<f64>>,
}

pub trait GeometryProvider {
    fn compute_geometry(&self, patch: PatchLocation) -> PatchGeometry;
    fn position_at(&self, face: Face, offset: Point2<f64>) -> Point3<f64>;
}
