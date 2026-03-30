struct Particle {
    pos: vec4<f32>,
    old_pos: vec4<f32>,
    force: vec4<f32>,
    inv_mass: vec4<f32>,
}

struct Constraint {
   node_a: u32,
   node_b: u32,
   rest_length: f32,
   stiffness: f32,
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<storage, read_write> constraints: array<Constraint>;


const DT: f32 = 0.01666;
const GRAVITY: vec3<f32> = vec3<f32>(0.0, -9.8, 0.0);

@compute @workgroup_size(64)

fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;

    let num_particles = arrayLength(&particles);
    if (idx >= num_particles) { return; }

    let p = particles[idx];

    let current_pos = p.pos.xyz;
    let velocity = current_pos - p.old_pos.xyz;

    let accel = (GRAVITY + p.force.xyz) * p.inv_mass.x;
    var next_pos = current_pos + velocity + accel * (DT * DT);
    next_pos.y = max(next_pos.y, 0.0);

    let is_above_floor = step(0.001, next_pos.y);
    let adjusted_old_pos = mix(next_pos, current_pos, is_above_floor);

    particles[idx].pos = vec4<f32>(next_pos, p.pos.w);
    particles[idx].old_pos = vec4<f32>(adjusted_old_pos, p.old_pos.w);
}