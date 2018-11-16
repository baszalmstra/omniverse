use crate::transform::Transform;
use std::cell::RefCell;
use crate::transform::Transformable;
use std::cell::Ref;
use std::cell::RefMut;

pub struct Planet {
    transform: RefCell<Transform>,

    radius: f64,
}

impl Transformable for Planet {
    fn transform(&self) -> Ref<Transform> {
        self.transform.borrow()
    }
    fn transform_mut(&self) -> RefMut<Transform> {
        self.transform.borrow_mut()
    }
}