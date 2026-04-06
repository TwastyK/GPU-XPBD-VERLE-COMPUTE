struct Particle {
    pos: vec4<f32>,
    old_pos: vec4<f32>,
    force: vec4<f32>,
    inv_mass: vec4<f32>,
};

struct Constraint {
   node_a: u32,
   node_b: u32,
   rest_length: f32,
   stiffness: f32,
};

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<storage, read_write> constraints: array<Constraint>;

const DT: f32 = 0.01666;
const GRAVITY: vec3<f32> = vec3<f32>(0.0, -9.8, 0.0);

@compute @workgroup_size(64)
fn integrate(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    let num_particles = arrayLength(&particles);
    if (idx >= num_particles) { return; }

    var p = particles[idx];
    let movable_mask = step(0.0001, p.inv_mass.x);

    let temp_pos = p.pos.xyz;
    let velocity = (p.pos.xyz - p.old_pos.xyz);

    // Интеграция Верле
    let new_pos = p.pos.xyz + velocity + (GRAVITY + p.force.xyz) * DT * DT * movable_mask;

    particles[idx].old_pos = vec4<f32>(temp_pos, 0.0);
    particles[idx].pos = vec4<f32>(new_pos, 0.0);
    particles[idx].force = vec4<f32>(0.0);
}

@compute @workgroup_size(64)
fn solve_constraints(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    let num_constraints = arrayLength(&constraints);
    if (idx >= num_constraints) { return; }

    let c = constraints[idx];
    let p1 = particles[c.node_a].pos.xyz;
    let p2 = particles[c.node_b].pos.xyz;
    let w1 = particles[c.node_a].inv_mass.x;
    let w2 = particles[c.node_b].inv_mass.x;

    let w_sum = w1 + w2;
    if (w_sum <= 0.0) { return; }

    let dir = p1 - p2;
    let dist = length(dir);
    let safe_dist = max(dist, 0.0001);
    let dir_norm = dir / safe_dist;

    let constraint_error = dist - c.rest_length;
    let alpha = c.stiffness / (DT * DT);
    let lambda = -constraint_error / (w_sum + alpha);
    let correction = dir_norm * lambda;

    // Применяем коррекцию напрямую к позициям
    particles[c.node_a].pos += vec4<f32>(correction * w1, 0.0);
    particles[c.node_b].pos -= vec4<f32>(correction * w2, 0.0);
}