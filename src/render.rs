use crate::command::DisplayCommand;
use crate::gfx::traits::FactoryExt;
use crate::gfx::Device;
use crate::layout;
use gfx;
use gfx_text;
use gfx_window_glutin;
use glutin;
pub type DepthFormat = gfx::format::DepthStencil;
pub type ColorFormat = gfx::format::Rgba8;

const SCREEN_WIDTH: usize = 1024;
const SCREEN_HEIGHT: usize = 768;
const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

#[derive(Copy, Clone)]
struct RenderText<'a> {
    text: &'a str,
    position: [i32; 2],
    color: [f32; 4],
}

fn render_texts(command_list: &[DisplayCommand]) -> Vec<RenderText> {
    Vec::new()
}

fn render_commands(command_list: &[DisplayCommand]) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut index_data = Vec::new();
    let mut rect_num: u16 = 0;

    for command in command_list {
        match *command {
            DisplayCommand::SolidRectangle(ref color, ref rect) => {
                let c = [color.r, color.g, color.b];

                let mut v = render_rectangle(&c, rect);
                vertices.append(&mut v);

                let index_base: u16 = rect_num * 4;
                index_data.append(&mut vec![
                    index_base,
                    index_base + 1,
                    index_base + 2,
                    index_base + 2,
                    index_base + 3,
                    index_base,
                ]);
                rect_num += 1;
            }
        }
    }
    return (vertices, index_data);
}

fn render_rectangle(c: &[f32; 3], rect: &layout::Rectangle) -> Vec<Vertex> {
    let (x, y, h, w) = transform_rectangle(rect);
    let vertices = vec![
        Vertex {
            pos: [x + w, y],
            color: *c,
        },
        Vertex {
            pos: [x, y],
            color: *c,
        },
        Vertex {
            pos: [x, y + h],
            color: *c,
        },
        Vertex {
            pos: [x + w, y + h],
            color: *c,
        },
    ];

    vertices
}

fn transform_rectangle(rect: &layout::Rectangle) -> (f32, f32, f32, f32) {
    let w = rect.width / SCREEN_WIDTH as f32 * 2.0;
    let h = rect.height / SCREEN_HEIGHT as f32 * 2.0;
    let x = rect.x / SCREEN_WIDTH as f32 * 2.0 - 1.0;
    let y = -(rect.y / SCREEN_HEIGHT as f32 * 2.0 - 1.0 + h);

    (x, y, h, w)
}

pub fn render_loop(command_list: &[DisplayCommand]) {
    let builder = glutin::WindowBuilder::new()
        .with_title(String::from("Browser"))
        .with_dimensions(glutin::dpi::LogicalSize::new(1024.0, 768.0));

    let events_loop = glutin::EventsLoop::new();
    let context = glutin::ContextBuilder::new();
    let (window, mut device, mut factory, main_color, _main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, context, &events_loop)
            .expect("failed");

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let pso = factory
        .create_pipeline_simple(
            include_bytes!("../shaders/solid.glslv"),
            include_bytes!("../shaders/solid.glslf"),
            pipe::new(),
        )
        .unwrap();

    let (vertices, index_data) = render_commands(command_list);
    let texts = render_texts(command_list);

    let (vertex_buffer, slice) =
        factory.create_vertex_buffer_with_slice(&vertices, &index_data[..]);

    let data = pipe::Data {
        vbuf: vertex_buffer,
        out: main_color,
    };

    let mut test_renderer = gfx_text::new(factory).build().unwrap();

    'main: loop {
        // for event in window.poll_events() {
        //     match event {
        //         glutin::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape))
        //         | glutin::WindowEvent::CloseRequested => break 'main,
        //         _ => {}
        //     }
        // }

        for text in &texts {
            test_renderer.add(text.text, text.position, text.color);
        }

        encoder.clear(&data.out, CLEAR_COLOR);

        encoder.draw(&slice, &pso, &data);
        test_renderer.draw(&mut encoder, &data.out);

        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
