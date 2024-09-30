use std::{collections::HashMap, f32::consts::PI};

use vrg::math::{mat::Mat4, vec::{Vec2, Vec3}};
use winit::event::{ElementState, VirtualKeyCode};

pub struct Controller {
    pub keys: HashMap<VirtualKeyCode, ElementState>,
    pub view_mat: Mat4,

    pub mouse_pos: Vec2,
    prev_mouse_pos: Vec2,

    sens: f32,

    pos: Vec3,
    vel: Vec3,
    rot: Vec3,
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            keys: HashMap::new(),
            view_mat: Mat4::identity(),

            mouse_pos: Vec2::new(0.0, 0.0), // TODO: Not accurate
            prev_mouse_pos: Vec2::new(0.0, 0.0),

            sens: 0.001,

            pos: Vec3::new(0.0, 0.0, -3.0),
            vel: Vec3::new(0.0, 0.0, 0.0),
            rot: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn update(&mut self, delta: f32) {
        let mouse_delta = self.mouse_pos - self.prev_mouse_pos;
        self.prev_mouse_pos = self.mouse_pos;

        self.rot.x -= mouse_delta.y * self.sens;
        self.rot.y += mouse_delta.x * self.sens;

        if self.key_down(VirtualKeyCode::W) {
            self.vel.z = -0.2;
        }
        if self.key_down(VirtualKeyCode::S) {
            self.vel.z = 0.2;
        }
        if self.key_down(VirtualKeyCode::A) {
            self.vel.x = -0.2;
        }
        if self.key_down(VirtualKeyCode::D) {
            self.vel.x = 0.2;
        }

        if self.key_down(VirtualKeyCode::Space) {
            self.vel.y = 0.2;
        }
        if self.key_down(VirtualKeyCode::LShift) {
            self.vel.y = -0.2;
        }

        // self.uv_pos.x %= 1.0;
        // self.uv_pos.y %= 1.0;

        self.pos.x += self.vel.x * self.rot.y.cos() - self.vel.z * self.rot.y.sin();
        self.pos.z -= self.vel.x * self.rot.y.sin() + self.vel.z * self.rot.y.cos();
        self.pos.y += self.vel.y;

        let mut view_dir = Vec3::new(0.0, 0.0, 1.0);

        view_dir.x = self.rot.x.cos() * self.rot.y.sin();
        view_dir.y = self.rot.x.sin();
        view_dir.z = self.rot.x.cos() * self.rot.y.cos();

        self.view_mat = Mat4::view(view_dir, self.pos);
        
        self.vel = Vec3::new(0.0, 0.0, 0.0);
    }

    fn key_down(&self, vk: VirtualKeyCode) -> bool {
        match self.keys.get(&vk).unwrap_or(&ElementState::Released) {
            &ElementState::Pressed => true,
            &ElementState::Released => false,
        }
    }
}