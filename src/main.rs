#[macro_use]
extern crate glium;
extern crate gpeg;

use gpeg::{pack_coeffs, read_data, Plane};
use std::borrow::Cow;
use std::rc::Rc;
use glium::{DisplayBuild, Surface};
use glium::backend::Facade;

#[derive(Copy, Clone)]
struct QuadVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(QuadVertex, position, tex_coords);

#[derive(Copy, Clone)]
struct BlockVertex {
    position: [f32; 2],
}

implement_vertex!(BlockVertex, position);

struct DecodeContext {
    facade: Rc<glium::backend::Context>,
    width: u32,
    height: u32,
    block_vertices: glium::vertex::VertexBuffer<BlockVertex>,
    block_vertices_dec: glium::vertex::VertexBuffer<BlockVertex>,
    quad_vertices: glium::vertex::VertexBuffer<QuadVertex>,
    quad_vertices_dec: glium::vertex::VertexBuffer<QuadVertex>,
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
        let block_vertices = glium::VertexBuffer::new(
            &facade,
            &make_block_vertices(width >> 3, height >> 3)).unwrap();
        let block_vertices_dec = glium::VertexBuffer::new(
            &facade,
            &make_block_vertices(width >> 4, height >> 4)).unwrap();
        
        let v1 = QuadVertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] };
        let v2 = QuadVertex { position: [-1.0, 1.0], tex_coords: [0.0, 1.0] };
        let v3 = QuadVertex { position: [1.0, -1.0], tex_coords: [1.0, 0.0] };
        let v4 = QuadVertex { position: [1.0, 1.0], tex_coords: [1.0, 1.0] };
        let strip = vec![v1, v2, v3, v4];
        let quad_vertices = glium::VertexBuffer::new(&facade, &strip).unwrap();

        let v1 = QuadVertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] };
        let v2 = QuadVertex { position: [-1.0, 0.0], tex_coords: [0.0, 1.0] };
        let v3 = QuadVertex { position: [0.0, -1.0], tex_coords: [1.0, 0.0] };
        let v4 = QuadVertex { position: [0.0, 0.0], tex_coords: [1.0, 1.0] };
        let strip = vec![v1, v2, v3, v4];
        let quad_vertices_dec = glium::VertexBuffer::new(&facade, &strip).unwrap();

        let vertex_shader_src = include_str!("simple_vertex.glsl");
        let fragment_shader_pass1_src = include_str!("idct8x8_pass1.glsl");
        let fragment_shader_pass2_src = include_str!("idct8x8_pass2.glsl");
        let fragment_shader_pass3_src = include_str!("idct8x8_pass3.glsl");
        let fragment_shader_convert_src = include_str!("convert.glsl");
        let program_pass1 = program!(
            &facade,
            140 => {
                vertex: vertex_shader_src,
                fragment: fragment_shader_pass1_src,
            }
        ).unwrap();
        let program_pass2 = program!(
            &facade,
            140 => {
                vertex: vertex_shader_src,
                fragment: fragment_shader_pass2_src,
            }
        ).unwrap();
        let program_pass3 = program!(
            &facade,
            140 => {
                vertex: vertex_shader_src,
                fragment: fragment_shader_pass3_src,
            }
        ).unwrap();
        let program_convert = program!(
            &facade,
            140 => {
                vertex: vertex_shader_src,
                fragment: fragment_shader_convert_src,
            }
        ).unwrap();

        // the intermediate textures can be width/8 because we only need hte
        // first column of each block
        let pass1_top = glium::texture::IntegralTexture2d::empty_with_format(
            &facade,
            glium::texture::UncompressedIntFormat::I16I16I16I16,
            glium::texture::MipmapsOption::NoMipmap,
            width / 8, height).unwrap();
        let pass1_bot = glium::texture::IntegralTexture2d::empty_with_format(
            &facade,
            glium::texture::UncompressedIntFormat::I16I16I16I16,
            glium::texture::MipmapsOption::NoMipmap,
            width / 8, height).unwrap();
        let pass2_top = glium::texture::IntegralTexture2d::empty_with_format(
            &facade,
            glium::texture::UncompressedIntFormat::I16I16I16I16,
            glium::texture::MipmapsOption::NoMipmap,
            width / 8, height).unwrap();
        let pass2_bot = glium::texture::IntegralTexture2d::empty_with_format(
            &facade,
            glium::texture::UncompressedIntFormat::I16I16I16I16,
            glium::texture::MipmapsOption::NoMipmap,
            width / 8, height).unwrap();

        DecodeContext {
            facade: facade,
            width: width,
            height: height,
            block_vertices: block_vertices,
            block_vertices_dec: block_vertices_dec,
            quad_vertices: quad_vertices,
            quad_vertices_dec: quad_vertices_dec,
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

fn decode_plane(ctx: &DecodeContext, plane: &Plane) -> glium::texture::IntegralTexture2d {
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);
    let vertices = if plane.width == ctx.width && plane.height == ctx.height {
        &ctx.quad_vertices
    } else {
        &ctx.quad_vertices_dec
    };
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
        target1.draw(vertices, &indices, &ctx.program_pass1, &uniforms_pass1,
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
    target2.draw(vertices, &indices, &ctx.program_pass2, &uniforms_pass2,
                 &Default::default()).unwrap();
    let uniforms_pass3 = uniform! {
        plane_dims: [plane.width as i32, plane.height as i32],
        pass2_top: &ctx.pass2_top,
        pass2_bot: &ctx.pass2_bot,
    };
    {
        let mut target3 = glium::framebuffer::SimpleFrameBuffer::new(&ctx.facade, &data_texture).unwrap();
        target3.draw(vertices, &indices, &ctx.program_pass3, &uniforms_pass3,
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
        target.draw(&ctx.quad_vertices, &indices, &ctx.program_convert, &uniforms,
                    &Default::default()).unwrap();
    }
    output
}

fn make_block_vertices(width: u32, height: u32) -> Vec<BlockVertex> {
    let mut vertices: Vec<BlockVertex> = Vec::with_capacity((width * height) as usize);
    for j in 0..width {
        for i in 0..height {
            vertices.push(BlockVertex {
                position: [i as f32 / width as f32, j as f32 / height as f32],
            });
        }
    }
    vertices
}

fn main() {
    let display = glium::glutin::WindowBuilder::new().with_dimensions(1024, 1024).build_glium().unwrap();
    loop {
        let width = 1024;
        let height = 576;

        let y_plane = Plane {
            width: width,
            height: height,
            data: read_data("f1.Y"),
        };
        let cb_plane = Plane {
            width: width / 2,
            height: height / 2,
            data: read_data("f1.Cb"),
        };
        let cr_plane = Plane {
            width: width / 2,
            height: height / 2,
            data: read_data("f1.Cr"),
        };

        assert!((y_plane.width * y_plane.height) as usize == y_plane.data.len());
        assert!((cb_plane.width * cb_plane.height) as usize == cb_plane.data.len());
        assert!((cr_plane.width * cr_plane.height) as usize == cr_plane.data.len());

        let ctx = DecodeContext::new(display.get_context().clone(), y_plane.width, y_plane.height);
        let planes = vec![y_plane, cb_plane, cr_plane];

        let (mut packed_coeffs, pack_indices) = pack_coeffs(planes[0].width, planes[0].height, &planes[0].data);
        let packed_index_buffer = unsafe {
            let bindings = Cow::Owned(vec![
                (Cow::Borrowed("pack_index"), 0, glium::vertex::AttributeType::U32)
            ]);
            glium::VertexBuffer::new_raw(&ctx.facade,
                                         &pack_indices,
                                         bindings,
                                         ::std::mem::size_of::<u32>())
        };

        let overage = packed_coeffs.len() % 512;
        if overage > 0 {
            let extra = 512 - overage;
            packed_coeffs.reserve(extra);
            for _ in 0..extra {
                packed_coeffs.push(0);
            }
        }
        let packed_image = glium::texture::RawImage2d {
            data: Cow::Borrowed(&packed_coeffs),
            width: 512,
            height: (packed_coeffs.len() / 512) as u32,
            format: glium::texture::ClientFormat::U16,
        };
        println!("packed image 1d width = {}", packed_coeffs.len());
        let packed_texture = glium::texture::UnsignedTexture2d::with_format(
            &ctx.facade,
            packed_image,
            glium::texture::UncompressedUintFormat::U16,
            glium::texture::MipmapsOption::NoMipmap).unwrap();
            

        let output: Vec<_> = planes.iter().map(|p| decode_plane(&ctx, p)).collect();

        let image = convert_planes(&ctx, 1024, 576, &output);

        // 16:9
        let v1 = QuadVertex { position: [-0.75, -0.09375], tex_coords: [0.0, 1.0] };
        let v2 = QuadVertex { position: [-0.75, 0.75], tex_coords: [0.0, 0.0] };
        let v3 = QuadVertex { position: [0.75, -0.09375], tex_coords: [1.0, 1.0] };
        let v4 = QuadVertex { position: [0.75, 0.75], tex_coords: [1.0, 0.0] };
        let strip = vec![v1, v2, v3, v4];
        let vertices = glium::VertexBuffer::new(&display, &strip).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

        let vertex_shader_src = include_str!("simple_vertex.glsl");
        let fragment_shader_src = include_str!("simple_output.glsl");
        let program = program!(
            &display,
            140 => {
                vertex: vertex_shader_src,
                fragment: fragment_shader_src,
                outputs_srgb: true,
            }
        ).unwrap();


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
