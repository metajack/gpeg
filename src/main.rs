#[macro_use]
extern crate glium;

use std::borrow::Cow;
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
        let pixels: Vec<i8> = vec![28, 28, 30, -32, 30, 28, 30, 32,
                                   29, 29, 29, 29, 29, 29, 29, 29,
                                   32, 31, 28, 26, 28, 31, 28, 26,
                                   31, 30, 28, 126, 28, 31, 28, 26,
                                   30, 30, 29, 28, 29, 31, 29, 28,
                                   28, 28, 29, 29, 28, 28, 28, 29,
                                   27, 27, 29, 30, 27, 26, 27, 30,
                                   28, 28, 29, 29, 27, 27, 27, 29];
        let coeffs: Vec<i16> = vec![3673, 24, 13, 2, -4, 0, 21, -15,
                                    61, -3, 3, 45, -12, -5, -9, 8,
                                    -3, -43, -22, -101, 58, 22, -13, 2,
                                    28, -52, -30, -63, 34, 16, -10, 6,
                                    23, -7, -5, 8, -5, -1, 1, 0,
                                    -16, 1, 3, -10, 11, 5, 1, -2,
                                    5, 11, 8, 15, -7, -3, 1, 0,
                                    17, 4, 4, 3, -2, -3, -1, -5];
    
        let pixel_image = glium::texture::RawImage2d {
            data: Cow::Owned(pixels),
            width: 8,
            height: 8,
            format: glium::texture::ClientFormat::I8,
        };

        let pixel_texture = glium::texture::IntegralTexture2d::with_format(&display, pixel_image, glium::texture::UncompressedIntFormat::I8, glium::texture::MipmapsOption::NoMipmap).unwrap();

        let coeff_image = glium::texture::RawImage2d {
            data: Cow::Owned(coeffs),
            width: 8,
            height: 8,
            format: glium::texture::ClientFormat::I16,
        };

        let coeffs_texture = glium::texture::IntegralTexture2d::with_format(&display, coeff_image, glium::texture::UncompressedIntFormat::I16, glium::texture::MipmapsOption::NoMipmap).unwrap();

        let v1 = Vertex { position: [-0.75, -0.75], tex_coords: [0.0, 0.0] };
        let v2 = Vertex { position: [-0.75, 0.75], tex_coords: [0.0, 1.0] };
        let v3 = Vertex { position: [0.75, -0.75], tex_coords: [1.0, 0.0] };
        let v4 = Vertex { position: [0.75, 0.75], tex_coords: [1.0, 1.0] };
        let strip = vec![v1, v2, v3, v4];

        let v1 = Vertex { position: [-0.75, 0.0], tex_coords: [0.0, 0.0] };
        let v2 = Vertex { position: [-0.75, 0.75], tex_coords: [0.0, 1.0] };
        let v3 = Vertex { position: [0.0, 0.0], tex_coords: [1.0, 0.0] };
        let v4 = Vertex { position: [0.0, 0.75], tex_coords: [1.0, 1.0] };
        let left_strip = vec![v1, v2, v3, v4];

        let v1 = Vertex { position: [0.0, 0.0], tex_coords: [0.0, 0.0] };
        let v2 = Vertex { position: [0.0, 0.75], tex_coords: [0.0, 1.0] };
        let v3 = Vertex { position: [0.75, 0.0], tex_coords: [1.0, 0.0] };
        let v4 = Vertex { position: [0.75, 0.75], tex_coords: [1.0, 1.0] };
        let right_strip = vec![v1, v2, v3, v4];

        let vertices_main = glium::VertexBuffer::new(&display, &strip).unwrap();
        let vertices_left = glium::VertexBuffer::new(&display, &left_strip).unwrap();
        let vertices_right = glium::VertexBuffer::new(&display, &right_strip).unwrap();
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
            coeffs: &coeffs_texture,
        };

        let output_pass1 = [("col_top",
                       &glium::texture::IntegralTexture2d::empty_with_format(
                           &display,
                           glium::texture::UncompressedIntFormat::I16,
                           glium::texture::MipmapsOption::NoMipmap,
                           8, 8).unwrap()),
                      ("col_bot",
                       &glium::texture::IntegralTexture2d::empty_with_format(
                           &display,
                           glium::texture::UncompressedIntFormat::I16,
                           glium::texture::MipmapsOption::NoMipmap,
                           8, 8).unwrap())
        ];
        let mut target1 = glium::framebuffer::MultiOutputFrameBuffer::new(&display,
                                                                          output_pass1.iter().cloned())
            .unwrap();
        target1.clear_color(0.0, 0.0, 0.0, 1.0);
        target1.draw(&vertices_main, &indices, &program_pass1, &uniforms_pass1,
                     &Default::default()).unwrap();

        let uniforms_pass2 = uniform! {
            col_top: output_pass1[0].1,
            col_bot: output_pass1[1].1,
        };

        let output_pass2 = [("pack_top",
                             &glium::texture::IntegralTexture2d::empty_with_format(
                                 &display,
                                 glium::texture::UncompressedIntFormat::I16,
                                 glium::texture::MipmapsOption::NoMipmap,
                                 8, 8).unwrap()),
                            ("pack_bot",
                             &glium::texture::IntegralTexture2d::empty_with_format(
                                 &display,
                                 glium::texture::UncompressedIntFormat::I16,
                                 glium::texture::MipmapsOption::NoMipmap,
                                 8, 8).unwrap())
        ];

        let mut target2 = glium::framebuffer::MultiOutputFrameBuffer::new(&display,
                                                                          output_pass2.iter().cloned())
            .unwrap();
        target2.clear_color(0.0, 0.0, 0.0, 1.0);
        target2.draw(&vertices_main, &indices, &program_pass2, &uniforms_pass2,
                     &Default::default()).unwrap();

        let uniforms_pass3 = uniform! {
            pack_top: output_pass2[0].1,
            pack_bot: output_pass2[1].1,
        };

        let output_pass3 = glium::texture::IntegralTexture2d::empty_with_format(
            &display,
            glium::texture::UncompressedIntFormat::I8,
            glium::texture::MipmapsOption::NoMipmap,
            8, 8).unwrap();
        
        let mut target3 = glium::framebuffer::SimpleFrameBuffer::new(&display, &output_pass3).unwrap();
        target3.clear_color(0.0, 0.0, 0.0, 1.0);
        target3.draw(&vertices_main, &indices, &program_pass3, &uniforms_pass3,
                     &Default::default()).unwrap();

        let uniforms_out = uniform! {
            tex: &pixel_texture,
        };
        
        let uniforms_out_left = uniform! {
            tex: &pixel_texture,
        };

        let uniforms_out_right = uniform! {
            tex: &output_pass3,
        };
        
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(&vertices_left, &indices, &program_output, &uniforms_out_left,
                    &Default::default()).unwrap();
        target.draw(&vertices_right, &indices, &program_output, &uniforms_out_right,
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