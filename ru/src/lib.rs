use neon::prelude::*;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
  misc::init(&mut cx)?;
  redis::init(&mut cx)?;
  svg2webp::init(&mut cx)?;
  Ok(())
}
