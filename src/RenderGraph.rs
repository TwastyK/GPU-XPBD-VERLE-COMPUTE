use bevy::render::render_graph::{RenderGraph, RenderLabel};
use bevy::prelude::*;
use crate::PhysicsNode::PhysicsNode;

#[derive(RenderLabel, Debug, Hash, PartialEq, Eq, Clone)]
enum Labels {
    Physics,
}

pub fn configure_graph(mut render_graph: ResMut<RenderGraph>) {
    render_graph.add_node(Labels::Physics, PhysicsNode);
    render_graph.add_node_edge(Labels::Physics, bevy::render::graph::CameraDriverLabel)

}

