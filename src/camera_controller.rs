use nalgebra::Vector3;
use transform::{Transformer};
use glium::glutin::KeyboardInput;

pub struct CameraController<'a> {
    pub transformable: &'a Transformer,

    movement_vector: Vector3<f64>,
}

impl<'a> CameraController<'a> {
    pub fn new(transformable: &'a Transformer) -> CameraController<'a> {
        CameraController {
            transformable,
            movement_vector: Vector3::new(0.0, 0.0, 0.0)
        }
    }

    pub fn key_event(&mut self, input: &KeyboardInput) {
        use glium::glutin::ElementState::{Released, Pressed};
        use glium::glutin::VirtualKeyCode::*;

        match *input {
            KeyboardInput { state: Pressed, virtual_keycode: Some(key), .. } => {
                match key {
                    W => self.movement_vector.z = 1.0,
                    S => self.movement_vector.z = -1.0,
                    A => self.movement_vector.x = 1.0,
                    D => self.movement_vector.x = -1.0,
                    _ => {}
                }
            },
            KeyboardInput { state: Released, virtual_keycode: Some(key), .. } => {
                match key {
                    W => self.movement_vector.z = if self.movement_vector.z > 0.0 { 0.0 } else { self.movement_vector.z },
                    S => self.movement_vector.z = if self.movement_vector.z < 0.0 { 0.0 } else { self.movement_vector.z },
                    A => self.movement_vector.x = if self.movement_vector.x > 0.0 { 0.0 } else { self.movement_vector.x },
                    D => self.movement_vector.x = if self.movement_vector.x < 0.0 { 0.0 } else { self.movement_vector.x },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    pub fn tick(&mut self, time_since_last_frame:f32) {
        let translation = self.movement_vector * time_since_last_frame as f64;
        self.transformable.translate_by(&translation);
    }
}