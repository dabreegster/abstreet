pub use crate::backend_glow::Drawable;
use crate::backend_glow::{GfxCtxInnards, GlowInnards};
use crate::{GeomBatch, ScreenDims};
use glow::HasContext;
use std::cell::Cell;
use std::rc::Rc;
use stdweb::traits::INode;
use webgl_stdweb::WebGL2RenderingContext;
use winit::platform::web::WindowExtStdweb;

pub fn setup(window_title: &str) -> (PrerenderInnards, winit::event_loop::EventLoop<()>) {
    stdweb::console!(log, "Setting up ezgui");

    // This doesn't seem to work for the shader panics here, but later it does work. Huh.
    std::panic::set_hook(Box::new(|info| {
        stdweb::console!(log, "panicked: %s", format!("{}", info));
    }));

    let event_loop = winit::event_loop::EventLoop::new();
    let size = {
        // TODO Not sure how to get scrollbar dims
        let scrollbars = 30;
        let win = stdweb::web::window();
        // `inner_width` corresponds to the browser's `self.innerWidth` function, which are in
        // Logical, not Physical, pixels
        winit::dpi::LogicalSize::new(
            win.inner_width() - scrollbars,
            win.inner_height() - scrollbars,
        )
    };
    let window = winit::window::WindowBuilder::new()
        .with_title(window_title)
        .with_inner_size(size)
        .build(&event_loop)
        .unwrap();
    let canvas = window.canvas();
    let document = stdweb::web::document();
    let body: stdweb::web::Node = document.body().expect("Get HTML body").into();
    body.append_child(&canvas);

    let webgl2_context: WebGL2RenderingContext = canvas.get_context().unwrap();
    let gl = glow::Context::from_webgl2_context(webgl2_context);

    let program = unsafe { gl.create_program().expect("Cannot create program") };

    unsafe {
        let shaders = [
            (glow::VERTEX_SHADER, include_str!("shaders/vertex_300.glsl")),
            (
                glow::FRAGMENT_SHADER,
                include_str!("shaders/fragment_300.glsl"),
            ),
        ]
        .iter()
        .map(|(shader_type, source)| {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, source);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                stdweb::console!(log, "Shader error: %s", gl.get_shader_info_log(shader));
                panic!(gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shader
        })
        .collect::<Vec<_>>();
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            stdweb::console!(log, "Linking error: %s", gl.get_program_info_log(program));
            panic!(gl.get_program_info_log(program));
        }
        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }
        gl.use_program(Some(program));

        gl.enable(glow::SCISSOR_TEST);

        gl.enable(glow::DEPTH_TEST);
        gl.depth_func(glow::LEQUAL);

        gl.enable(glow::BLEND);
        gl.blend_func_separate(
            glow::SRC_ALPHA,
            glow::ONE_MINUS_SRC_ALPHA,
            glow::SRC_ALPHA,
            glow::ONE_MINUS_SRC_ALPHA,
        );
    }

    (
        PrerenderInnards {
            glow_innards: GlowInnards { gl: Rc::new(gl) },
            program,
            window,
            total_bytes_uploaded: Cell::new(0),
        },
        event_loop,
    )
}

pub(crate) struct VertexArray {
    pub(crate) id: glow::WebVertexArrayKey,
    pub(crate) was_destroyed: bool,
}

pub(crate) struct Buffer {
    pub(crate) id: glow::WebBufferKey,
    pub(crate) was_destroyed: bool,
}

pub struct PrerenderInnards {
    glow_innards: GlowInnards,
    window: winit::window::Window,
    program: <glow::Context as glow::HasContext>::Program,

    // TODO Prerender doesn't know what things are temporary and permanent. Could make the API more
    // detailed.
    pub total_bytes_uploaded: Cell<usize>,
}

impl PrerenderInnards {
    pub fn actually_upload(&self, permanent: bool, batch: GeomBatch) -> Drawable {
        self.glow_innards.actually_upload(permanent, batch)
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn set_cursor_icon(&self, icon: winit::window::CursorIcon) {
        self.window.set_cursor_icon(icon);
    }

    pub fn draw_new_frame(&self) -> GfxCtxInnards {
        GfxCtxInnards::new(&self.glow_innards.gl, &self.program)
    }

    pub fn window_resized(&self, new_size: ScreenDims, scale_factor: f64) {
        self.glow_innards.window_resized(new_size, scale_factor);
    }

    pub fn window_size(&self, scale_factor: f64) -> ScreenDims {
        self.window.inner_size().to_logical(scale_factor).into()
    }

    pub fn set_window_icon(&self, icon: winit::window::Icon) {
        self.window.set_window_icon(Some(icon));
    }

    pub fn monitor_scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }

    pub fn draw_did_finish(&self, _gfc_ctx_innards: GfxCtxInnards) {}
}
