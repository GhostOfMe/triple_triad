#![allow(unused_imports, dead_code)]
use crevice::std140::AsStd140;
use ggez::event;
use ggez::glam::{Mat4, Vec3, u32};
use ggez::graphics;
use ggez::{Context, GameResult};
use std::env;
use std::f32;
use std::path;
use wgpu::util::DeviceExt;

type Isometry3 = Mat4;
type Point3 = Vec3;
type Vector3 = Vec3;

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct Vertex {
    pos: [f32; 4],
    tex_coord: [f32; 2],
}

impl Vertex {
    fn new(p: [i8; 3], t: [i8; 2]) -> Self {
        Self {
            pos: [f32::from(p[0]), f32::from(p[1]), f32::from(p[2]), 1.0],
            tex_coord: [f32::from(t[0]), f32::from(t[1])],
        }
    }
}

#[derive(AsStd140)]
struct Locals {
    transform: mint::ColumnMatrix4<f32>,
    rotation: mint::ColumnMatrix4<f32>,
}

fn default_view() -> Isometry3 {
    // Eye location, target location, up-vector
    Mat4::look_at_rh(
        Point3::new(0f32, -30.0, 0.0),
        Point3::new(0f32, 0.0, 0.0),
        Vector3::Z,
    )
}

fn view(x_offset: f32) -> Isometry3 {
    Mat4::look_at_rh(
        Point3::new(x_offset, -40.0, -13.0),
        Point3::new(x_offset, 0.0, -13.0),
        Vector3::Z,
    )
}

pub struct Shape {
    frames: usize,
    transform: mint::ColumnMatrix4<f32>,
    rotation: f32,

    verts: wgpu::Buffer,
    inds: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    locals: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    depth: graphics::ScreenImage,
    pow: Mat4,
}

impl Shape {
    pub fn new(ctx: &mut Context, x_offset: f32) -> Self {
        // Shaders.
        let shader = ctx
            .gfx
            .wgpu()
            .device
            .create_shader_module(wgpu::include_wgsl!("../../assets/cube.wgsl"));

        // Cube geometry
        #[rustfmt::skip]
        let vertex_data = [
            // top (0, 0, 1)
            Vertex::new([-1, -1,  1], [0, 0]),
            Vertex::new([ 1, -1,  1], [1, 0]),
            Vertex::new([ 1,  1,  1], [1, 1]),
            Vertex::new([-1,  1,  1], [0, 1]),
            // right (1, 0, 0)
            Vertex::new([ 1,  1,  1],  [1, 0]),
            Vertex::new([ 1, -1,  1],  [0, 0]),
            Vertex::new([ 0,  0, -1],  [1, 1]),
            // left (-1, 0, 0)
            Vertex::new([-1, -1,  1], [1, 0]),
            Vertex::new([-1,  1,  1], [0, 0]),
            Vertex::new([ 0,  0, -1], [0, 1]),
            // front (0, 1, 0)
            Vertex::new([ 0,  0, -1], [1, 0]),
            Vertex::new([-1,  1,  1], [0, 1]),
            Vertex::new([ 1,  1,  1], [1, 1]),
            // back (0, -1, 0)
            Vertex::new([ 1, -1,  1], [0, 0]),
            Vertex::new([-1, -1,  1], [1, 0]),
            Vertex::new([ 0,  0, -1], [1, 1]),

        ];

        #[rustfmt::skip]
        let index_data: &[u32] = &[
             0,  1,  2,  2,  3,  0, // top
             4,  5,  6,             // right
             7,  8,  9,             // left
            10, 11, 12,             // front
            13, 14, 15,             // back
        ];

        // Create vertex and index buffers.
        let verts = ctx
            .gfx
            .wgpu()
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(vertex_data.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let inds = ctx
            .gfx
            .wgpu()
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(index_data),
                usage: wgpu::BufferUsages::INDEX,
            });

        let pipeline =
            ctx.gfx
                .wgpu()
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: None,
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<Vertex>() as _,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[
                                // pos
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x4,
                                    offset: 0,
                                    shader_location: 0,
                                },
                                // tex_coord
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x2,
                                    offset: 16,
                                    shader_location: 1,
                                },
                            ],
                        }],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Greater,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: ctx.gfx.surface_format(),
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::SrcAlpha,
                                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                    operation: wgpu::BlendOperation::Add,
                                },
                                alpha: wgpu::BlendComponent::OVER,
                            }),

                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                });

        // Create 1-pixel blue texture.
        let image = graphics::Image::from_solid(ctx, 1, graphics::Color::from_rgb_u32(0x00e0_e000));

        let sampler = ctx
            .gfx
            .wgpu()
            .device
            .create_sampler(&graphics::Sampler::default().into());

        let locals = ctx
            .gfx
            .wgpu()
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: Locals::std140_size_static() as _,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                mapped_at_creation: false,
            });

        let bind_group = ctx
            .gfx
            .wgpu()
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &locals,
                            offset: 0,
                            size: None,
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(image.wgpu().1),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

        let depth = graphics::ScreenImage::new(ctx, graphics::ImageFormat::Depth32Float, 1., 1., 1);

        // FOV, spect ratio, znear, zfar
        let proj = Mat4::perspective_rh(f32::consts::PI / 4.0, 4.0 / 3.0, 1.0, 50.0);
        let transform = proj * view(x_offset);

        Self {
            frames: 0,
            transform: transform.into(),
            rotation: 0.0,
            verts,
            inds,
            pipeline,
            locals,
            bind_group,
            depth,
            pow: view(x_offset),
        }
    }

    pub fn update(&mut self, _ctx: &mut Context) {
        self.rotation += 0.01;
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        let locals = Locals {
            transform: self.transform,
            rotation: Mat4::from_rotation_z(self.rotation).into(),
        };
        ctx.gfx
            .wgpu()
            .queue
            .write_buffer(&self.locals, 0, locals.as_std140().as_bytes());

        let depth = self.depth.image(ctx);

        let frame = ctx.gfx.frame().clone();
        let cmd = ctx.gfx.commands().unwrap();
        let mut pass = cmd.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: frame.wgpu().1,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth.wgpu().1,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.),
                    store: false,
                }),
                stencil_ops: None,
            }),
        });
        pass.set_blend_constant(wgpu::Color::TRANSPARENT);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.verts.slice(..));
        pass.set_index_buffer(self.inds.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..18, 0, 0..1);
    }
}
