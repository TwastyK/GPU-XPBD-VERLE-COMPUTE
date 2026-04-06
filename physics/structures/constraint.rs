use bevy::render::render_resource::ShaderType;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, ShaderType)]

pub struct DistanceConstraints {
    pub node_a: u32,
    pub node_b: u32,
    pub rest_length: f32,
    pub stiffness: f32,
}