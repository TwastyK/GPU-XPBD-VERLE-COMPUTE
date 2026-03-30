use crate::structures::{Particle, DistanceConstraints}; // Твои структуры
use gltf::import;
use bevy::math::Vec3;

pub struct NpcTemplate {
    pub particles: Vec<Particle>,
    pub constraints: Vec<DistanceConstraints>,
}

pub fn parse_model(path: &str) -> NpcTemplate {
    // Используем import, чтобы сразу получить буферы (бинарные данные)
    let (document, buffers, _) = import(path).expect("Failed to load gltf");

    let mut particles = Vec::new();
    let mut constraints = Vec::new();

    // Берем первый скин (скелет) из файла
    if let Some(skin) = document.skins().next() {
        // 1. Создаем частицы из костей
        for joint in skin.joints() {
            let (pos, _, _) = joint.transform().decomposed();
            particles.push(Particle {
                pos: [pos[0], pos[1], pos[2], 1.0],
                old_pos: [pos[0], pos[1], pos[2], 1.0],
                force: [0.0; 4],
                inv_mass: [1.0, 0.0, 0.0, 0.0],
            });
        }

        // 2. Создаем связи (Constraints)
        // БЕЗ ЭТОГО ШЕЙДЕР НЕ ПОЙМЕТ, ЧТО КОСТИ СОЕДИНЕНЫ
        for (idx, joint) in skin.joints().enumerate() {
            for child in joint.children() {
                if let Some(child_idx) = skin.joints().position(|j| j.index() == child.index()) {
                    let p1 = Vec3::from_slice(&particles[idx].pos[0..3]);
                    let p2 = Vec3::from_slice(&particles[child_idx].pos[0..3]);

                    constraints.push(DistanceConstraints {
                        node_a: idx as u32,
                        node_b: child_idx as u32,
                        rest_length: p1.distance(p2), // Важно для шейдера!
                        stiffness: 1.0,
                    });
                }
            }
        }
    }

    NpcTemplate { particles, constraints }
}