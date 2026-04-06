#import bevy_pbr::mesh_view_bindings::view

struct Particle {
    pos: vec4<f32>,
    old_pos: vec4<f32>,
    force: vec4<f32>,
    inv_mass: vec4<f32>,
};

@group(2) @binding(0) var<storage, read> particles: array<Particle>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

// rendering/draw.wgsl

@vertex
fn vertex(input: VertexInput, @builtin(instance_index) instance_idx: u32) -> VertexOutput {
    // 1. Читаем данные
    let node_per_npc = 19u;
    let particle = particles[instance_idx * node_per_npc];
    let particle_pos = particle.pos.xyz;

    // 2. Считаем позицию
    // ВАЖНО: Если вы спавните сущности через commands.spawn,
    // у них есть свой Transform. Если он в (0,0,0), то world_pos будет верным.
    let world_pos = input.position + particle_pos;

    // Добавь искусственное смещение, чтобы увидеть толпу
    let x_spacing = f32(instance_idx % 223u) * 3.0;
    let z_spacing = f32(instance_idx / 223u) * 3.0;

    let final_pos = input.position + particle_pos + vec3<f32>(x_spacing, 0.0, z_spacing);
    
    var out: VertexOutput;
    out.clip_position = view.clip_from_world * vec4<f32>(world_pos, 1.0);

    // 3. ДИАГНОСТИКА: Если вы видите один цвет — значит instance_idx всегда 0.
    // Если цвета разные — значит индексы работают, но позиции в буфере одинаковые.
    let r = f32(instance_idx % 255u) / 255.0;
    let g = f32((instance_idx / 255u) % 255u) / 255.0;
    out.color = vec4<f32>(r, g, 1.0, 1.0);

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}