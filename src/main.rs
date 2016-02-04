#[macro_use]
extern crate glium;

use std::borrow::Cow;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use glium::{DisplayBuild, Surface};
use glium::backend::Facade;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

struct DecodeContext {
    facade: Rc<glium::backend::Context>,
    vertices: glium::vertex::VertexBuffer<Vertex>,
    program_pass1: glium::program::Program,
    program_pass2: glium::program::Program,
    program_pass3: glium::program::Program,
    program_convert: glium::program::Program,
    pass1_top: glium::texture::IntegralTexture2d,
    pass1_bot: glium::texture::IntegralTexture2d,
    pass2_top: glium::texture::IntegralTexture2d,
    pass2_bot: glium::texture::IntegralTexture2d,
}

impl DecodeContext {
    pub fn new(facade: Rc<glium::backend::Context>, width: u32, height: u32) -> DecodeContext {
        let v1 = Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] };
        let v2 = Vertex { position: [-1.0, 1.0], tex_coords: [0.0, 1.0] };
        let v3 = Vertex { position: [1.0, -1.0], tex_coords: [1.0, 0.0] };
        let v4 = Vertex { position: [1.0, 1.0], tex_coords: [1.0, 1.0] };
        let strip = vec![v1, v2, v3, v4];
        let vertices = glium::VertexBuffer::new(&facade, &strip).unwrap();

        let vertex_shader_src = include_str!("simple_vertex.glsl");
        let fragment_shader_pass1_src = include_str!("idct8x8_pass1.glsl");
        let fragment_shader_pass2_src = include_str!("idct8x8_pass2.glsl");
        let fragment_shader_pass3_src = include_str!("idct8x8_pass3.glsl");
        let fragment_shader_convert_src = include_str!("convert.glsl");
        let program_pass1 = glium::Program::from_source(&facade, vertex_shader_src,
                                                        fragment_shader_pass1_src, None).unwrap();
        let program_pass2 = glium::Program::from_source(&facade, vertex_shader_src,
                                                        fragment_shader_pass2_src, None).unwrap();
        let program_pass3 = glium::Program::from_source(&facade, vertex_shader_src,
                                                        fragment_shader_pass3_src, None).unwrap();
        let program_convert = glium::Program::from_source(&facade, vertex_shader_src,
                                                          fragment_shader_convert_src, None).unwrap();
        let pass1_top = glium::texture::IntegralTexture2d::empty_with_format(
            &facade,
            glium::texture::UncompressedIntFormat::I16I16I16I16,
            glium::texture::MipmapsOption::NoMipmap,
            width, height).unwrap();
        let pass1_bot = glium::texture::IntegralTexture2d::empty_with_format(
            &facade,
            glium::texture::UncompressedIntFormat::I16I16I16I16,
            glium::texture::MipmapsOption::NoMipmap,
            width, height).unwrap();
        let pass2_top = glium::texture::IntegralTexture2d::empty_with_format(
            &facade,
            glium::texture::UncompressedIntFormat::I16I16I16I16,
            glium::texture::MipmapsOption::NoMipmap,
            width, height).unwrap();
        let pass2_bot = glium::texture::IntegralTexture2d::empty_with_format(
            &facade,
            glium::texture::UncompressedIntFormat::I16I16I16I16,
            glium::texture::MipmapsOption::NoMipmap,
            width, height).unwrap();

        DecodeContext {
            facade: facade,
            vertices: vertices,
            program_pass1: program_pass1,
            program_pass2: program_pass2,
            program_pass3: program_pass3,
            program_convert: program_convert,
            pass1_top: pass1_top,
            pass1_bot: pass1_bot,
            pass2_top: pass2_top,
            pass2_bot: pass2_bot,
        }
    }
}

struct Plane {
    width: u32,
    height: u32,
    data: Vec<i16>,
}

