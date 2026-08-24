#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct PostParams {
    resolution: vec2<f32>,
    time: f32,
    feedback_mix: f32,
    vignette: f32,
    chroma: f32,
    _padding: vec2<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var source_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var source_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var<uniform> material: PostParams;

fn sample_scene(uv: vec2<f32>) -> vec3<f32> {
    return textureSample(source_texture, source_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0))).rgb;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    let centered = uv * 2.0 - 1.0;
    let radius = length(centered);
    let direction = centered / max(radius, 0.0001);

    let wave = sin((uv.y + material.time * 0.08) * 42.0) * 0.0025;
    let chroma_offset = direction * material.chroma * 0.0025 + vec2<f32>(wave, 0.0);

    let r = sample_scene(uv + chroma_offset).r;
    let g = sample_scene(uv + vec2<f32>(wave * 0.4, -wave * 0.2)).g;
    let b = sample_scene(uv - chroma_offset).b;
    var color = vec3<f32>(r, g, b);

    let echo = sample_scene(0.5 + centered * (0.965 + 0.012 * sin(material.time)));
    color = mix(color, echo, material.feedback_mix * 0.18);

    let vignette = smoothstep(1.32, material.vignette, radius);
    let scanline = 0.94 + 0.06 * sin(uv.y * material.resolution.y * 1.35);
    color *= vignette * scanline;
    color = pow(color, vec3<f32>(0.92));

    return vec4<f32>(color, 1.0);
}
