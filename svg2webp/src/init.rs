use neon::prelude::*;

pub fn init(cx: &mut ModuleContext) -> NeonResult<()> {
  cx.export_function("svgWebp", crate::svg_webp)?;
  Ok(())
}

#[cfg(feature = "main")]
#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
  crate::init(&mut cx)?;
  Ok(())
}
