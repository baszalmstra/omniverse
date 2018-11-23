pub type Transform = nalgebra::Isometry3<f64>;
pub type Rotation = nalgebra::UnitQuaternion<f64>;

pub trait Transformable {
    fn transform(&self) -> &Transform;
    fn transform_mut(&mut self) -> &mut Transform;

    fn translate_by(&mut self, translation: &nalgebra::Vector3<f64>) -> &mut Self {
        self.transform_mut()
            .append_translation_mut(&nalgebra::Translation3::from_vector(*translation));
        self
    }

    fn rotate_by(&mut self, rotation: &Rotation) -> &mut Self {
        self.transform_mut().append_rotation_mut(rotation);
        self
    }
}
