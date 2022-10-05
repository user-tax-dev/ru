use neon::prelude::*;

pub fn init(cx: &mut ModuleContext) -> NeonResult<()> {
  ${init}
  Ok(())
}

#[cfg(feature = "main")]
#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
  crate::init(&mut cx)?;
  Ok(())
}
