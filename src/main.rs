mod make_ppis;
mod render_image;

#[pollster::main]
async fn main() {
    // make_ppis::make_ppis("tminus.mp4").unwrap();

    render_image::render_images().await;
}
