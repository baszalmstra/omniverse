use alga::general::SubsetOf;
use crate::transform::Transform;
use crate::culling;
use crate::culling::Classify;
use crate::culling::Containment;
use nalgebra::{convert, Matrix4, Point3, Scalar, Vector3, Vector4};
use ncollide::bounding_volume::AABB3;

pub struct Frustum {
    pub transform: Transform,

    pub projection: Matrix4<f32>,
    pub view_projection: Matrix4<f32>,

    planes: [Vector4<f32>; 6],
}

impl Frustum {
    pub fn new(transform: Transform, projection: Matrix4<f32>) -> Frustum {
        let view: Matrix4<f32> = convert(transform.inverse().to_homogeneous());
        let view_projection = projection * view;
        Frustum {
            transform,
            projection,
            view_projection,
            planes: [
                view_projection.row(3).transpose() + view_projection.row(0).transpose(), // Left
                view_projection.row(3).transpose() - view_projection.row(0).transpose(), // Right
                view_projection.row(3).transpose() + view_projection.row(1).transpose(), // Bottom
                view_projection.row(3).transpose() - view_projection.row(1).transpose(), // Top
                view_projection.row(3).transpose() + view_projection.row(2).transpose(), // Near
                view_projection.row(3).transpose() - view_projection.row(2).transpose(), // Far
            ],
        }
    }

    pub fn far_distance(&self) -> f32 {
        let far_plane = self.planes[5];
        let normalized_plane = far_plane / Vector3::new(far_plane.x, far_plane.y, far_plane.z).norm();
        let pos = self.transform.translation.vector;
        (normalized_plane.x as f64*pos.x + normalized_plane.y as f64*pos.y + normalized_plane.z as f64*pos.z + normalized_plane.w as f64) as f32
    }
}

impl<T: Scalar> Classify<Point3<T>> for Frustum
where
    T: SubsetOf<f32>,
{
    fn classify(&self, shape: &Point3<T>) -> Containment {
        for plane in self.planes.iter() {
            match half_space(plane, &shape.coords) {
                HalfSpace::Negative => return Containment::Outside,
                HalfSpace::On => return Containment::Intersects,
                _ => {}
            }
        }

        Containment::Inside
    }
}

impl<T: Scalar> Classify<AABB3<T>> for Frustum
where
    T: SubsetOf<f32>,
    Point3<T>: ncollide::ncollide_math::Point,
{
    fn classify(&self, shape: &AABB3<T>) -> Containment {
        let corners = culling::corners(shape);

        // Test all corners against all planes. If all points are behind 1 specific plane, the AABB
        // is outside the frustum. If all points are inside the frustum the AABB is inside.
        let mut total_corners_inside = 0;
        for plane in self.planes.iter() {
            let mut total_positive = 8;
            let mut point_inside = 1;

            for corner in corners.iter() {
                if half_space(plane, &corner.coords) == HalfSpace::Negative {
                    point_inside = 0;
                    total_positive -= 1;
                }
            }

            if total_positive == 0 {
                return Containment::Outside;
            }

            total_corners_inside += point_inside;
        }

        if total_corners_inside == 6 {
            Containment::Inside
        } else {
            Containment::Intersects
        }
    }
}

#[derive(PartialEq)]
enum HalfSpace {
    Negative,
    On,
    Positive,
}

fn half_space<T: Scalar + SubsetOf<f32>>(plane: &Vector4<f32>, p: &Vector3<T>) -> HalfSpace {
    let p = convert::<Vector3<T>, Vector3<f32>>(*p);
    let det = plane.x * p.x + plane.y * p.y + plane.z * p.z + plane.w;
    if det < 0.0 {
        HalfSpace::Negative
    } else if det > 0.0 {
        HalfSpace::Positive
    } else {
        HalfSpace::On
    }
}
