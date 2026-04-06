use crate::structures::{Particle, DistanceConstraints};
use gltf::import;
use bevy::math::Vec3;
use bevy::prelude::info;

pub struct NpcTemplate {
    pub particles: Vec<Particle>,
    pub constraints: Vec<DistanceConstraints>,
    pub index_count: u32,
}

pub fn parse_model(path: &str) -> NpcTemplate {
    info!("Loading GLTF model from: {}", path);
    let (document, buffers, _) = import(path).expect("Failed to load gltf");

    let mut particles = Vec::new();
    let mut constraints = Vec::new();
    let mut index_count = 0u32;

    // --- 1. СЧИТАЕМ ИНДЕКСЫ МОДЕЛИ ---
    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            if let Some(indices) = reader.read_indices() {
                index_count += indices.into_u32().count() as u32;
            }
        }
    }

    // --- 2. ПАРСИМ СКЕЛЕТ ---
    if let Some(skin) = document.skins().next() {
        let joints: Vec<_> = skin.joints().collect();
        info!("Found skin with {} joints", joints.len());

        for joint in &joints {
            let (pos, _, _) = joint.transform().decomposed();
            particles.push(Particle {
                pos: [pos[0], pos[1], pos[2], 1.0],
                old_pos: [pos[0], pos[1], pos[2], 1.0],
                force: [0.0; 4],
                inv_mass: [1.0, 0.0, 0.0, 0.0],
            });
        }

        if !particles.is_empty() {
            particles[0].inv_mass[0] = 1.0; // Корневая точка
        }

        for (idx, joint) in joints.iter().enumerate() {
            for child in joint.children() {
                if let Some(child_idx) = joints.iter().position(|j| j.index() == child.index()) {
                    let p1 = Vec3::from_slice(&particles[idx].pos[0..3]);
                    let p2 = Vec3::from_slice(&particles[child_idx].pos[0..3]);

                    constraints.push(DistanceConstraints {
                        node_a: idx as u32,
                        node_b: child_idx as u32,
                        rest_length: p1.distance(p2),
                        stiffness: 1.0,
                    });
                }
            }
        }
    }

    info!("Parse complete: {} particles, {} constraints, {} indices",
          particles.len(), constraints.len(), index_count);

    NpcTemplate {
        particles,
        constraints,
        index_count,
    }
}