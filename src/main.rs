use crate::render_image::Frame;

mod make_ppis;
mod render_image;

#[pollster::main]
async fn main() {
    // make_ppis::make_ppis("tminus.mp4").unwrap();
    dbg!(std::process::id());
    let frame_recv = render_image::render_images().await;

    let frames = frame_recv.try_iter().collect::<Vec<_>>();
    let len = frames.last().unwrap().2;
    let (mut total_load_image, mut total_send_texture, mut total_render) = (0, 0, 0);
    for frame in frames {
        match frame.0 {
            Frame::LoadImage => total_load_image += frame.2 - frame.1,
            Frame::SendTexture => total_send_texture += frame.2 - frame.1,
            Frame::Render => total_render += frame.2 - frame.1,
        }
    }

    println!("Total load image: {} ms ({}%)", total_load_image as f64 / 1000.0, total_load_image as f64 / len as f64 * 100.0);
    println!("Total send texture: {} ms ({}%)", total_send_texture as f64 / 1000.0, total_send_texture as f64 / len as f64 * 100.0);
    println!("Total render: {} ms ({}%)", total_render as f64 / 1000.0, total_render as f64 / len as f64 * 100.0);
}
