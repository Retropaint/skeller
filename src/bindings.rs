use miniquad::*;

use crate::mq_backbone::Texture;
use crate::mq_backbone::Vec2;
use crate::mq_backbone::Vertex;
use crate::utils::rotate;

pub fn rect_tex_verts(pos: &Vec2, scale: &Vec2, size: &Vec2, rot: f32) -> Vec<Vertex> {
    let hard_scale = 0.001;
    let mut vertices: Vec<Vertex> = vec![
        Vertex {
            pos: Vec2 {
                x: -hard_scale * scale.x * size.x as f32,
                y: -hard_scale * scale.y * size.y as f32,
            },
            uv: Vec2 { x: 0., y: 0. },
        },
        Vertex {
            pos: Vec2 {
                x: hard_scale * scale.x * size.x as f32,
                y: -hard_scale * scale.y * size.y as f32,
            },
            uv: Vec2 { x: 1., y: 0. },
        },
        Vertex {
            pos: Vec2 {
                x: hard_scale * scale.x * size.x as f32,
                y: hard_scale * scale.y * size.y as f32,
            },
            uv: Vec2 { x: 1., y: 1. },
        },
        Vertex {
            pos: Vec2 {
                x: -hard_scale * scale.x * size.x as f32,
                y: hard_scale * scale.y * size.y as f32,
            },
            uv: Vec2 { x: 0., y: 1. },
        },
    ];

    for v in &mut vertices {
        v.pos = rotate(&v.pos, rot);

        // move vertex with bone
        v.pos.x += pos.x;
        v.pos.y += pos.y;
    }

    vertices
}

/// Creates a rectangular texture.
pub fn rect_tex(
    mq_ctx: &mut Box<dyn RenderingBackend>,
    pos: &Vec2,
    scale: &Vec2,
    texture: &Texture,
    rot: f32,
) -> (Bindings, Vec<Vertex>) {
    let vertices = rect_tex_verts(
        &pos,
        &scale,
        &texture.size,
        rot,
    );

    let vertex_buffer = mq_ctx.new_buffer(
        BufferType::VertexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&vertices),
    );

    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
    let index_buffer = mq_ctx.new_buffer(
        BufferType::IndexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&indices),
    );

    let tex =
        mq_ctx.new_texture_from_rgba8(texture.size.x as u16, texture.size.y as u16, &texture.bytes);

    (
        Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![tex],
        },
        vertices,
    )
}

/// For quick testing purposes
pub fn placeholder_binding(mq_ctx: &mut Box<dyn RenderingBackend>) -> Bindings {
    let vertices: [Vertex; 3] = [
        Vertex {
            pos: Vec2 { x: -0.5, y: -0.5 },
            uv: Vec2 { x: 0., y: 0. },
        },
        Vertex {
            pos: Vec2 { x: 0.5, y: -0.5 },
            uv: Vec2 { x: 1., y: 0. },
        },
        Vertex {
            pos: Vec2 { x: 0.5, y: 0.5 },
            uv: Vec2 { x: 1., y: 1. },
        },
    ];
    let vertex_buffer = mq_ctx.new_buffer(
        BufferType::VertexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&vertices),
    );

    let indices: [u16; 3] = [0, 1, 2];
    let index_buffer = mq_ctx.new_buffer(
        BufferType::IndexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&indices),
    );

    #[rustfmt::skip]
    let pixels: [u8; 4 * 4 * 4] = [
        0x00, 0xFF, 0xFF, 0xFF, 
        0x00, 0x00, 0x00, 0xFF, 
        0x00, 0xFF, 0xFF, 0xFF, 
        0x00, 0x00, 0x00, 0xFF, 
        0x00, 0x00, 0x00, 0xFF, 
        0x00, 0xFF, 0xFF, 0xFF, 
        0x00, 0x00, 0x00, 0xFF, 
        0x00, 0xFF, 0xFF, 0xFF, 
        0x00, 0xFF, 0xFF, 0xFF, 
        0x00, 0x00, 0x00, 0x00, 
        0x00, 0xFF, 0xFF, 0xFF, 
        0x00, 0x00, 0x00, 0xFF, 
        0x00, 0x00, 0x00, 0xFF, 
        0x00, 0xFF, 0xFF, 0xFF, 
        0x00, 0x00, 0x00, 0xFF,
        0x00, 0xFF, 0xFF, 0xFF,
    ];
    let texture = mq_ctx.new_texture_from_rgba8(4, 4, &pixels);

    Bindings {
        vertex_buffers: vec![vertex_buffer],
        index_buffer,
        images: vec![texture],
    }
}

pub fn rect_bind(mq_ctx: &mut Box<dyn RenderingBackend>, vertices: &Vec<Vertex>, size: &Vec2, color: [u8; 4]) -> Bindings {
    let vertex_buffer = mq_ctx.new_buffer(
        BufferType::VertexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&vertices),
    );

    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
    let index_buffer = mq_ctx.new_buffer(
        BufferType::IndexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&indices),
    );

    #[rustfmt::skip]
    let texture = mq_ctx.new_texture_from_rgba8(1, 1, &color);

    Bindings {
        vertex_buffers: vec![vertex_buffer],
        index_buffer,
        images: vec![texture],
    }
}

/// Creates a rectangular texture.
pub fn tri_bind(
    mq_ctx: &mut Box<dyn RenderingBackend>,
    pos: Vec2,
    offset: Vec2,
    scale: Vec2,
    rot: f32,
    color: [u8; 4],
) -> (Bindings, Vec<Vertex>) {
    let mut vertices: Vec<Vertex> = vec![
        Vertex {
            pos: Vec2 {
                x: -scale.x + offset.x,
                y: offset.y,
            },
            uv: Vec2::default(),
        },
        Vertex {
            pos: Vec2 {
                x: offset.x,
                y: scale.y + offset.y,
            },
            uv: Vec2::default(),
        },
        Vertex {
            pos: Vec2 {
                x: scale.x + offset.x,
                y: offset.y,
            },
            uv: Vec2::default(),
        },
    ];

    for v in &mut vertices {
        // rotate vertex
        v.pos = Vec2 {
            x: v.pos.x * rot.cos() - v.pos.y * rot.sin(),
            y: v.pos.x * rot.sin() + v.pos.y * rot.cos(),
        };

        // move vertex with bone
        v.pos.x += pos.x;
        v.pos.y += pos.y;
    }

    let vertex_buffer = mq_ctx.new_buffer(
        BufferType::VertexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&vertices),
    );

    let indices: [u16; 3] = [0, 1, 2];
    let index_buffer = mq_ctx.new_buffer(
        BufferType::IndexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&indices),
    );

    #[rustfmt::skip]
    let pixels: [u8; 4] = [color[0], color[1], color[2], 0xFF];
    let tex = mq_ctx.new_texture_from_rgba8(1, 1, &pixels);
    (
        Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![tex],
        },
        vertices,
    )
}
