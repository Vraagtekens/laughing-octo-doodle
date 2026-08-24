#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct SlimeParams {
    resolution: vec2<f32>,
    phase: f32,
    density: f32,
    filaments: f32,
    glow: f32,
    thickness: f32,
    reach: f32,
    contrast: f32,
    _padding: vec2<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: SlimeParams;

const TAU: f32 = 6.28318530718;
const COLUMNS: u32 = 7u;
const ROWS: u32 = 6u;

fn hash(value: f32) -> f32 {
    return fract(sin(value * 127.1) * 43758.5453);
}

fn cell_position(column: u32, row: u32, theta: f32) -> vec2<f32> {
    let index = f32(row * COLUMNS + column);
    let grid_size = vec2<f32>(f32(COLUMNS), f32(ROWS));
    let jitter = vec2<f32>(hash(index + 11.0), hash(index + 37.0)) - 0.5;
    let base = (vec2<f32>(f32(column), f32(row)) + 0.5 + jitter * 0.64) / grid_size;

    let frequency_x = 1.0 + f32((column + row) % 3u);
    let frequency_y = 1.0 + f32((column + row * 2u) % 2u);
    let movement = vec2<f32>(
        sin(theta * frequency_x + hash(index + 71.0) * TAU),
        cos(theta * frequency_y + hash(index + 103.0) * TAU)
    ) * (0.007 + material.reach * 0.012);

    return fract(base + movement + 1.0);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = material.resolution.x / material.resolution.y;
    let theta = material.phase * TAU;
    let irregularity = clamp(material.density, 0.35, 2.0);

    let warp_frequency = 1.4 + material.filaments * 0.08;
    let warp = vec2<f32>(
        sin(mesh.uv.y * TAU * warp_frequency + theta * 2.0)
            + sin((mesh.uv.x + mesh.uv.y) * TAU * 3.7 - theta) * 0.42,
        sin(mesh.uv.x * TAU * (warp_frequency * 0.87) - theta)
            + cos((mesh.uv.x - mesh.uv.y) * TAU * 3.1 + theta * 2.0) * 0.38
    ) * (0.010 + material.reach * 0.017);
    let point = fract(mesh.uv + warp + 1.0);

    var nearest = 100.0;
    var second_nearest = 100.0;

    for (var row: u32 = 0u; row < ROWS; row += 1u) {
        for (var column: u32 = 0u; column < COLUMNS; column += 1u) {
            let index = f32(row * COLUMNS + column);
            let cell = cell_position(column, row, theta);
            var difference = point - cell;
            difference -= round(difference);
            difference.x *= aspect;

            let organic_offset = vec2<f32>(
                sin(difference.y * 24.0 + hash(index + 181.0) * TAU),
                cos(difference.x * 21.0 + hash(index + 211.0) * TAU)
            ) * (0.005 + irregularity * 0.004);
            let weight = mix(1.10, 0.84, hash(index + 149.0) * irregularity * 0.5);
            let distance = length(difference + organic_offset) * weight;

            if (distance < nearest) {
                second_nearest = nearest;
                nearest = distance;
            } else if (distance < second_nearest) {
                second_nearest = distance;
            }
        }
    }

    let wall_variation = 0.78 + 0.22 * sin(
        mesh.uv.x * 53.0 + mesh.uv.y * 41.0 + sin(mesh.uv.y * 19.0 + theta) * 2.0
    );
    let boundary_distance = (second_nearest - nearest) / wall_variation;
    let line_width = 0.005 + material.thickness * 0.082;
    let membrane = 1.0 - smoothstep(line_width * 0.16, line_width, boundary_distance);
    let diffusion = 1.0 - smoothstep(line_width * 0.8, line_width * (2.5 + material.glow), boundary_distance);

    let large_texture = 0.70 + 0.30 * sin(
        mesh.uv.x * 37.0
        + mesh.uv.y * 29.0
        + sin(mesh.uv.y * 17.0 - theta * 2.0) * 1.8
    );
    let grain = hash(
        floor(mesh.uv.x * material.resolution.x * 0.5)
        + floor(mesh.uv.y * material.resolution.y * 0.5) * material.resolution.x
    ) - 0.5;

    var intensity = membrane * large_texture;
    intensity += diffusion * (0.08 + material.glow * 0.13);
    intensity += grain * membrane * 0.18;
    intensity = clamp(
        (intensity - 0.30) * material.contrast + 0.24,
        0.0,
        1.0
    );

    return vec4<f32>(vec3<f32>(intensity), 1.0);
}
