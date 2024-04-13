@group(0)
@binding(0)
var<storage, read> alpha: f32;

@fragment
fn main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, alpha, 0.0);
}