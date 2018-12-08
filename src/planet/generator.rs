use super::constants::{NORMALS_PER_PATCH, VERTICES_PER_PATCH};
use crate::planet;
use crate::planet::geometry_provider::{PatchGeometry, PatchLocation};
use crate::planet::GeometryProvider;
use nalgebra::{Point3, Vector2, Vector3};

pub struct Generator {
    description: planet::Description,
}

impl Generator {
    pub fn new(description: planet::Description) -> Generator {
        Generator { description }
    }

    fn compute_vertex(&self, x: f64, y: f64, patch: &PatchLocation) -> Point3<f64> {
        let oriented_position = patch.face.orientation() * Vector3::new(x, y, 1.0);
        let dir = morph(oriented_position);

        let height = 20.0 * (f64::sin(30.0 * (dir.x + dir.y + dir.z)) + 1.0);

        Point3::from_coordinates(dir * (self.description.radius + height))
    }

    fn compute_normal(&self, x: f64, y: f64, patch: &PatchLocation) -> Vector3<f64> {
        let eps = 0.001;

        let px1 = self.compute_vertex(x - eps, y, patch);
        let px2 = self.compute_vertex(x + eps, y, patch);
        let py1 = self.compute_vertex(x, y - eps, patch);
        let py2 = self.compute_vertex(x, y + eps, patch);

        let x_diff = px2 - px1;
        let y_diff = py2 - py1;

        x_diff.cross(&y_diff).normalize()
    }
}

impl GeometryProvider for Generator {
    fn provide(&self, patch: PatchLocation) -> PatchGeometry {
        // Generate vertices
        let vertex_step = patch.size / (VERTICES_PER_PATCH as f64 - 1.0);
        let mut positions: Vec<Point3<f64>> =
            Vec::with_capacity(VERTICES_PER_PATCH * VERTICES_PER_PATCH);
        for y in 0..VERTICES_PER_PATCH {
            for x in 0..VERTICES_PER_PATCH {
                let local_position = Vector2::<f64>::new(
                    x as f64 * vertex_step - 0.5 + patch.offset.x,
                    y as f64 * vertex_step - 0.5 + patch.offset.y,
                ) * 2.0;
                positions.push(self.compute_vertex(local_position.x, local_position.y, &patch));
            }
        }

        // Generate normals
        let normal_step = patch.size / ((NORMALS_PER_PATCH - 2) as f64);
        let mut normals: Vec<Vector3<f64>> =
            Vec::with_capacity(NORMALS_PER_PATCH * NORMALS_PER_PATCH);
        for y in 0..NORMALS_PER_PATCH {
            for x in 0..NORMALS_PER_PATCH {
                let local_position = Vector2::<f64>::new(
                    x as f64 * normal_step - 0.5 + patch.offset.x,
                    y as f64 * normal_step - 0.5 + patch.offset.y,
                ) * 2.0;
                normals.push(self.compute_normal(local_position.x, local_position.y, &patch));
            }
        }

        PatchGeometry { positions, normals }
    }
}

fn morph(pos: Vector3<f64>) -> Vector3<f64> {
    let pos_squared = Vector3::new(pos.x * pos.x, pos.y * pos.y, pos.z * pos.z);
    let a = Vector3::new(pos_squared.y, pos_squared.z, pos_squared.x) * 0.5;
    let b = Vector3::new(pos_squared.z, pos_squared.x, pos_squared.y) * 0.5;
    Vector3::new(
        pos.x * f64::sqrt(1.0 - a.x - b.x + pos_squared.y * pos_squared.z / 3.0),
        pos.y * f64::sqrt(1.0 - a.y - b.y + pos_squared.z * pos_squared.x / 3.0),
        pos.z * f64::sqrt(1.0 - a.z - b.z + pos_squared.x * pos_squared.y / 3.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Point2;
//    use test::Bencher;
//
//    #[bench]
//    fn provide(b: &mut Bencher) {
//        let generator = Generator::new(planet::Description { radius: 100.0 });
//        b.iter(|| {
//            generator.provide(PatchLocation {
//                face: planet::Face::Back,
//                offset: Point2::new(0.0, 0.0),
//                size: 1.0,
//            })
//        });
//    }
}
