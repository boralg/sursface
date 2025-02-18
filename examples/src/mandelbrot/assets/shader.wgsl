struct Uniforms {
    translation: vec2<f32>,
    cursor_pos: vec2<f32>,
    scale: f32,
    aspect_ratio: f32,
    _padding: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) fragUV: vec2<f32>,
};

@vertex
fn vs_main(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(position, 1.0);
    output.fragUV = uv;
    return output;
}

@fragment
fn fs_main(@location(0) fragUV: vec2<f32>) -> @location(0) vec4<f32> {
    let max_iter: u32 = 200u;
    
    let centered_uv = (fragUV * 2.0 - vec2<f32>(1.0, 1.0)) * vec2<f32>(uniforms.aspect_ratio, 1.0);
    let c = uniforms.translation + centered_uv * uniforms.scale;

    var z = vec2<f32>(0.0, 0.0);
    var iter: u32 = 0u;

    for (var i: u32 = 0u; i < max_iter; i++) {
        if (dot(z, z) > 4.0) { break; }
        z = vec2<f32>(
            z.x * z.x - z.y * z.y + c.x,
            2.0 * z.x * z.y + c.y
        );
        iter++;
    }

    if (iter == max_iter) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else {
        let modulus_squared = dot(z, z);
        let log_zn = 0.5 * log2(modulus_squared);
        let nu = log2(log_zn);
        let smooth_iter = f32(iter) + 1.0 - nu;
        let t = smooth_iter / f32(max_iter);

        let speed = 2.0;
        let brightness = 0.7;
        let r = brightness * sin(t * speed * 6.2831);
        let g = brightness * sin(t * speed * 6.2831 + 2.094);
        let b = brightness * sin(t * speed * 6.2831 + 4.188);
        
        return vec4<f32>(
            0.5 + r * 0.5,
            0.5 + g * 0.5,
            0.5 + b * 0.5,
            1.0
        );
    }
}