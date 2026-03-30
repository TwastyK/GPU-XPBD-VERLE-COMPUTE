use std::borrow::Cow;
use bevy::prelude::*;
use bevy::render::render_resource::*;
use crate::PhysicsNode::SpecializedPhysicsPipelineId;

#[derive(Resource)]
pub struct PhysicsPipeline {
    pub shader: Handle<Shader>,
    pub layout: BindGroupLayout, // Этот объект нужен для PhysicsNode.rs
}

pub fn queue_physics_pipeline(
    mut pipeline_id: ResMut<SpecializedPhysicsPipelineId>,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<PhysicsPipeline>>,
    pipeline: Res<PhysicsPipeline>,
) {
    // Здесь мы задаем ключ (например, всегда с гравитацией для теста)
    let key = PhysicsPipelineKey { use_gravity: true };

    // Запрашиваем специализацию пайплайна
    pipeline_id.0 = Some(pipelines.specialize(&pipeline_cache, &pipeline, key));
}
impl FromWorld for PhysicsPipeline {
    fn from_world(render_world: &mut World) -> Self {
        let asset_server = render_world.resource::<AssetServer>();
        let render_device = render_world.resource::<bevy::render::renderer::RenderDevice>();

        let shader = asset_server.load("Physics.wgsl");

        // Создаем реальный объект BindGroupLayout для использования в BindGroup
        let layout = render_device.create_bind_group_layout(
            "Physics Bind Group Layout",
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        );

        Self { shader, layout }
    }
}

impl SpecializedComputePipeline for PhysicsPipeline {
    type Key = PhysicsPipelineKey;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some(Cow::Borrowed("Physics Pipeline")),
            // ИСПРАВЛЕНИЕ: Здесь должен быть Vec<BindGroupLayoutDescriptor>, а не BindGroupLayout
            layout: vec![BindGroupLayoutDescriptor {
                label: Cow::Borrowed("Physics Bind Group Layout"),
                entries: vec![
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            }],
            push_constant_ranges: vec![],
            shader: self.shader.clone(),
            shader_defs: if key.use_gravity {
                vec!["USE_GRAVITY".into()]
            } else {
                vec![]
            },
            entry_point: Some(Cow::Borrowed("main")),
            zero_initialize_workgroup_memory: true,
        }
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct PhysicsPipelineKey {
    pub use_gravity: bool,
}