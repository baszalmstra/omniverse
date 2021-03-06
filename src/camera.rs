#![allow(dead_code)]

use crate::frustum::Frustum;
use crate::transform::{Transform, Transformable};
use nalgebra;

pub struct Camera {
    transform: Transform,

    fov: f32,
    near: f32,
    far: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            transform: Transform::identity(),
            fov: 1.0,
            near: 0.1,
            far: 10000.0,
        }
    }

    pub fn set_near(&mut self, near: f32) -> &mut Self {
        self.near = near;
        self
    }

    pub fn set_far(&mut self, far: f32) -> &mut Self {
        self.far = far;
        self
    }

    pub fn set_field_of_view(&mut self, fov: f32) -> &mut Self {
        self.fov = fov;
        self
    }

    pub fn frustum(&self, aspect_ratio: f32) -> Frustum {
        Frustum::new(
            self.transform,
            nalgebra::Matrix4::new_perspective(aspect_ratio, self.fov, self.near, self.far),
            self.far
        )
    }
}

impl Transformable for Camera {
    fn transform(&self) -> &Transform {
        &self.transform
    }
    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}