fn decode_plane(ctx: &DecodeContext, plane: &Plane) -> glium::texture::IntegralTexture2d {
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);
    let data_image = glium::texture::RawImage2d {
        data: Cow::Borrowed(&plane.data),
        width: plane.width,
        height: plane.height,
        format: glium::texture::ClientFormat::I16,
    };
    let data_texture = glium::texture::IntegralTexture2d::with_format(
        &ctx.facade,
        data_image,
        glium::texture::UncompressedIntFormat::I16,
        glium::texture::MipmapsOption::NoMipmap).unwrap();
    {
        let uniforms_pass1 = uniform! {
            plane_dims: [plane.width as i32, plane.height as i32],
            data: &data_texture,
        };
        let output_pass1 = [
            ("pass1_top", &ctx.pass1_top),
            ("pass1_bot", &ctx.pass1_bot),
        ];
        let mut target1 = glium::framebuffer::MultiOutputFrameBuffer::new(
            &ctx.facade,
            output_pass1.iter().cloned()).unwrap();
        target1.draw(&ctx.vertices, &indices, &ctx.program_pass1, &uniforms_pass1,
                     &Default::default()).unwrap();
    }
    let uniforms_pass2 = uniform! {
        plane_dims: [plane.width as i32, plane.height as i32],
        pass1_top: &ctx.pass1_top,
        pass1_bot: &ctx.pass1_bot,
    };
    let output_pass2 = [
        ("pass2_top", &ctx.pass2_top),
        ("pass2_bot", &ctx.pass2_bot),
    ];
    let mut target2 = glium::framebuffer::MultiOutputFrameBuffer::new(
        &ctx.facade,
        output_pass2.iter().cloned()).unwrap();
    target2.draw(&ctx.vertices, &indices, &ctx.program_pass2, &uniforms_pass2,
                 &Default::default()).unwrap();
    let uniforms_pass3 = uniform! {
        plane_dims: [plane.width as i32, plane.height as i32],
        pass2_top: &ctx.pass2_top,
        pass2_bot: &ctx.pass2_bot,
    };
    {
        let mut target3 = glium::framebuffer::SimpleFrameBuffer::new(&ctx.facade, &data_texture).unwrap();
        target3.draw(&ctx.vertices, &indices, &ctx.program_pass3, &uniforms_pass3,
                     &Default::default()).unwrap();
    }
    data_texture
}

fn convert_planes(ctx: &DecodeContext, width: u32, height: u32,
                  textures: &Vec<glium::texture::IntegralTexture2d>)
                  -> glium::texture::Texture2d {
    let output = glium::texture::Texture2d::empty_with_format(
        &ctx.facade,
        glium::texture::UncompressedFloatFormat::U8U8U8,
        glium::texture::MipmapsOption::NoMipmap,
        width, height).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);
    let uniforms = uniform! {
        plane_dims: [width as i32, height as i32],
        y_plane: &textures[0],
        cb_plane: &textures[1],
        cr_plane: &textures[2],
    };
    {
        let mut target = glium::framebuffer::SimpleFrameBuffer::new( &ctx.facade, &output).unwrap();
        target.draw(&ctx.vertices, &indices, &ctx.program_convert, &uniforms,
                    &Default::default()).unwrap();
    }
    output
}

fn read_data(file: &str) -> Vec<i16> {
        let mut f = File::open(file).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s.split_whitespace()
            .map(|coeff_str| i16::from_str_radix(coeff_str, 10).unwrap())
            .collect()
}

fn main() {
    let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();
    loop {
        let y_plane = Plane {
            width: 1024,
            height: 576,
            data: read_data("f1.Y"),
        };
        let cb_plane = Plane {
            width: 512,
            height: 288,
            data: read_data("f1.Cb"),
        };
        let cr_plane = Plane {
            width: 512,
            height: 288,
            data: read_data("f1.Cr"),
        };

        assert!((y_plane.width * y_plane.height) as usize == y_plane.data.len());
        assert!((cb_plane.width * cb_plane.height) as usize == cb_plane.data.len());
        assert!((cr_plane.width * cr_plane.height) as usize == cr_plane.data.len());

        let ctx = DecodeContext::new(display.get_context().clone(), y_plane.width, y_plane.height);
        let planes = vec![y_plane, cb_plane, cr_plane];
        let output: Vec<_> = planes.iter().map(|p| decode_plane(&ctx, p)).collect();

        let image = convert_planes(&ctx, 1024, 576, &output);
        
        // 16:9
        let v1 = Vertex { position: [-0.75, -0.09375], tex_coords: [0.0, 1.0] };
        let v2 = Vertex { position: [-0.75, 0.75], tex_coords: [0.0, 0.0] };
        let v3 = Vertex { position: [0.75, -0.09375], tex_coords: [1.0, 1.0] };
        let v4 = Vertex { position: [0.75, 0.75], tex_coords: [1.0, 0.0] };
        let strip = vec![v1, v2, v3, v4];
        let vertices = glium::VertexBuffer::new(&display, &strip).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

        let vertex_shader_src = include_str!("simple_vertex.glsl");
        let fragment_shader_src = include_str!("simple_output.glsl");
        let program = glium::Program::from_source(
            &display, vertex_shader_src, fragment_shader_src, None).unwrap();


        let uniforms = uniform! {
            tex: &image,
        };

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(&vertices, &indices, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => return,
                _ => (),
            }
        }
    }
}
