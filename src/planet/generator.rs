use super::constants::VERTICES_PER_PATCH;
use crate::planet;
use nalgebra::Point3;
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
                let local_position = Point3::<f64>::new(x as f64 * step - 0.5,y as f64 * step - 0.5,0.5) * 2.0 * self.description.radius;
                let oriented_position = patch.face.orientation() * local_position;
                positions.push(oriented_position);
            }
        };

        PatchGeometry {
            positions
        }
    }
}
