pub type Transform = nalgebra::Isometry3<f64>;

pub trait Transformable {
    fn transform(&self) -> &Transform;
    fn transform_mut(&mut self) -> &mut Transform;

    fn translate_by(&mut self, translation: &nalgebra::Vector3<f64>) -> &mut Self {
        self.transform_mut().append_translation_mut(&nalgebra::Translation3::from(*translation));
        self
    }
}

impl Transformable for Transform {
    fn transform(&self) -> &Transform { self }
    fn transform_mut(&mut self) -> &mut Transform { self }
}