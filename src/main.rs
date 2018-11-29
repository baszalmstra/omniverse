#[macro_use]
extern crate log;
extern crate glium;
extern crate pretty_env_logger;

#[macro_use]
extern crate imgui;

extern crate nalgebra;
extern crate omniverse;

use glium::CapabilitiesSource;
use nalgebra::Vector3;
use omniverse::camera::Camera;
use omniverse::camera_controller::CameraController;
use omniverse::planet;
use omniverse::timeline;
use omniverse::transform::{Transform, Transformable};

fn create_imgui(display: &glium::Display) -> (imgui_glium_renderer::Renderer, imgui::ImGui) {
    let mut imgui = imgui::ImGui::init();
    imgui.set_ini_filename(None);

    let dpi_factor = display.gl_window().get_hidpi_factor();

    let font_size = (12.0 * dpi_factor) as f32;

    imgui.fonts().add_default_font_with_config(
        imgui::ImFontConfig::new()
            .oversample_h(1)
            .pixel_snap_h(true)
            .size_pixels(font_size),
    );

    imgui.set_font_global_scale((1.0 / dpi_factor) as f32);

    (
        imgui_glium_renderer::Renderer::init(&mut imgui, display)
            .expect("Could not create imgui renderer"),
        imgui,
    )
}

fn main() {
    use glium::glutin;
    use glium::Surface;

    pretty_env_logger::init();

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new().with_title("Omniverse");
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(window, context, &events_loop).unwrap();
//    let hidpi_factor = display.gl_window().get_hidpi_factor();

    if !display.get_extensions().gl_arb_multi_draw_indirect {
        error!("Missing required OpenGL extension: GL_ARB_multi_draw_indirect");
        return;
    }

    // Imgui initialization
    let (mut imgui_renderer, mut imgui) = create_imgui(&display);
    imgui_glutin_support::configure_keys(&mut imgui);

    let mut camera = Camera::new();
    camera.translate_by(&Vector3::new(0.0, 0.0, 2000.0));

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
            &planet::DrawParameters { wire_frame: true },
        );


        imgui_glutin_support::update_mouse_cursor(&imgui, &display.gl_window());

        let imgui_frame_size = {
            let window = display.gl_window();
            window.get_inner_size().map(|logical_size| imgui::FrameSize {
                logical_size: logical_size.into(),
                hidpi_factor: window.get_hidpi_factor(),
            })
        };

        let ui = imgui.frame(imgui_frame_size.unwrap(), timeline.previous_frame_time());

        ui.window(im_str!("Hello world"))
            .size((300.0, 100.0), imgui::ImGuiCond::FirstUseEver)
            .build(|| {
                ui.text(im_str!("Hello world!"));
                ui.separator();
                let mouse_pos = ui.imgui().mouse_pos();
                ui.text(im_str!(
                "Mouse Position: ({:.1},{:.1})",
                mouse_pos.0,
                mouse_pos.1
            ));
            });

        imgui_renderer.render(&mut frame, ui).expect("Could not draw UI");

        frame.finish().unwrap();

        events_loop.poll_events(|ev| {
            imgui_glutin_support::handle_event(&mut imgui, &ev);

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
