use alga::general::SubsetOf;
use crate::culling;
use crate::culling::{Classify, Containment};
use nalgebra::{Point3, Real, Scalar, Vector3};
use ncollide::bounding_volume::AABB3;

/// Implementation based on: https://cesium.com/blog/2013/04/25/horizon-culling/

#[derive(Clone)]
pub struct Cone<T: Real> {
    origin: Point3<T>,
    direction: Vector3<T>,
    near_distance: T,
    cos_angle: T,
}

impl<T: Real> Cone<T> {
    /// Construct a cone to perform horizon culling
    pub fn new(camera_position: Point3<T>, planet_radius: T) -> Cone<T> {
        let distance_to_center = camera_position.coords.norm();
        let radius_squared = planet_radius * planet_radius;
        let distance_from_center_to_plane = radius_squared / distance_to_center;
        let near_distance = distance_to_center - distance_from_center_to_plane;
        let cos_angle =
            near_distance / (distance_to_center * distance_to_center - radius_squared).sqrt();
        Cone {
            direction: -camera_position.coords.normalize(),
            origin: camera_position,
            near_distance,
            cos_angle,
        }
    }
}

impl<N: Scalar, T: Real> Classify<Point3<N>> for Cone<T>
where
    N: SubsetOf<T>,
{
    fn classify(&self, shape: &Point3<N>) -> Containment {
        let position_camera = nalgebra::convert::<Point3<N>, Point3<T>>(*shape) - self.origin;
        let point_cos_angle = nalgebra::dot(&self.direction, &position_camera);
        if point_cos_angle > self.near_distance
            && point_cos_angle / position_camera.norm() > self.cos_angle
        {
            Containment::Inside
        } else {
            Containment::Outside
        }
    }
}

impl<N: Scalar + Real, T: Real + Scalar> Classify<AABB3<N>> for Cone<T>
where
    N: SubsetOf<T>,
{
    fn classify(&self, shape: &AABB3<N>) -> Containment {
        let corners = culling::corners(shape);

        let mut corners_inside = 0;
        for (i, corner) in corners.iter().enumerate() {
            if self.contains(corner) {
                corners_inside += 1;

                if corners_inside != i {
                    return Containment::Intersects;
                }
            }
        }

        if corners_inside == 8 {
            Containment::Inside
        } else {
            Containment::Outside
        }
    }

    fn contains(&self, shape: &AABB3<N>) -> bool {
        let corners = culling::corners(shape);
        for corner in corners.iter() {
            if !self.contains(corner) {
                return false;
            }
        }

        true
    }
}
