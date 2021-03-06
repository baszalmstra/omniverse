use alga::linear::AffineTransformation;

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
        {
            let t: &mut Transform = self.transform_mut();
            *t = t.prepend_rotation(rotation);
        }
        self
    }

    fn pitch(&mut self, angle:f64) -> &mut Self {
        self.rotate_by(&nalgebra::UnitQuaternion::from_euler_angles(angle, 0.0, 0.0))
    }

    fn translation(&self) -> nalgebra::Vector3<f64> {
        self.transform().translation.vector
    }
}

impl Transformable for Transform {
    fn transform(&self) -> &Transform {
        self
    }
    fn transform_mut(&mut self) -> &mut Transform {
        self
    }
}
