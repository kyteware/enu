@vertex
fn main(@location(0) clip: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(clip, 1.0);
}