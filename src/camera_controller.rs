use crate::transform::Rotation;
use crate::transform::Transform;
use crate::transform::Transformable;
use crate::planet;
use glium::glutin;
use glium::glutin::KeyboardInput;
use nalgebra::Vector2;
use nalgebra::Vector3;


pub struct CameraController {
    movement_vector: Vector3<f64>,
    movement_speed: f32,
    //up_vector: nalgebra::Unit<Vector3<f64>>,
    delta_mouse_position: Vector2<f64>,
}

impl CameraController {
    pub fn new() -> CameraController {
        CameraController {
            movement_vector: Vector3::new(0.0, 0.0, 0.0),
            //up_vector: Vector3::y_axis(),
            delta_mouse_position: Vector2::new(0.0, 0.0),
            movement_speed: 50.0,
        }
    }

    pub fn key_event(&mut self, input: &KeyboardInput) {
        use glium::glutin::ElementState::{Pressed, Released};
        use glium::glutin::VirtualKeyCode::*;

        match *input {
            KeyboardInput {
                state: Pressed,
                virtual_keycode: Some(key),
                ..
            } => match key {
                W => self.movement_vector.z = -1.0,
                S => self.movement_vector.z = 1.0,
                A => self.movement_vector.x = -1.0,
                D => self.movement_vector.x = 1.0,
                _ => {}
            },
            KeyboardInput {
                state: Released,
                virtual_keycode: Some(key),
                ..
            } => match key {
                W => {
                    self.movement_vector.z = if self.movement_vector.z < 0.0 {
                        0.0
                    } else {
                        self.movement_vector.z
                    }
                }
                S => {
                    self.movement_vector.z = if self.movement_vector.z > 0.0 {
                        0.0
                    } else {
                        self.movement_vector.z
                    }
                }
                A => {
                    self.movement_vector.x = if self.movement_vector.x < 0.0 {
                        0.0
                    } else {
                        self.movement_vector.x
                    }
                }
                D => {
                    self.movement_vector.x = if self.movement_vector.x > 0.0 {
                        0.0
                    } else {
                        self.movement_vector.x
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn mouse_moved(&mut self, delta_position: &(f64, f64)) {
        self.delta_mouse_position += Vector2::new(delta_position.0, delta_position.1);
    }

    pub fn mouse_wheel_event(&mut self, delta: glutin::MouseScrollDelta) {
        match delta {
            glutin::MouseScrollDelta::LineDelta(_, y) => {
                const SPEED: f32 = 0.3;
                if y >= 0.0 {
                    self.movement_speed = (self.movement_speed * (1.0 + y * SPEED)).max(1.0);
                } else {
                    self.movement_speed = (self.movement_speed / (1.0 - y * SPEED)).max(1.0);
                }
            }
            _ => {}
        }
    }

    pub fn tick<T: Transformable>(
        &mut self,
        time_since_last_frame: f32,
        transform: &mut T,
        planet_desc: &planet::Description,
        planet_transform: &Transform
    ) {
        // Determine movement speed based on distance to planet
        let planet_distance = nalgebra::norm(&(transform.translation() - planet_transform.translation())) - planet_desc.radius;
        self.movement_speed = self.movement_speed.min(50.0 + planet_distance as f32 * 10.0);

        // Determine up vector w.r.t. the planet
        let up_vec = nalgebra::normalize(&(transform.translation() - planet_transform.translation()));

        let translation = if self.movement_vector.dot(&self.movement_vector) > 0.0 {
            self.movement_vector * self.movement_speed as f64 * f64::from(time_since_last_frame)
        } else {
            self.movement_vector
        };

        transform.rotate_by(&Rotation::from_axis_angle(
            &Vector3::y_axis(),
            self.delta_mouse_position.x * -0.003,
        ));
        transform.rotate_by(&Rotation::from_axis_angle(
            &Vector3::x_axis(),
            self.delta_mouse_position.y * -0.003,
        ));
        self.delta_mouse_position = Vector2::new(0.0, 0.0);
        let local_translation = transform.transform() * translation;
        transform.translate_by(&local_translation);

        // If we're close to the planet surface (i.e., within what we'll here call the 'correction_height'),
        // straighten the camera by rotating the current camera up vector to the planet up vector
        let correction_height = 0.2 * planet_desc.radius;

        if planet_distance < correction_height {
            // Let the correction speed depend on how far we're in the atmosphere. The closer we
            // are to the planet surface, the faster the rotation will happen
            let corr_speed = (1.0 - (planet_distance / correction_height)) * 10.0;
            let local_up_vec = transform.transform().inverse() * up_vec;
            transform.rotate_by(&Rotation::from_axis_angle(
                &Vector3::z_axis(),
                time_since_last_frame as f64 * -local_up_vec.x * corr_speed)
            );
        }
    }
}
