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
        let pixels: Vec<i8> = vec![80, 46, -49, -61, -118, -52, 48, 53, 
                                   68, -58, -12, -12, 0, 26, -50, 18, 
                                   36, 35, 97, 86, 85, 77, -23, 15, 
                                   28, 54, 90, 78, 74, 42, -41, 7, 
                                   37, 33, -11, 24, 14, -68, -65, -27, 
                                   66, 55, 72, 73, 34, 32, -33, 16, 
                                   82, 37, 60, 70, -2, 1, -31, 80, 
                                   87, 52, 7, 56, -7, -94, 9, 99];
        let coeffs: Vec<i16> = vec![2851, 2238, 754, -890, 1705, -160, 772, -535, 
                                    -1108, -702, 256, 1238, -927, 132, 480, 72, 
                                    -779, -813, 3131, -277, 306, 860, -475, -138, 
                                    -1320, 281, 1404, 277, -449, 279, -383, -289, 
                                    -1150, 222, 1420, -478, -14, 37, -1530, 150, 
                                    1449, -8, -94, -517, -775, -65, 230, -307, 
                                    1043, 131, -163, -235, -584, -2, -487, -22, 
                                    -523, 110, 368, -163, 88, 41, -466, -14];
    
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

        let v1 = Vertex { position: [-0.75, -0.75], tex_coords: [0.0, 1.0] };
        let v2 = Vertex { position: [-0.75, 0.75], tex_coords: [0.0, 0.0] };
        let v3 = Vertex { position: [0.75, -0.75], tex_coords: [1.0, 1.0] };
        let v4 = Vertex { position: [0.75, 0.75], tex_coords: [1.0, 0.0] };
        let strip = vec![v1, v2, v3, v4];

        let v1 = Vertex { position: [-0.75, 0.0], tex_coords: [0.0, 1.0] };
        let v2 = Vertex { position: [-0.75, 0.75], tex_coords: [0.0, 0.0] };
        let v3 = Vertex { position: [0.0, 0.0], tex_coords: [1.0, 1.0] };
        let v4 = Vertex { position: [0.0, 0.75], tex_coords: [1.0, 0.0] };
        let left_strip = vec![v1, v2, v3, v4];

        let v1 = Vertex { position: [0.0, 0.0], tex_coords: [0.0, 1.0] };
        let v2 = Vertex { position: [0.0, 0.75], tex_coords: [0.0, 0.0] };
        let v3 = Vertex { position: [0.75, 0.0], tex_coords: [1.0, 1.0] };
        let v4 = Vertex { position: [0.75, 0.75], tex_coords: [1.0, 0.0] };
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
