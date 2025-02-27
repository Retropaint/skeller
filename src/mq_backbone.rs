use image::{ImageBuffer, ImageReader, Rgba};
use mq::*;
use {egui_miniquad as egui_mq, miniquad as mq};

use crate::bindings::*;

#[repr(C)]
#[derive(Clone, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
pub struct Vertex {
    pub pos: Vec2,
    pub uv: Vec2,
}

#[derive(Clone, Default)]
pub struct Armature {
    pub bones: Vec<Bone>,
}

#[derive(Clone, Default)]
pub struct BoneTexture {
    pub idx: usize, // index relative to skelements texture vector
}

#[derive(Clone, Default)]
pub struct Bone {
    pub name: String,
    pub parent_id: i32,
    pub pos: Vec2,
    pub rot: f32,
    pub scale: Vec2,
    pub id: i32,
    pub tex: BoneTexture,

    // used to properly offset bone's movement to counteract it's parent
    pub parent_rot: f32,
}

#[derive(Default)]
pub struct Camera {
    pub pos: Vec2,
    pub zoom: f32,
}

/// persistent data that lives in stage, but benefits
/// from being passed separately to prevent over-scoping
#[derive(Default)]
pub struct Skelements {
    pub armature: Armature,
    pub selected_bone: usize,
    pub textures: Vec<Texture>,

    // u-related stuff
    pub dragging: bool,
    pub mouse: Vec2,
    pub mouse_pressed: bool,
    pub mouse_pressed_frames: i32,
    pub op_mode: i32, // should be enum but too lazy
    pub mouse_prev: Vec2, // used to get mouse velocity
    pub window_size: Vec2,
    pub hovered_bone: i32,
    pub camera: Camera,

    // debugging
    pub made_test: bool,
}

pub struct Stage {
    pub egui_mq: egui_mq::EguiMq,
    pub mq_ctx: Box<dyn mq::RenderingBackend>,
    pub pipeline: Pipeline,
    pub bindings: Bindings,
    pub skelements: Skelements,
}

#[derive(Default, Clone)]
pub struct Texture {
    pub size: Vec2,
    pub bytes: Vec<u8>,
}

impl Stage {
    pub fn new(textures: Vec<Texture>) -> Self {
        let mut mq_ctx = mq::window::new_rendering_backend();

        let shader = mq_ctx
            .new_shader(
                match mq_ctx.info().backend {
                    Backend::OpenGl => ShaderSource::Glsl {
                        vertex: shader::VERTEX,
                        fragment: shader::FRAGMENT,
                    },
                    Backend::Metal => ShaderSource::Msl {
                        program: shader::METAL,
                    },
                },
                shader::meta(),
            )
            .unwrap();

        let pipeline = mq_ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("in_pos", VertexFormat::Float2),
                VertexAttribute::new("in_uv", VertexFormat::Float2),
            ],
            shader,
            PipelineParams {
                // enable transparency
                alpha_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
        );

        let bindings = placeholder_binding(&mut mq_ctx);

        Self {
            egui_mq: egui_mq::EguiMq::new(&mut *mq_ctx),
            mq_ctx,
            pipeline,
            bindings,
            skelements: Skelements {
                selected_bone: usize::MAX,
                textures: textures,
                camera: Camera{
                    zoom: 1.,
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}

pub mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 in_pos;
    attribute vec2 in_uv;

    uniform vec2 offset;

    varying lowp vec2 texcoord;

    void main() {
        gl_Position = vec4(in_pos + offset, 0, 1);
        texcoord = in_uv;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 texcoord;

    uniform sampler2D tex;

    void main() {
        gl_FragColor = texture2D(tex, texcoord);
    }
    "#;

    pub const METAL: &str = r#"
    #include <metal_stdlib>

    using namespace metal;

    struct Uniforms
    {
        float2 offset;
    };

    struct Vertex
    {
        float2 in_pos   [[attribute(0)]];
        float2 in_uv    [[attribute(1)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
        float2 uv       [[user(locn0)]];
    };

    vertex RasterizerData vertexShader(
      Vertex v [[stage_in]], 
      constant Uniforms& uniforms [[buffer(0)]])
    {
        RasterizerData out;

        out.position = float4(v.in_pos.xy + uniforms.offset, 0.0, 1.0);
        out.uv = v.in_uv;

        return out;
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]], texture2d<float> tex [[texture(0)]], sampler texSmplr [[sampler(0)]])
    {
        return tex.sample(texSmplr, in.uv);
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("offset", UniformType::Float2)],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub offset: (f32, f32),
    }
}

pub fn add_image(path: String, textures: &mut Vec<Texture>) {
    let img = ImageReader::open(path)
        .unwrap()
        .decode()
        .unwrap()
        .flipv()
        .to_rgba8();

    // create iamge buffer with new size
    let mut img_buf = <ImageBuffer<Rgba<u8>, _>>::new(img.width(), img.height());

    // add new image
    for x in 0..img.width() {
        for y in 0..img.height() {
            img_buf.put_pixel(x, y, *img.get_pixel(x, y));
        }
    }
    textures.push(Texture {
        size: Vec2{
            x: img_buf.width() as f32,
            y: img_buf.height() as f32
        },
        bytes: img_buf.to_vec(),
    })
}
