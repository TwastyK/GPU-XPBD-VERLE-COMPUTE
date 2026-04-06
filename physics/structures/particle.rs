use bevy::render::render_resource::ShaderType;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default, ShaderType)]

pub struct Particle {
    pub pos: [f32; 4],
    pub old_pos: [f32; 4],
    pub force: [f32; 4],
    pub inv_mass: [f32; 4],
}