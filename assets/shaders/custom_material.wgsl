#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_functions

@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
};

let PHIMINUS1: f32 = 0.61803398875;
let TAU: f32 = 6.2831853071795864769252867665590;

fn sphericalFibonacci(i: f32, n: f32) -> vec3<f32> {
    let phi = TAU * fract(i * PHIMINUS1);
    let cosTheta = 1.0 - (2.0 * i + 1.0) * (1.0 / n);
    let sinTheta = sqrt(saturate(1.0 - cosTheta * cosTheta));

    return vec3(
        cos(phi) * sinTheta,
        sin(phi) * sinTheta,
        cosTheta);
}

fn rand(co: f32) -> f32 { 
    return fract(sin(co*(91.3458)) * 47453.5453); 
}

fn ssao(radius: f32, bias: f32, frag_view: vec3<f32>, frag_coord: vec2<f32>, normal_view: vec3<f32>) -> f32 {
    let frame_size = view.viewport.zw;
    let view_uv = frag_coord / frame_size;

    let kernel_size = 64.0;
    let double_kernel_size = kernel_size * 2.0;

    var occlusion: f32 = 0.0;

    let i_kernel_size = i32(kernel_size);

    let randomVec = vec3(rand(frag_coord.x), rand(frag_coord.y), rand(frag_coord.x*frag_coord.y));
    let tangent = normalize(randomVec - normal_view * dot(randomVec, normal_view));
    let bitangent = cross(normal_view, tangent);
    let TBN = mat3x3<f32>(tangent, bitangent, normal_view);

    for (var i = 0; i < i_kernel_size; i = i + 1) {
        let sample_offset_view = TBN * sphericalFibonacci(f32(i), double_kernel_size); // from tangent to view space
        let sample_view = vec4<f32>(frag_view.xyz + sample_offset_view * radius, 1.0);

        let sample_clip = view.projection * sample_view; // from view to clip space
        let sample_ndc = sample_clip.xyz / sample_clip.w; // perspective divide
        // sample_ndc.x is [-1,1] left to right, so * 0.5 + 0.5 remaps to [0,1] left to right
        // sample_ndc.y is [-1,1] bottom to top, so * -0.5 + 0.5 remaps to [0,1] top to bottom
        let depth_uv = vec2<f32>(sample_ndc.x * 0.5 + 0.5, sample_ndc.y * -0.5 + 0.5);

        let depth = -textureLoad(texture, vec2<i32>(depth_uv * frame_size), 0).w;

        let range_check = smoothstep(0.0, 1.0, radius / distance(frag_view.z, depth));
        if (depth >= sample_view.z + bias) {
            occlusion += range_check;
        }
    }

    let occ = 1.0 - (occlusion / kernel_size);

    return occ;
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var V = normalize(view.world_position.xyz - in.world_position.xyz);
    let normal_depth = textureLoad(texture, vec2<i32>(in.frag_coord.xy), 0);

    let frag_view_homogeneous = view.inverse_view * vec4<f32>(in.world_position.xyz, 1.0);
    var frag_view = frag_view_homogeneous.xyz / frag_view_homogeneous.w;
    let normal_view = (view.inverse_view * vec4(normal_depth.xyz, 0.0)).xyz;
    var ssao_v = ssao(0.25, 0.025, frag_view, in.frag_coord.xy, normal_view);

    return vec4(vec3(dot(V, normal_view) * 0.5 * ssao_v*ssao_v*ssao_v), 1.0);
}
