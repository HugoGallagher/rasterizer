use std::{os::raw::c_void, ptr::{null, null_mut}, slice};

use ash::vk::{self, Handle};
use imgui::sys::{ImDrawVert, ImVec2};
use vrg::{buffer::BufferBuilder, descriptors::CreationReference, graphics_pass::{GraphicsPassBuilder, GraphicsPassDrawInfo}, image::ImageBuilder, layer::LayerExecution, math::vec::{Vec2, Vec4}, vertex_buffer::{VertexAttribute, VertexAttributes}, Renderer};

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

pub struct Gui {
    ctx: *mut imgui::sys::ImGuiContext,
}

impl Gui {
    pub fn new(renderer: &mut Renderer, layer_name: &str, pass_name: &str) -> Gui {
        unsafe {
            let ctx = imgui::sys::igCreateContext(null_mut());

            let (w, h) = renderer.get_target_size();

            let io = imgui::sys::igGetIO();
            let font_atlas = (*io).Fonts;

            (*io).DisplaySize = ImVec2::new(w as f32, h as f32);
            (*io).DisplayFramebufferScale = ImVec2::new(1.0, 1.0);
            (*io).DeltaTime = 0.0001;

            let mut font_atlas_width = 0;
            let mut font_atlas_height = 0;
            let mut font_atlas_pixels: *mut u8 = 0 as *mut u8; // TODO: make this safer
            let mut font_atlas_bytes = 0;
            imgui::sys::ImFontAtlas_GetTexDataAsRGBA32(font_atlas, &mut font_atlas_pixels, &mut font_atlas_width, &mut font_atlas_height, &mut font_atlas_bytes);
            let font_atlas_width = font_atlas_width as u32;
            let font_atlas_height = font_atlas_height as u32;

            let font_buffer = BufferBuilder::new()
                .size(size_of::<u32>() * (font_atlas_width * font_atlas_height) as usize)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .properties(vk::MemoryPropertyFlags::DEVICE_LOCAL)
                .build_with_data(&renderer.core, &renderer.device, font_atlas_pixels as *const c_void);

            let font_image_builder = ImageBuilder::new()
                .width(font_atlas_width)
                .height(font_atlas_height)
                .depth(1)
                .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
                .format(vk::Format::R8G8B8A8_UNORM)
                .layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .data(font_buffer);

            renderer.add_images("gui_font", font_image_builder);

            let gui_creation_refs = vec![CreationReference::Sampler("gui_font".to_string())];

            let gui_pass_builder = GraphicsPassBuilder::<ImDrawVertWrapper, u16>::new()
                .vertex_shader("./res/shaders/bin/gui.vert.spv")
                .fragment_shader("./res/shaders/bin/gui.frag.spv")
                .has_verts()
                .indexed()
                .resizable_vertex_buffer()
                .vertex_push_constant::<GUIPushConstant>()
                .fragment_descriptors(gui_creation_refs, &renderer.data)
                .draw_info(GraphicsPassDrawInfo::simple_empty())
                .targets(renderer.get_images("swapchain_image"));

            renderer.add_layer(layer_name, true, LayerExecution::Main);
            renderer.add_graphics_pass(layer_name, pass_name, gui_pass_builder);
            renderer.get_layer_mut(layer_name).set_root_path(pass_name);

            let gui_font_desc_set = renderer.get_layer_mut("base").get_graphics_pass_mut("gui").fragment_descriptors.as_mut().expect("Error with GUI pass creation").sets.first().expect("Error with GUI pass descriptor sets").as_raw();
            (*font_atlas).TexID = gui_font_desc_set as *mut c_void;

            Gui {
                ctx,
            }
        }
    }

    pub fn update<F: Fn()>(&self, f: F) {
        
    }

    pub fn render(&self, renderer: &mut Renderer) {
        unsafe {
            imgui::sys::igNewFrame();

            let mut col = [0.0, 0.0, 0.0, 0.0];

            imgui::sys::igBegin("asdf".as_ptr() as *const i8, null_mut(), 0);
            imgui::sys::igText("asdfasdf".as_ptr() as *const i8);
            imgui::sys::igColorEdit4("aaaa".as_ptr() as *const i8, col.as_mut_ptr(), 0);
            imgui::sys::igEnd();
            
            imgui::sys::igRender();
            let mut io = imgui::sys::igGetIO();
            let (w, h) = renderer.get_target_size();
            (*io).DisplaySize = ImVec2::new(w as f32, h as f32);
            (*io).DisplayFramebufferScale = ImVec2::new(1.0, 1.0);
            (*io).DeltaTime = 0.001;

            let draw_data = imgui::sys::igGetDrawData();
            let mut verts = Vec::<ImDrawVertWrapper>::with_capacity((*draw_data).TotalVtxCount as usize);
            let mut indices = Vec::<u16>::with_capacity((*draw_data).TotalIdxCount as usize);
            let lists_count = (*draw_data).CmdListsCount;
            for i in 0..lists_count {
                let cmd_list = **(*draw_data).CmdLists.wrapping_add(i as usize);
                for vert in slice::from_raw_parts(cmd_list.VtxBuffer.Data as *const ImDrawVert, cmd_list.VtxBuffer.Size as usize) {
                    verts.push(ImDrawVertWrapper { pos: Vec2::new(vert.pos.x, vert.pos.y), uv: Vec2::new(vert.uv.x, vert.uv.y), col: vert.col })
                }
                for index in slice::from_raw_parts(cmd_list.IdxBuffer.Data, cmd_list.IdxBuffer.Size as usize) {
                    indices.push(*index);
                }
            }

            let display_size = (*draw_data).DisplaySize;
            let scale = Vec2 {
                x: 2.0 / display_size.x,
                y: 2.0 / display_size.y,
            };
            // TODO: This won't work with multiple viewports
            let pre_translate = Vec2::zero();
            let gui_push_constant = GUIPushConstant {
                scale,
                pre_translate
            };

            renderer.get_layer_mut("base").fill_vertex_push_constant("gui", &gui_push_constant);

            if verts.len() > 0 {
                renderer.update_vertex_buffer("base", "gui", Some(&verts), Some(&indices));
                renderer.get_layer_mut("base").get_graphics_pass_mut("gui").draw_infos.clear();
                renderer.get_layer_mut("base").get_graphics_pass_mut("gui").draw_infos.push(GraphicsPassDrawInfo::simple_indexed(verts.len(), indices.len()));
            }
        }
    }
}