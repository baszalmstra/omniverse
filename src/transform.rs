use std::cell::RefMut;
use std::cell::Ref;

pub type Transform = nalgebra::Isometry3<f64>;

pub trait Transformable {
    fn transform(&self) -> Ref<Transform>;
    fn transform_mut(&self) -> RefMut<Transform>;
}

pub trait Transformer {
    fn translate_by(&self, translation: &nalgebra::Vector3<f64>) -> &Transformer;
}

impl<T> Transformer for T
    where
        T: Transformable
{
    fn translate_by(&self, translation: &nalgebra::Vector3<f64>) -> &Transformer {
        self.transform_mut().append_translation_mut(&nalgebra::Translation3::from(*translation));
        self
    }
}