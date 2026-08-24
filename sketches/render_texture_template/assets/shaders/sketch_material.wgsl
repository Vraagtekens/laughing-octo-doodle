#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct SketchParams {
    color_a: vec4<f32>,
    color_b: vec4<f32>,
    time: f32,
    index: f32,
    _padding: vec2<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: SketchParams;

fn hash(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    let centered = uv * 2.0 - 1.0;
    let radius = length(centered);
    let angle = atan2(centered.y, centered.x);
    let motif = material.index - floor(material.index / 7.0) * 7.0;

    let rings = sin(radius * 20.0 - material.time * 3.0 + material.index);
    let spokes = sin(angle * (3.0 + motif) + material.time * 1.7);
    let grain = hash(floor((uv + material.time * 0.015) * 90.0));
    let mask = smoothstep(1.0, 0.22, radius);
    let pulse = 0.5 + 0.5 * sin(material.time * 1.6 + material.index);

    let mix_value = clamp(0.5 + 0.32 * rings + 0.18 * spokes + 0.12 * grain, 0.0, 1.0);
    var color = mix(material.color_a, material.color_b, mix_value);
    color = vec4<f32>(color.rgb * (0.55 + pulse * 0.85), color.a);
    color.a = mask * smoothstep(1.08, 0.72, radius);
    return color;
}
