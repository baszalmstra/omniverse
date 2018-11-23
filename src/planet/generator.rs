use super::constants::VERTICES_PER_PATCH;
use crate::planet;
use nalgebra::{Point3, Vector3};
use planet::geometry_provider::{PatchGeometry, PatchLocation};
use planet::GeometryProvider;
use std::mem;

pub struct Generator {
    description: planet::Description,
}

impl Generator {
    pub fn new(description: planet::Description) -> Generator {
        Generator { description }
    }

    pub fn description(&self) -> &planet::Description {
        &self.description
    }
}

impl GeometryProvider for Generator {
    fn provide(&self, patch: PatchLocation) -> PatchGeometry {
        let step = patch.size/(VERTICES_PER_PATCH as f64-1.0);
        let mut positions:Vec<Point3<f64>> = Vec::with_capacity(VERTICES_PER_PATCH*VERTICES_PER_PATCH);

        for y in 0..VERTICES_PER_PATCH {
            for x in 0..VERTICES_PER_PATCH {
                let local_position = Point3::<f64>::new(x as f64 * step - 0.5,y as f64 * step - 0.5,0.5) * 2.0;
                let oriented_position = patch.face.orientation() * local_position;
                let p = morph(oriented_position) * self.description.radius;
                positions.push(p);
            }
        };

        PatchGeometry {
            positions
        }
    }
}

fn morph(pos: Point3<f64>) -> Point3<f64> {
    let pos_squared = Vector3::new(pos.x*pos.x, pos.y*pos.y, pos.z*pos.z);
    let a = Vector3::new(pos_squared.y, pos_squared.z, pos_squared.x) * 0.5;
    let b = Vector3::new(pos_squared.z, pos_squared.x, pos_squared.y) * 0.5;
    Point3::new(pos.x * f64::sqrt(1.0 - a.x - b.x + pos_squared.y*pos_squared.z/3.0),
                pos.y * f64::sqrt(1.0 - a.y - b.y + pos_squared.z*pos_squared.x/3.0),
                pos.z * f64::sqrt(1.0 - a.z - b.z + pos_squared.x*pos_squared.y/3.0))
}