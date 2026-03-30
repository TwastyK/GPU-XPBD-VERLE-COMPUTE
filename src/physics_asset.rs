use bevy::prelude::*;
use bevy::asset::RenderAssetUsages as RAU;
use bevy::render::storage::ShaderStorageBuffer;
use crate::gltf_parser::parse_model;

use crate::structures::{Particle, DistanceConstraints};

use bevy::render::Extract;

// Эта структура будет жить в мире рендеринга
#[derive(Resource, Clone, Deref)]
pub struct RenderPhysicsBuffers(pub PhysicsBuffers);

pub fn extract_physics_buffers(
    mut commands: Commands,
    // В 0.18.1 Extract<Res<T>> используется напрямую как ссылка
    buffers: Extract<Res<PhysicsBuffers>>,
) {
    // Вставляем копию (Handle — это дешевое клонирование) в Render World
    commands.insert_resource(RenderPhysicsBuffers(buffers.clone()));
}

// Создаем короткие имена для размеров структур
const S_PARTICLE: usize = std::mem::size_of::<Particle>();
const S_CONSTRAINT: usize = std::mem::size_of::<DistanceConstraints>();
#[derive(Resource, Clone)]
pub struct PhysicsBuffers {
    pub particles: Handle<ShaderStorageBuffer>,
    pub constraints: Handle<ShaderStorageBuffer>,
}

pub fn setup_physics_assets(
    mut commands: Commands,
    mut storage_buffers: ResMut<Assets<ShaderStorageBuffer>>
) {

    let template = parse_model("models/my_npc.glb");
    let npc_count = 100000;

    let nodes_count = template.particles.len();
    let constraints_count = template.constraints.len();

    let mut all_particles = Vec::with_capacity(nodes_count * npc_count);
    let mut all_constraints = Vec::with_capacity(constraints_count * npc_count);

    for i in 0..npc_count {
        // 1. Считаем позицию в мире (сетка), чтобы они не стояли в (0,0,0)
        let x = (i % 316) as f32 * 2.0;
        let z = (i / 316) as f32 * 2.0;
        let offset = Vec3::new(x, 0.0, z);

        // 2. Копируем частицы из шаблона и сдвигаем их
        for p in &template.particles {
            let mut new_p = p.clone();
            new_p.pos[0] += offset.x; // Сдвигаем по X
            new_p.pos[2] += offset.z; // Сдвигаем по Z
            all_particles.push(new_p);
        }

        // 3. Тот самый КРИТИЧЕСКИЙ OFFSET индексов
        let index_offset = (i * nodes_count) as u32;
        for c in &template.constraints {
            let mut new_c = c.clone();
            new_c.node_a += index_offset; // Теперь связь указывает на "свою" частицу
            new_c.node_b += index_offset;
            all_constraints.push(new_c);
        }
    }
    // Определяем права доступа
    let usages = RAU::RENDER_WORLD | RAU::MAIN_WORLD;

    // Превращаем векторы структур в байты для GPU
    let p_bytes: &[u8] = bytemuck::cast_slice(&all_particles);
    let c_bytes: &[u8] = bytemuck::cast_slice(&all_constraints);

    // 4. Создаем буферы через .new()
    let p_buffer = ShaderStorageBuffer::new(p_bytes, usages);
    let c_buffer = ShaderStorageBuffer::new(c_bytes, usages);

    // 5. Регистрируем ресурсы
    commands.insert_resource(PhysicsBuffers {
        particles: storage_buffers.add(p_buffer),
        constraints: storage_buffers.add(c_buffer),
    });
}