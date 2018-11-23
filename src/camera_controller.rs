use nalgebra::Vector3;
use nalgebra::Vector2;
use crate::transform::Transformable;
use glium::glutin::KeyboardInput;
use glium::glutin::dpi::PhysicalPosition;
use transform::Rotation;

pub struct CameraController {
    movement_vector: Vector3<f64>,
    up_vector: nalgebra::Unit<Vector3<f64>>,
    delta_mouse_position: Vector2<f64>,
}

impl CameraController {
    pub fn new() -> CameraController {
        CameraController {
            movement_vector: Vector3::new(0.0, 0.0, 0.0),
            up_vector: Vector3::y_axis(),
            delta_mouse_position: Vector2::new(0.0, 0.0),
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
            }
            KeyboardInput { state: Released, virtual_keycode: Some(key), .. } => {
                match key {
                    W => self.movement_vector.z = if self.movement_vector.z > 0.0 { 0.0 } else { self.movement_vector.z },
                    S => self.movement_vector.z = if self.movement_vector.z < 0.0 { 0.0 } else { self.movement_vector.z },
                    A => self.movement_vector.x = if self.movement_vector.x > 0.0 { 0.0 } else { self.movement_vector.x },
                    D => self.movement_vector.x = if self.movement_vector.x < 0.0 { 0.0 } else { self.movement_vector.x },
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn mouse_moved(&mut self, _position: &PhysicalPosition, delta_position: &PhysicalPosition) {
        self.delta_mouse_position = Vector2::new(delta_position.x, delta_position.y);
    }

    pub fn tick<T: Transformable>(&mut self, time_since_last_frame: f32, transform: &mut T) {
        let translation = self.movement_vector * time_since_last_frame as f64;

        transform.rotate_by(&Rotation::from_axis_angle(&Vector3::y_axis(), self.delta_mouse_position.x * 0.03));
        transform.rotate_by(&Rotation::from_axis_angle(&Vector3::x_axis(), self.delta_mouse_position.y * 0.03));
        self.delta_mouse_position = Vector2::new(0.0, 0.0);
        transform.translate_by(&translation);
    }
}