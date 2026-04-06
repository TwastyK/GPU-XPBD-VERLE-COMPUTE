use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::storage::ShaderStorageBuffer;
use bevy::render::Extract;
use crate::gltf_parser::parse_model;
use crate::structures::{Particle, DistanceConstraints};

#[derive(Resource, Clone, Deref)]
pub struct RenderPhysicsBuffers(pub PhysicsBuffers);

pub fn extract_physics_buffers(
    mut commands: Commands,
    buffers: Extract<Res<PhysicsBuffers>>,
) {
    commands.insert_resource(RenderPhysicsBuffers(buffers.clone()));
}

#[derive(Resource, Clone)]
pub struct PhysicsBuffers {
    pub particles: Handle<ShaderStorageBuffer>,
    pub constraints: Handle<ShaderStorageBuffer>,
    pub indirect_cmd: Handle<ShaderStorageBuffer>, // ДОБАВЬ ЭТО
    pub total_particles: usize,
}

pub fn setup_physics_assets(
    mut commands: Commands,
    mut storage_buffers: ResMut<Assets<ShaderStorageBuffer>>
) {
    let template = parse_model("src/glb/my_npc.glb");
    let npc_count = 50000;

    let nodes_count = template.particles.len();
    let total_particles_count = npc_count * nodes_count;
    let total_constraints_count = npc_count * template.constraints.len();

    info!("--- Physics Initialization ---");
    info!("NPC Count: {}", npc_count);
    info!("Total Particles: {}", total_particles_count);
    info!("Total Constraints: {}", total_constraints_count);

    let mut all_particles = Vec::with_capacity(total_particles_count);
    let mut all_constraints = Vec::with_capacity(total_constraints_count);

    for i in 0..npc_count {
        let offset = Vec3::new((i % 223) as f32 * 3.0, 0.0, (i / 223) as f32 * 3.0);
        let index_offset = (i * nodes_count) as u32;

        for p in &template.particles {
            let mut new_p = *p;
            new_p.pos[0] += offset.x;
            new_p.pos[1] += offset.y;
            new_p.pos[2] += offset.z;
            new_p.old_pos = new_p.pos;
            all_particles.push(new_p);
        }

        for c in &template.constraints {
            let mut new_c = *c;
            new_c.node_a += index_offset;
            new_c.node_b += index_offset;
            all_constraints.push(new_c);
        }
    }

    let p_handle = storage_buffers.add(ShaderStorageBuffer::new(
        bytemuck::cast_slice(&all_particles),
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    ));

    let c_handle = storage_buffers.add(ShaderStorageBuffer::new(
        bytemuck::cast_slice(&all_constraints),
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    ));

    info!("GPU Storage Buffers created successfully.");

    commands.insert_resource(PhysicsBuffers {
        particles: p_handle,
        constraints: c_handle,
        total_particles: total_particles_count,
        indirect_cmd: c_handle,
    });
}