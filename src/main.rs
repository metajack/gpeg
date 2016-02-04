#[macro_use]
extern crate glium;

use std::borrow::Cow;
use std::fs::File;
use std::io::Read;
use glium::{DisplayBuild, Surface};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

fn main() {
    let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();
    loop {
        let mut f = File::open("f1.Y").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        let coeffs: Vec<i16> = s.split_whitespace()
            .map(|coeff_str| i16::from_str_radix(coeff_str, 10).unwrap())
            .collect();

        let plane_width = 1024;
        let plane_height = 576;
        let coeff_image = glium::texture::RawImage2d {
            data: Cow::Owned(coeffs),
            width: plane_width,
            height: plane_height,
            format: glium::texture::ClientFormat::I16,
        };

        let coeffs_texture = glium::texture::IntegralTexture2d::with_format(&display, coeff_image, glium::texture::UncompressedIntFormat::I16, glium::texture::MipmapsOption::NoMipmap).unwrap();

        let v1 = Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] };
        let v2 = Vertex { position: [-1.0, 1.0], tex_coords: [0.0, 1.0] };
        let v3 = Vertex { position: [1.0, -1.0], tex_coords: [1.0, 0.0] };
        let v4 = Vertex { position: [1.0, 1.0], tex_coords: [1.0, 1.0] };
        let strip = vec![v1, v2, v3, v4];

        // 16:9
        let v1 = Vertex { position: [-0.75, -0.09375], tex_coords: [0.0, 1.0] };
        let v2 = Vertex { position: [-0.75, 0.75], tex_coords: [0.0, 0.0] };
        let v3 = Vertex { position: [0.75, -0.09375], tex_coords: [1.0, 1.0] };
        let v4 = Vertex { position: [0.75, 0.75], tex_coords: [1.0, 0.0] };
        let present_strip = vec![v1, v2, v3, v4];

        let vertices_main = glium::VertexBuffer::new(&display, &strip).unwrap();
        let vertices_present = glium::VertexBuffer::new(&display, &present_strip).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

        let vertex_shader_src = include_str!("simple_vertex.glsl");
        let fragment_shader_pass1_src = include_str!("idct8x8_pass1.glsl");
        let fragment_shader_pass2_src = include_str!("idct8x8_pass2.glsl");
        let fragment_shader_pass3_src = include_str!("idct8x8_pass3.glsl");
        let fragment_shader_out_src = include_str!("simple_output.glsl");

        let program_pass1 = glium::Program::from_source(&display, vertex_shader_src,
                                                        fragment_shader_pass1_src, None).unwrap();
        let program_pass2 = glium::Program::from_source(&display, vertex_shader_src,
                                                        fragment_shader_pass2_src, None).unwrap();
        let program_pass3 = glium::Program::from_source(&display, vertex_shader_src,
                                                        fragment_shader_pass3_src, None).unwrap();
        let program_output = glium::Program::from_source(&display, vertex_shader_src,
                                                         fragment_shader_out_src, None).unwrap();

        let uniforms_pass1 = uniform! {
            plane_dims: [plane_width as i32, plane_height as i32],
            coeffs: &coeffs_texture,
        };

        let col_top = glium::texture::IntegralTexture2d::empty_with_format(
            &display,
            glium::texture::UncompressedIntFormat::I16I16I16I16,
            glium::texture::MipmapsOption::NoMipmap,
            plane_width, plane_height).unwrap();
        let col_bot = glium::texture::IntegralTexture2d::empty_with_format(
            &display,
            glium::texture::UncompressedIntFormat::I16I16I16I16,
            glium::texture::MipmapsOption::NoMipmap,
            plane_width, plane_height).unwrap();
        let output_pass1 = [
            ("col_top", &col_top),
            ("col_bot", &col_bot)
        ];
        let mut target1 = glium::framebuffer::MultiOutputFrameBuffer::new(&display,
                                                                          output_pass1.iter().cloned())
            .unwrap();
        target1.draw(&vertices_main, &indices, &program_pass1, &uniforms_pass1,
                     &Default::default()).unwrap();

        let uniforms_pass2 = uniform! {
            plane_dims: [plane_width as i32, plane_height as i32],
            col_top: output_pass1[0].1,
            col_bot: output_pass1[1].1,
        };

        let pack_top = glium::texture::IntegralTexture2d::empty_with_format(
            &display,
            glium::texture::UncompressedIntFormat::I16I16I16I16,
            glium::texture::MipmapsOption::NoMipmap,
            plane_width, plane_height).unwrap();
        let pack_bot = glium::texture::IntegralTexture2d::empty_with_format(
            &display,
            glium::texture::UncompressedIntFormat::I16I16I16I16,
            glium::texture::MipmapsOption::NoMipmap,
            plane_width, plane_height).unwrap();
        let output_pass2 = [
            ("pack_top", &pack_top),
            ("pack_bot", &pack_bot)
        ];

        let mut target2 = glium::framebuffer::MultiOutputFrameBuffer::new(
            &display,
            output_pass2.iter().cloned()).unwrap();
        target2.draw(&vertices_main, &indices, &program_pass2, &uniforms_pass2,
                     &Default::default()).unwrap();

        let uniforms_pass3 = uniform! {
            plane_dims: [plane_width as i32, plane_height as i32],
            pack_top: output_pass2[0].1,
            pack_bot: output_pass2[1].1,
        };

        let output_pass3 = glium::texture::IntegralTexture2d::empty_with_format(
            &display,
            glium::texture::UncompressedIntFormat::I8,
            glium::texture::MipmapsOption::NoMipmap,
            plane_width, plane_height).unwrap();
        
        let mut target3 = glium::framebuffer::SimpleFrameBuffer::new(&display, &output_pass3).unwrap();
        target3.draw(&vertices_main, &indices, &program_pass3, &uniforms_pass3,
                     &Default::default()).unwrap();

        let uniforms_out = uniform! {
            plane_dims: [plane_width as i32, plane_height as i32],
            tex: &output_pass3,
        };

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(&vertices_present, &indices, &program_output, &uniforms_out,
                    &Default::default()).unwrap();
        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => return,
                _ => (),
            }
        }
    }
}
