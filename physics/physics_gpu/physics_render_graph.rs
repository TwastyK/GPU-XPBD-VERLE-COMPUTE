// render_graph
use bevy::render::render_graph::{RenderGraph, RenderLabel};
use crate::physics_node::PhysicsNode;

#[derive(RenderLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum Labels {
    Physics,
}

// Принимаем &mut RenderGraph вместо ResMut
pub fn configure_graph(render_graph: &mut RenderGraph) {
    render_graph.add_node(Labels::Physics, PhysicsNode);
    render_graph.add_node_edge(Labels::Physics, bevy::render::graph::CameraDriverLabel);
}

