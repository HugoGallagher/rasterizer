use core::slice;
use std::{f32::consts::{PI, TAU}, os::raw::c_void, ptr::{null, null_mut}};

use imgui::{sys::{ImDrawVert, ImFontAtlasFlags, ImTextureID, ImVec2}, FontId};
use vrg::{buffer::BufferBuilder, compute_pass::{ComputePassBuilder, ComputePassDispatchInfo}, descriptors::{storage_descriptor::{self, StorageDescriptorBuilder}, BindingReference, CreationReference, DescriptorsBuilder}, graphics_pass::{GraphicsPassBuilder, GraphicsPassDrawInfo}, image::{Image, ImageBuilder}, layer::{LayerExecution, PassDependency}, math::{mat::Mat4, vec::{Vec2, Vec3, Vec4}}, mesh::{self, parse_obj_as_tris, FromObjTri}, renderer_data::ResourceReference, shader::ShaderType, vertex_buffer::{NoVertices, VertexAttribute, VertexAttributes}};

use vrg::Renderer;
use vrg::util::frametime::Frametime;

use std::collections::HashMap;
use ash::vk::{self, Handle, MicromapBuildSizesInfoEXT};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use winit::event::{VirtualKeyCode, ElementState};

use crate::{controller::Controller, gui::Gui};
use crate::objects::mesh::Mesh;

#[repr(C)]
#[repr(align(16))]
#[derive(Copy, Clone)]
struct ImDrawVertWrapper {
    pub pos: Vec2,
    pub uv: Vec2,
    pub col: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct GUIPushConstant {
    pub scale: Vec2,
    pub pre_translate: Vec2,
}

impl VertexAttributes for ImDrawVertWrapper {
    fn get_attribute_data() -> Vec<vrg::vertex_buffer::VertexAttribute> {
        vec![
            VertexAttribute { format: vk::Format::R32G32_SFLOAT, offset: 0 },
            VertexAttribute { format: vk::Format::R32G32_SFLOAT, offset: 8 },
            VertexAttribute { format: vk::Format::R8G8B8A8_UNORM, offset: 16 },
        ]
    }
}

impl Default for ImDrawVertWrapper {
    fn default() -> Self {
        Self {
            pos: Vec2::zero(),
            uv: Vec2::zero(),
            col: 0,
        }
    }
}

#[repr(C)]
pub struct MeshPushConstant {
    pub view_proj: Mat4,
}

pub struct App {
    pub renderer: Renderer,
    pub controller: Controller,
    pub gui: Gui,

    pub screen_res: Vec2,

    pub frametime: Frametime,

    mesh_push_constant: MeshPushConstant,

    pub ctx: *mut imgui::sys::ImGuiContext,
}

impl App {
    pub unsafe fn new(window: RawWindowHandle, display: RawDisplayHandle, r: Vec2) -> App {
        let mesh_push_constant = MeshPushConstant {
            view_proj: Mat4::identity(),
        };

        let monkey_mesh = Mesh::from_obj("./res/meshes/asdf.obj");

        let mut renderer = Renderer::new(window, display, true);
        let gui = Gui::new(&mut renderer, "base", "gui");
        
        let mut app = App {
            renderer,
            controller: Controller::new(),
            gui,

            screen_res: r,

            frametime: Frametime::new(),

            mesh_push_constant,

            ctx: imgui::sys::igCreateContext(null_mut()),
        };

        let cw: usize = 20;
        let mesh_count: usize = cw * cw * cw;
        let mut mesh_data = Vec::<Mat4>::with_capacity(mesh_count);
        for i in 0..cw {
            for j in 0..cw {
                for k in 0..cw {
                    mesh_data.push(Mat4::identity());
                    mesh_data[i * cw * cw + j * cw + k].w.x = i as f32 * 2.0;
                    mesh_data[i * cw * cw + j * cw + k].w.y = j as f32 * 2.0;
                    mesh_data[i * cw * cw + j * cw + k].w.z = k as f32 * 2.0;
                }
            }
        }

        let storage_buffer = BufferBuilder::new()
            .size(mesh_data.len() * size_of::<Mat4>())
            .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .properties(vk::MemoryPropertyFlags::DEVICE_LOCAL);

        app.renderer.add_buffers("mesh_transforms", storage_buffer, Some(mesh_data.as_ptr()));

        let mesh_pass_creation_refs = vec![CreationReference::Storage("mesh_transforms".to_string())];

        let mesh_pass_builder = GraphicsPassBuilder::new()
            .vertex_shader("./res/shaders/bin/mesh.vert.spv")
            .fragment_shader("./res/shaders/bin/mesh.frag.spv")
            .draw_info(GraphicsPassDrawInfo::instanced_indexed(monkey_mesh.verts.len(), monkey_mesh.indices.len(), mesh_count))
            .targets(app.renderer.get_images("swapchain_image"))
            .vertex_descriptors(mesh_pass_creation_refs, &app.renderer.data)
            .verts(&monkey_mesh.verts)
            .vertex_indices(&monkey_mesh.indices)
            .vertex_push_constant::<MeshPushConstant>()
            .clear_col(Vec4::new(0.82, 0.8, 0.9, 1.0))
            .with_depth_buffer();

        app.renderer.add_graphics_pass("base", "mesh_draw", mesh_pass_builder);

        let dep = PassDependency {
            resource: ResourceReference::Image(app.renderer.data.get_image_refs("swapchain_image")),
            src_access: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            src_stage: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_shader: ShaderType::Fragment,
            dst_access: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_shader: ShaderType::Fragment,
        };

        app.renderer.add_pass_dependency("base", "mesh_draw", "gui", Some(dep));
        
        app.renderer.get_layer_mut("base").set_root_path("gui");
        app.renderer.set_root_layer("base");

        app
    }

    pub unsafe fn main_loop(&mut self) {
        let delta = self.frametime.get_delta();

        self.frametime.refresh();

        self.update(delta);
        self.frametime.set("Game");

        self.draw();
        self.frametime.set("Draw");

        //println!("{}", self.frametime);
    }

    pub fn update(&mut self, delta: f32) {
        self.controller.update(delta);

        self.mesh_push_constant.view_proj = (self.controller.view_mat * Mat4::perspective(16.0 / 9.0, PI / 2.0, 0.0005, 100.0)).transpose();
    }

    pub unsafe fn draw(&mut self) {
        self.renderer.pre_draw();
        
        self.gui.render(&mut self.renderer);

        self.renderer.get_layer_mut("base").fill_vertex_push_constant("mesh_draw", &self.mesh_push_constant);

        self.renderer.draw();
    }

    pub fn update_key(&mut self, vk: VirtualKeyCode, s: ElementState) {
        self.controller.keys.insert(vk, s);
    }

    pub fn update_mouse(&mut self, d: Vec2) {
        self.controller.mouse_pos += d;
    }
}