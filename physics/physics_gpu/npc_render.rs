use bevy::prelude::*;
use bevy::render::{
    render_resource::*,
    render_phase::{RenderCommand, RenderCommandResult, PhaseItem, SetItemPipeline, TrackedRenderPass},
    render_asset::RenderAssets,
    storage::{ShaderStorageBuffer, GpuShaderStorageBuffer},
    Extract,
};
use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::{SystemParamItem, lifetimeless::SRes};
use bevy::shader::ShaderRef;
use bytemuck::{Pod, Zeroable, cast_slice};
use bevy::pbr::GpuMeshPreprocessPlugin;
use bevy::render::mesh::{RenderMesh, RenderMeshBufferInfo};
use crate::gltf_parser::parse_model;
use crate::structures::{Particle, DistanceConstraints};

// --- МАТЕРИАЛ ---
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct NpcMaterial {
    #[storage(0, read_only)]
    pub buffer: Handle<ShaderStorageBuffer>,
}

impl Material for NpcMaterial {
    fn vertex_shader() -> ShaderRef { "shaders/draw.wgsl".into() }
    fn fragment_shader() -> ShaderRef { "shaders/draw.wgsl".into() }
}

// --- СТРУКТУРЫ ---
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default, ShaderType)]
pub struct DrawIndexedIndirectArgs {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub first_instance: u32,
}

#[derive(Resource, Clone, Deref)]
pub struct RenderPhysicsBuffers(pub PhysicsBuffers);

#[derive(Resource, Clone)]
pub struct PhysicsBuffers {
    pub particles: Handle<ShaderStorageBuffer>,
    pub constraints: Handle<ShaderStorageBuffer>,
    pub indirect_buffer: Handle<ShaderStorageBuffer>,
    pub mesh_indices: Handle<Mesh>,
    pub npc_count: u32,
}

// --- КОМАНДА РЕНДЕРИНГА ---
// Добавь это в npc_render.rs
pub struct DrawNpcCommand;

impl<P: PhaseItem> RenderCommand<P> for DrawNpcCommand {
    type Param = (
        SRes<RenderAssets<GpuShaderStorageBuffer>>,
        SRes<RenderPhysicsBuffers>,
    );
    type ViewQuery = ();
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        _view: (),
        _instance: (),
        (storage_buffers, physics_buffers): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        // Достаем Indirect буфер (команда на отрисовку 50к)
        let Some(indirect_buffer) = storage_buffers.get(&physics_buffers.indirect_cmd) else {
            return RenderCommandResult::Failure;
        };

        // Достаем буфер частиц (позиции костей)
        let Some(particle_buffer) = storage_buffers.get(&physics_buffers.particles) else {
            return RenderCommandResult::Failure;
        };

        // Устанавливаем Bind Group для шейдера (group 2 в draw.wgsl)
        pass.set_bind_group(2, &particle_buffer.bind_group, &[]);

        // Финальный аккорд: говорим GPU рисовать, используя данные из indirect буфера
        pass.draw_indexed_indirect(&indirect_buffer.buffer, 0);

        RenderCommandResult::Success
    }
}

// А теперь связываем это в типе DrawNpcPipeline (который у тебя уже есть)
pub type DrawNpcPipeline = (
    SetItemPipeline,
    DrawNpcCommand, // Добавляем нашу команду сюда
);

// --- СИСТЕМЫ ---

pub fn extract_physics_buffers(
    mut commands: Commands,
    buffers: Extract<Res<PhysicsBuffers>>,
) {
    // Логируем перенос данных в Render World (происходит каждый кадр)
    // Можно закомментировать, если будет слишком много спама в консоли
    // info!("Extracting physics buffers to Render World...");
    commands.insert_resource(RenderPhysicsBuffers(buffers.clone()));
}

pub fn setup_npc_rendering(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<NpcMaterial>>,
    buffers: Res<PhysicsBuffers>,
) {
    info!("Setting up NPC rendering manager...");

    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD
    );

    // ВАЖНО: Нам нужно, чтобы в меше было 2106 индексов (как в логе),
    // чтобы пайплайн Bevy позволил сделать отрисовку.
    // Но так как мы используем Indirect, мы просто даем Bevy понять, что рисовать ЕСТЬ ЧТО.

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(NpcMaterial {
            buffer: buffers.particles.clone(),
        })),
        // Делаем AABB огромным, чтобы NPC не исчезали, когда ты отворачиваешься
        bevy::camera::primitives::Aabb::from_min_max(Vec3::splat(-1000.0), Vec3::splat(1000.0)),
    ));

    info!("NPC Manager spawned with Entity ID: Target NPC Count: {}", buffers.npc_count);
}

pub fn setup_physics_assets(
    mut commands: Commands,
    mut storage_buffers: ResMut<Assets<ShaderStorageBuffer>>
) {
    info!("Starting physics assets setup (Origin Engine)...");

    let template = parse_model("src/glb/my_npc.glb");
    let npc_count = 50000;

    info!("Model parsed. Nodes per NPC: {}, Constraints per NPC: {}", template.particles.len(), template.constraints.len());

    let nodes_count = template.particles.len();
    let mut all_particles = Vec::with_capacity(npc_count * nodes_count);
    let mut all_constraints = Vec::with_capacity(npc_count * template.constraints.len());

    info!("Allocating memory for {} NPCs (Total particles: {})", npc_count, npc_count * nodes_count);

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

    let indirect_cmd = DrawIndexedIndirectArgs {
        index_count: template.index_count,
        instance_count: npc_count as u32,
        first_index: 0,
        base_vertex: 0,
        first_instance: 0,
    };

    // Вместо логирования всей структуры целиком:
    info!(
    "Indirect command prepared: indices={}, instances={}, base_vtx={}",
    indirect_cmd.index_count,
    indirect_cmd.instance_count,
    indirect_cmd.base_vertex
    );

    let usages = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;

    let p_handle = storage_buffers.add(ShaderStorageBuffer::new(cast_slice(&all_particles), usages));
    let c_handle = storage_buffers.add(ShaderStorageBuffer::new(cast_slice(&all_constraints), usages));
    let i_handle = storage_buffers.add(ShaderStorageBuffer::new(cast_slice(&[indirect_cmd]), usages));

    info!("Buffers added to Assets. Handles: P:{:?}, C:{:?}, I:{:?}", p_handle.id(), c_handle.id(), i_handle.id());

    commands.insert_resource(PhysicsBuffers {
        particles: p_handle,
        constraints: c_handle,
        indirect_buffer: i_handle,
        mesh_indices: Default::default(),
        npc_count: npc_count as u32,
    });

    info!("PhysicsBuffers resource inserted successfully.");
}