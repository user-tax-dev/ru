mod init;
use image::EncodableLayout;
pub use init::init;
use nlib::*;
use thiserror::Error;
use tiny_skia::PremultipliedColorU8;
use webp::Encoder;

#[derive(Error, Debug)]
pub enum Error {
  #[error("tiny_skia::Pixmap::new return None")]
  PIXMAP,

  #[error("resvg::render return None")]
  RESVG,
}

fn _svg_webp(svg: impl AsRef<[u8]>, quality: f32) -> anyhow::Result<Box<[u8]>> {
  let opt = usvg::Options::default();
  let rtree = usvg::Tree::from_data(svg.as_ref(), &opt.to_ref())?;
  let pixmap_size = rtree.svg_node().size.to_screen_size();
  let width = pixmap_size.width();
  let height = pixmap_size.height();
  if let Some(mut pixmap) = tiny_skia::Pixmap::new(width, height) {
    // 去除透明度（默认是黑底，255-颜色会改为用白底）
    for px in pixmap.pixels_mut() {
      *px = PremultipliedColorU8::from_rgba(255 - px.red(), 255 - px.green(), 255 - px.blue(), 255)
        .unwrap();
    }
    if resvg::render(
      &rtree,
      usvg::FitTo::Original,
      tiny_skia::Transform::default(),
      pixmap.as_mut(),
    )
    .is_some()
    {
      let img = pixmap.data();

      let encoder = Encoder::from_rgba(img, width, height);
      let encoded_webp = encoder.encode(quality);
      return Ok(Box::from(encoded_webp.as_bytes()));
    } else {
      return Err(Error::RESVG)?;
    }
  }
  Err(Error::PIXMAP)?
}

js_fn! {
svg_webp |cx| {
  let svg = to_bin(cx, 0)?;
  let quality = cx.argument::<JsNumber>(1)?.value(cx) as f32;
  jswait(cx, async move { _svg_webp(svg, quality) })?
}
}
