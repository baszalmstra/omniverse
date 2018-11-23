use crate::transform::Transform;
use crate::transform::Transformable;

pub struct Planet {
    transform: Transform,

    radius: f64,
}

impl Transformable for Planet {
    fn transform(&self) -> &Transform {
        &self.transform
    }
    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}