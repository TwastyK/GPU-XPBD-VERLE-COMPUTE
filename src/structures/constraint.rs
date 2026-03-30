use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]

pub struct DistanceConstraints {
    pub node_a: u32,
    pub node_b: u32,
    pub rest_length: f32,
    pub stiffness: f32,
}