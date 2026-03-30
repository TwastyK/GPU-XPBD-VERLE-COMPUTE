use bevy::prelude::*;
use bevy::render::render_graph::{Node, NodeRunError, RenderGraphContext};
use bevy::render::renderer::RenderContext;
use bevy::render::render_resource::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::storage::GpuShaderStorageBuffer;
use crate::PhysicsPipeline::PhysicsPipeline;
use crate::physics_asset::RenderPhysicsBuffers;
use crate::structures::Particle;

// Создаем ресурс для хранения ID готового пайплайна в Render World
#[derive(Resource, Default)]
pub struct SpecializedPhysicsPipelineId(pub Option<CachedComputePipelineId>);

pub struct PhysicsNode;

impl Node for PhysicsNode {
    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext<'_>,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let physics_pipeline = world.resource::<PhysicsPipeline>();

        // Получаем ID, который мы подготовили заранее в системе специализации
        let pipeline_id = world.resource::<SpecializedPhysicsPipelineId>();

        let Some(actual_id) = pipeline_id.0 else {
            return Ok(()); // Пайплайн еще не запрошен
        };

        // Проверяем, скомпилировался ли он в PipelineCache
        let Some(compute_pipeline) = pipeline_cache.get_compute_pipeline(actual_id) else {
            return Ok(()); // Еще компилируется
        };

        let storage_buffers = world.resource::<RenderAssets<GpuShaderStorageBuffer>>();
        let buffers = world.get_resource::<RenderPhysicsBuffers>();

        if buffers.is_none() {
            return Ok(()); // Вместо Aborted просто выходим
        }
        let buffers = buffers.unwrap();

        let Some(gpu_particles) = storage_buffers.get(&buffers.particles) else { return Ok(()); };
        let Some(gpu_constraints) = storage_buffers.get(&buffers.constraints) else { return Ok(()); };

        // Используем лейаут, который сохранен в самом ресурсе PhysicsPipeline
        let bind_group = render_context.render_device().create_bind_group(
            "physics_bind_group",
            &physics_pipeline.layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: gpu_particles.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: gpu_constraints.buffer.as_entire_binding(),
                },
            ],
        );

        let mut pass = render_context.command_encoder().begin_compute_pass(&ComputePassDescriptor {
            label: Some("physics_update_pass"),
            timestamp_writes: None,
        });

        pass.set_pipeline(compute_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        // В PhysicsNode.rs внутри fn run
        let num_particles = gpu_particles.buffer.size() / std::mem::size_of::<Particle>() as u64;
        let dispatch_count = ((num_particles as f32) / (64.0)).ceil() as u32;
        Ok(())
    }
}