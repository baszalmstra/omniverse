#[macro_use]
extern crate log;
extern crate glium;
extern crate pretty_env_logger;


extern crate nalgebra;
extern crate omniverse;

use std::fs;

use glium::CapabilitiesSource;
use nalgebra::Vector3;
use omniverse::camera::Camera;
use omniverse::camera_controller::CameraController;
use omniverse::planet;
use omniverse::timeline;
use omniverse::ui;
use omniverse::transform::{Transform, Transformable};

fn main() {
    use glium::glutin;
    use glium::Surface;

    pretty_env_logger::init();

    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new().with_title("Omniverse");
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(window_builder, context, &events_loop).unwrap();
    let window = display.gl_window();
//    let hidpi_factor = display.gl_window().get_hidpi_factor();

    if !display.get_extensions().gl_arb_multi_draw_indirect {
        error!("Missing required OpenGL extension: GL_ARB_multi_draw_indirect");
        return;
    }

    // Imgui initialization
    let mut ui = ui::UI::new(12.0, &display, ui::hello_world);

    let mut camera = Camera::new();
    camera.translate_by(&Vector3::new(0.0, 0.0, 402_000.0));
    camera.set_far(200_00000.0);
    camera.pitch(std::f64::consts::PI*0.5);

    let terrain_str = fs::read_to_string("resources/terrain.json").expect("Missing resource file: resources/terrain.json");
    let terrain_desc = serde_json::from_str(&terrain_str).expect("Corrupt JSON in file: resources/terrain.json");

    let planet_desc = planet::Description { radius: 400_000.0, terrain: terrain_desc };
    let planet_transform = Transform::identity();

    let geometry_provider = planet::Generator::new(planet_desc.clone());
    let async_geometry_provider = planet::ThreadpoolGeometryProvider::new(geometry_provider);
    let mut planet_renderer =
        planet::Renderer::new(&display, planet_desc.clone(), async_geometry_provider)
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
            &planet::DrawParameters { wire_frame: false },
        );

        ui.draw(&mut frame, &window, timeline.previous_frame_time());

        frame.finish().unwrap();

        events_loop.poll_events(|ev| {
            ui.handle_event(&ev);

            match ev {
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
                    glutin::WindowEvent::MouseWheel {delta, ..} => {
                        camera_controller.mouse_wheel_event(delta);
                    },
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
                glutin::Event::DeviceEvent { event, .. } => {
                    if let glutin::DeviceEvent::MouseMotion { delta, .. } = event {
                        if left_mouse_pressed {
                            camera_controller.mouse_moved(&delta);
                        }
                    }
                }
                _ => (),
            }
        })
    }
}
