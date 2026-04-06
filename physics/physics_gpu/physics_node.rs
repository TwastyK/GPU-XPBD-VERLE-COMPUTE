use bevy::prelude::*;
use bevy::render::render_graph::{Node, NodeRunError, RenderGraphContext};
use bevy::render::renderer::RenderContext;
use bevy::render::render_resource::*;
use bevy::render::storage::GpuShaderStorageBuffer;
use bevy::render::render_asset::RenderAssets;

use crate::physics_pipeline::{PhysicsPipeline, SpecializedPhysicsPipelineIds};
use crate::physics_asset::RenderPhysicsBuffers;
use crate::structures::{Particle, DistanceConstraints};

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
        let ids = world.resource::<SpecializedPhysicsPipelineIds>();
        let storage_buffers = world.resource::<RenderAssets<GpuShaderStorageBuffer>>();

        let Some(buffers) = world.get_resource::<RenderPhysicsBuffers>() else { return Ok(()); };
        let Some(gpu_particles) = storage_buffers.get(&buffers.particles) else { return Ok(()); };
        let Some(gpu_constraints) = storage_buffers.get(&buffers.constraints) else { return Ok(()); };

        // Оставляем один BindGroup, так как лейаут у проходов одинаковый
        let bind_group = render_context.render_device().create_bind_group(
            "physics_bind_group",
            &physics_pipeline.layout,
            &[
                BindGroupEntry { binding: 0, resource: gpu_particles.buffer.as_entire_binding() },
                BindGroupEntry { binding: 1, resource: gpu_constraints.buffer.as_entire_binding() },
            ],
        );

        let num_particles = gpu_particles.buffer.size() / std::mem::size_of::<Particle>() as u64;
        let num_constraints = gpu_constraints.buffer.size() / std::mem::size_of::<DistanceConstraints>() as u64;

        let dispatch_particles = ((num_particles as f32) / 64.0).ceil() as u32;
        let dispatch_constraints = ((num_constraints as f32) / 64.0).ceil() as u32;

        // --- ПРОХОД 1: ИНТЕГРАЦИЯ ---
        if let Some(int_id) = ids.integration {
            if let Some(pipeline) = pipeline_cache.get_compute_pipeline(int_id) {
                let mut pass = render_context.command_encoder().begin_compute_pass(&ComputePassDescriptor {
                    label: Some("physics_integration_pass"),
                    ..default()
                });
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(dispatch_particles, 1, 1);
            }
        }

        // --- ПРОХОД 2: СОЛВЕР (XPBD Constraints) ---
        // Запускаем 4 итерации для стабильности
        if let Some(solv_id) = ids.solver {
            if let Some(pipeline) = pipeline_cache.get_compute_pipeline(solv_id) {
                for _ in 0..4 {
                    let mut pass = render_context.command_encoder().begin_compute_pass(&ComputePassDescriptor {
                        label: Some("physics_solver_pass"),
                        ..default()
                    });
                    pass.set_pipeline(pipeline);
                    pass.set_bind_group(0, &bind_group, &[]);
                    pass.dispatch_workgroups(dispatch_constraints, 1, 1);
                }
            }
        }

        Ok(())
    }
}