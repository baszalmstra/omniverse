use nalgebra::{Point3, Scalar};
use ncollide::bounding_volume::AABB3;

#[derive(Copy, Clone, PartialEq)]
pub enum Containment {
    Outside,
    Inside,
    Intersects,
}

pub trait Classify<T> {
    fn classify(&self, shape: &T) -> Containment;
    fn intersects(&self, shape: &T) -> bool {
        self.classify(shape) != Containment::Outside
    }
    fn contains(&self, shape: &T) -> bool {
        self.classify(shape) == Containment::Inside
    }
}

pub fn corners<T: Scalar>(aabb: &AABB3<T>) -> [Point3<T>; 8]
where
    Point3<T>: ncollide::ncollide_math::Point,
{
    // Compute the corners of the bounding box
    let min = aabb.mins();
    let max = aabb.maxs();
    [
        Point3::new(min.x, min.y, min.z),
        Point3::new(max.x, min.y, min.z),
        Point3::new(min.x, max.y, min.z),
        Point3::new(max.x, max.y, min.z),
        Point3::new(min.x, min.y, max.z),
        Point3::new(max.x, min.y, max.z),
        Point3::new(min.x, max.y, max.z),
        Point3::new(max.x, max.y, max.z),
    ]
}
