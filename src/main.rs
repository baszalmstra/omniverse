#![feature(test)]

#[macro_use]
extern crate glium;
#[macro_use]
extern crate lazy_static;
extern crate alga;
extern crate nalgebra;
extern crate ncollide;

extern crate core;
extern crate test;

use crate::camera::Camera;
use crate::camera_controller::CameraController;
use crate::transform::{Transform, Transformable};
use nalgebra::Vector3;

mod camera;
mod camera_controller;
mod frustum;
mod planet;
mod timeline;
mod transform;
mod id_arena;

fn main() {
    use glium::glutin;
    use glium::Surface;

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new().with_title("Omniverse");
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let mut camera = Camera::new();
    camera.translate_by(&Vector3::new(0.0, 0.0, 1500.0));

    let planet_desc = planet::Description { radius: 1000.0 };
    let planet_transform = Transform::identity();
    let mut planet_renderer =
        planet::Renderer::new(&display, planet_desc, planet::Generator::new(planet_desc))
            .expect("Could not instantiate renderer");

    let mut camera_controller = CameraController::new();

    let mut closed = false;
    let mut timeline = timeline::Timeline::new();
    let _rotation: f32 = 0.0;

    let mut left_mouse_pressed = false;
    let mut last_logical_mouse_position = glutin::dpi::LogicalPosition::new(0.0, 0.0);
    let mut mouse_down_mouse_position = last_logical_mouse_position;

    while !closed {
        timeline.next_frame();

        camera_controller.tick(timeline.previous_frame_time(), &mut camera);

        let mut frame = display.draw();
        let frame_size = frame.get_dimensions();
        let aspect_ratio = frame_size.0 as f32 / frame_size.1 as f32;
        let frustum = camera.frustum(aspect_ratio);

        frame.clear_color_and_depth((0.01, 0.01, 0.01, 1.0), 1.0);

        planet_renderer.ensure_resident_patches(&frustum, &planet_transform);
        planet_renderer.draw(
            &mut frame,
            &frustum,
            &planet_transform,
            &planet::DrawParameters {
                wireframe: true,
                ..Default::default()
            },
        );

        frame.finish().unwrap();

        events_loop.poll_events(|ev| match ev {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => closed = true,
                glutin::WindowEvent::KeyboardInput { input, .. } => {
                    camera_controller.key_event(&input);
                }
                glutin::WindowEvent::MouseInput {
                    state: glutin::ElementState::Pressed,
                    button: glutin::MouseButton::Left,
                    ..
                } => {
                    left_mouse_pressed = true;
                    mouse_down_mouse_position = last_logical_mouse_position;
                    display.gl_window().hide_cursor(true);
                }
                glutin::WindowEvent::MouseInput {
                    state: glutin::ElementState::Released,
                    button: glutin::MouseButton::Left,
                    ..
                } => {
                    left_mouse_pressed = false;
                    display
                        .gl_window()
                        .set_cursor_position(mouse_down_mouse_position)
                        .unwrap();
                    display.gl_window().hide_cursor(false);
                }
                glutin::WindowEvent::CursorMoved { position, .. } => {
                    last_logical_mouse_position = position;
                }
                _ => (),
            },
            glutin::Event::DeviceEvent { event, .. } => match event {
                glutin::DeviceEvent::MouseMotion { delta, .. } => {
                    if left_mouse_pressed {
                        camera_controller.mouse_moved(&delta);
                    }
                }
                _ => (),
            },
            _ => (),
        })
    }
}
