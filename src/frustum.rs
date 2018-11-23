use crate::transform::Transform;
use nalgebra as na;

pub struct Frustum {
    pub transform: Transform,

    pub projection: na::Matrix4<f32>,
    pub view_projection: na::Matrix4<f32>,
}

impl Frustum {
    pub fn new(transform: Transform, projection: na::Matrix4<f32>) -> Frustum {
        Frustum {
            transform,
            projection,
            view_projection: projection * na::convert::<na::Matrix4<f64>, na::Matrix4<f32>>(
                transform.to_homogeneous(),
            ),
        }
    }
}
