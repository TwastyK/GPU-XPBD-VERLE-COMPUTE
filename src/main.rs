// main.rs
#![allow(unused_variables)]
#![allow(unused_imports)]
pub mod structures;
pub mod physics_render_graph;
pub mod physics_node;
pub mod physics_pipeline;
pub mod physics_asset; // Можно удалить этот файл позже, если всё перенесли в npc_render
pub mod gltf_parser;
pub mod rendering;
pub mod camera;

use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::render::{RenderApp, Render, ExtractSchedule, RenderSystems};
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::render::render_phase::AddRenderCommand;

// Правильные импорты
use crate::camera::{fly_camera_system, setup_camera};
use crate::physics_pipeline::{PhysicsPipeline as PhysicsPipelineStruct, queue_physics_pipeline, SpecializedPhysicsPipelineIds};
// Берем всё из npc_render, так как там самая свежая версия логики
use crate::rendering::npc_render::{
    NpcMaterial,
    DrawNpcPipeline,
    setup_npc_rendering,
    setup_physics_assets,
    extract_physics_buffers
};
use crate::physics_render_graph::configure_graph;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: bevy::window::PresentMode::Immediate,
                title: "Origin Engine: 100k NPC Battle Simulation".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())

        // 1. Регистрация материала (обязательно для Render World)
        .add_plugins(MaterialPlugin::<NpcMaterial>::default())

        .add_plugins(PhysicsPlugin)

        // 2. Системы инициализации
        // setup_physics_assets создаст буферы, setup_npc_rendering заспавнит "триггер" отрисовки
        .add_systems(Startup, (
            setup_physics_assets,
            setup_npc_rendering.after(setup_physics_assets), // Ждем создания буферов
            setup_camera
        ))
        .add_systems(Update, fly_camera_system)
        .run();
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, _app: &mut App) {
        // Логика Startup вынесена в основной App выше
    }

    fn finish(&self, app: &mut App) {
        let shader_source = include_str!("../physics/wgsl_shaders/physics.wgsl");

        let shader_handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<Shader>>();
            assets.add(Shader::from_wgsl(shader_source, "shaders/physics.wgsl"))
        };

        let render_app = app.sub_app_mut(RenderApp);
        let render_device = render_app.world().resource::<RenderDevice>().clone();
        let pipeline = PhysicsPipelineStruct::new(&render_device, shader_handle);

        render_app
            .insert_resource(pipeline)
            .init_resource::<SpecializedPhysicsPipelineIds>()
            .init_resource::<SpecializedComputePipelines<PhysicsPipelineStruct>>()
            // 3. Передача данных из Main World в Render World
            .add_systems(ExtractSchedule, extract_physics_buffers)
            // 4. Подготовка пайплайна
            .add_systems(Render, queue_physics_pipeline.in_set(RenderSystems::Queue))
            // 5. РЕГИСТРАЦИЯ КОМАНДЫ INDIRECT DRAW
            // Без этой строчки Bevy не поймет, как выполнять DrawNpcPipeline
            .add_render_command::<Opaque3d, DrawNpcPipeline>();

        let mut render_graph = render_app.world_mut().resource_mut::<bevy::render::render_graph::RenderGraph>();
        configure_graph(&mut render_graph);
    }
}