
pub type Textures = imgui::Textures<glium::Texture2d>;

pub struct UI<F> {
    imgui: imgui::ImGui,
    renderer: imgui_glium_renderer::Renderer,
    run_ui: F,
}


impl<F: FnMut(&imgui::Ui, &mut Textures)> UI<F> {
    /// Create the imgui renderer and display
    pub fn new(font_size: f64, display: &glium::Display, run_ui: F) -> UI<F> {
        let mut imgui = imgui::ImGui::init();
        imgui.set_ini_filename(None);

        let dpi_factor = display.gl_window().get_hidpi_factor();

        let font_size = (font_size * dpi_factor) as f32;

        imgui.fonts().add_default_font_with_config(
            imgui::ImFontConfig::new()
                .oversample_h(1)
                .pixel_snap_h(true)
                .size_pixels(font_size),
        );

        imgui.set_font_global_scale((1.0 / dpi_factor) as f32);
        let renderer = imgui_glium_renderer::Renderer::init(&mut imgui, display).expect("Could not create imgui renderer");

        imgui_glutin_support::configure_keys(&mut imgui);
        UI {
            imgui,
            renderer,
            run_ui,
        }
    }

    /// Draw the UI
    pub fn draw(&mut self, frame: &mut glium::Frame, window: &glium::glutin::Window, previous_frame_time: f32) {
        imgui_glutin_support::update_mouse_cursor(&self.imgui, &window);

        let imgui_frame_size = get_frame_size(&window);
        let ui = self.imgui.frame(imgui_frame_size.unwrap(), previous_frame_time);

        (self.run_ui)(&ui, self.renderer.textures());
        self.renderer.render(frame, ui).expect("Could not draw UI");
    }

    /// Handle window events
    pub fn handle_event(&mut self, event: &glium::glutin::Event) {
        imgui_glutin_support::handle_event(&mut self.imgui, event);
    }
}

pub fn hello_world(ui: &imgui::Ui, _textures: &mut Textures) {
    // Draw the actual UI
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
}

/// Get the logical size + dpi factor for the window
pub fn get_frame_size(window: &glium::glutin::Window) -> Option<imgui::FrameSize> {
    window.get_inner_size().map(|logical_size| imgui::FrameSize {
        logical_size: logical_size.into(),
        hidpi_factor: window.get_hidpi_factor(),
    })
}
