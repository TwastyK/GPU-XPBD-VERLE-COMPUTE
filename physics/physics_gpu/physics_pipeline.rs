use std::borrow::Cow;
use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;

#[derive(Resource)]
pub struct PhysicsPipeline {
    pub shader: Handle<Shader>,
    pub layout: BindGroupLayout,
    // Сохраняем описание входов для использования в specialize
    pub layout_entries: Vec<BindGroupLayoutEntry>,
}

// Ключ специализации определяет, какую функцию (entry_point) вызвать в шейдере
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct PhysicsPipelineKey {
    pub entry_point: &'static str,
}

impl PhysicsPipeline {
    pub fn new(render_device: &RenderDevice, shader: Handle<Shader>) -> Self {
        // 1. Описываем структуру привязок (Bind Group Layout)
        let entries = vec![
            // Binding 0: Частицы (Read-Write)
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
            // Binding 1: Связи/Constraints (Read-Write)
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
        ];

        // 2. Создаем сам объект Layout в GPU
        let layout = render_device.create_bind_group_layout(
            "Physics Bind Group Layout",
            &entries,
        );

        Self {
            shader,
            layout,
            layout_entries: entries,
        }
    }
}

impl SpecializedComputePipeline for PhysicsPipeline {
    type Key = PhysicsPipelineKey;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some(Cow::from(format!("Physics_{}", key.entry_point))),
            // В 0.18.1 передаем дескриптор, созданный из сохраненных записей
            layout: vec![BindGroupLayoutDescriptor {
                label: Cow::Borrowed("Physics Pipeline Layout"),
                entries: self.layout_entries.clone(),
            }],
            push_constant_ranges: vec![],
            shader: self.shader.clone(),
            shader_defs: vec![],
            // Выбираем точку входа на основе ключа
            entry_point: Some(Cow::Borrowed(key.entry_point)),
            zero_initialize_workgroup_memory: true,
        }
    }
}

// Структура для хранения ID готовых пайплайнов для PhysicsNode
#[derive(Resource, Default)]
pub struct SpecializedPhysicsPipelineIds {
    pub integration: Option<CachedComputePipelineId>,
    pub solver: Option<CachedComputePipelineId>,
}

// Система для регистрации пайплайнов в кэше
pub fn queue_physics_pipeline(
    mut ids: ResMut<SpecializedPhysicsPipelineIds>,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<PhysicsPipeline>>,
    pipeline: Res<PhysicsPipeline>,
) {
    // Запрашиваем компиляцию для стадии интеграции
    ids.integration = Some(pipelines.specialize(
        &pipeline_cache,
        &pipeline,
        PhysicsPipelineKey { entry_point: "integrate" },
    ));

    // Запрашиваем компиляцию для стадии солвера
    ids.solver = Some(pipelines.specialize(
        &pipeline_cache,
        &pipeline,
        PhysicsPipelineKey { entry_point: "solve_constraints" },
    ));
}