pub mod structures;
pub mod RenderGraph;
pub mod PhysicsNode;
pub mod PhysicsPipeline;
pub mod physics_asset;
pub mod gltf_parser;

use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::render::render_resource::SpecializedComputePipelines;

// Используем alias, чтобы избежать путаницы с именами модулей
use crate::PhysicsPipeline::PhysicsPipeline as PhysicsPipelineStruct;
use crate::physics_asset::{setup_physics_assets, extract_physics_buffers};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_physics_assets);

    // RenderApp здесь используется как ЗНАЧЕНИЕ (AppLabel)
    if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app
            .init_resource::<PhysicsPipelineStruct>()
            .init_resource::<SpecializedComputePipelines<PhysicsPipelineStruct>>()
            .add_systems(ExtractSchedule, extract_physics_buffers);

        // Используем прямой вызов функции из модуля
        RenderGraph::configure_graph(render_app.world_mut());
    }

    app.run();
}