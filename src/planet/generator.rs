use super::constants::{NORMALS_PER_PATCH, VERTICES_PER_PATCH};
use crate::planet;
use crate::planet::geometry_provider::{PatchGeometry, PatchLocation};
use crate::planet::GeometryProvider;
use nalgebra::{Point3, Vector3, Point2};
use crate::planet::Face;
use crate::planet::TerrainLayer;

#[derive(Clone)]
pub struct Generator {
    description: planet::Description,
    terrain: TerrainLayer,
}

impl Generator {
    pub fn new(description: planet::Description, terrain: TerrainLayer) -> Generator {
        Generator {
            description,
            terrain
        }
    }

    #[inline]
    //fn compute_vertex(&self, x: f64, y: f64, patch: &PatchLocation) -> Point3<f64> {
    fn compute_vertex(&self, oriented_position:Vector3<f64>) -> Point3<f64> {
        let dir = morph(oriented_position);

        let dir32 = Vector3::new(dir.x as f32, dir.y as f32, dir.z as f32);
        let height : f32 = self.terrain.compute_height(&dir32);

        Point3::from_coordinates(dir * (self.description.radius + height as f64))
    }

    fn compute_vertex_and_color(&self, oriented_position:Vector3<f64>) -> (Point3<f64>, Vector3<f32>) {
        let dir = morph(oriented_position);

        let dir32 = Vector3::new(dir.x as f32, dir.y as f32, dir.z as f32);
        let (height, color) = self.terrain.compute_height_and_color(&dir32);

        let position = Point3::from_coordinates(dir * (self.description.radius + height as f64));

        (position, color)
    }

    fn compute_normal(&self, oriented_position:Vector3<f64>, tangent:&Vector3<f64>, binormal:&Vector3<f64>) -> Vector3<f64> {
        let eps = 0.000001;

        let px1 = self.compute_vertex(&oriented_position - tangent*eps);
        let px2 = self.compute_vertex(&oriented_position + tangent*eps);
        let py1 = self.compute_vertex(&oriented_position - binormal*eps);
        let py2 = self.compute_vertex(&oriented_position + binormal*eps);

        let x_diff = px2 - px1;
        let y_diff = py2 - py1;

        x_diff.cross(&y_diff).normalize()
    }
}

impl GeometryProvider for Generator {
    fn compute_geometry(&self, patch: PatchLocation) -> PatchGeometry {
        // Generate vertex positions and colors
        let vertex_step = patch.size / (VERTICES_PER_PATCH as f64 - 1.0);
        let mut positions: Vec<Point3<f64>> =
            Vec::with_capacity(VERTICES_PER_PATCH * VERTICES_PER_PATCH);
        let mut colors: Vec<Vector3<f32>> =
            Vec::with_capacity(VERTICES_PER_PATCH * VERTICES_PER_PATCH);

        let corner = patch.face.orientation() * Vector3::new(patch.offset.x - 0.5, patch.offset.y - 0.5, 0.5) * 2.0;
        let tangent = patch.face.orientation() * Vector3::new(1.0, 0.0, 0.0);
        let binormal = patch.face.orientation() * Vector3::new(0.0, 1.0, 0.0);

        for y in 0..VERTICES_PER_PATCH {
            for x in 0..VERTICES_PER_PATCH {
                let local_position = corner + tangent*(vertex_step * 2.0 * x as f64) + binormal*(vertex_step * 2.0 * y as f64);
                let (position, color) = self.compute_vertex_and_color(local_position);
                positions.push(position);
                colors.push(color);
            }
        }

        // Generate normals
        let normal_step = patch.size / ((NORMALS_PER_PATCH - 2) as f64);
        let mut normals: Vec<Vector3<f64>> =
            Vec::with_capacity(NORMALS_PER_PATCH * NORMALS_PER_PATCH);
        for y in 0..NORMALS_PER_PATCH {
            for x in 0..NORMALS_PER_PATCH {
                let local_position = corner + tangent*(normal_step * 2.0 * x as f64) + binormal*(normal_step * 2.0 * y as f64);
                normals.push(self.compute_normal(local_position, &tangent, &binormal));
            }
        }

        PatchGeometry { positions, normals, colors }
    }

    fn position_at(&self, face: Face, offset: Point2<f64>) -> Point3<f64> {
        self.compute_vertex(face.orientation() * Vector3::new(offset.x, offset.y, 0.5))
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

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use nalgebra::Point2;
////    use test::Bencher;
////
////    #[bench]
////    fn provide(b: &mut Bencher) {
////        let generator = Generator::new(planet::Description { radius: 100.0 });
////        b.iter(|| {
////            generator.provide(PatchLocation {
////                face: planet::Face::Back,
////                offset: Point2::new(0.0, 0.0),
////                size: 1.0,
////            })
////        });
////    }
//}
