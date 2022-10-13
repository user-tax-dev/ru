use neon::prelude::*;

pub fn init(cx: &mut ModuleContext) -> NeonResult<()> {
  cx.export_function("b64", crate::b64)?;
  cx.export_function("unb64", crate::unb64)?;
  cx.export_function("passwordHash", crate::password_hash)?;
  cx.export_function("z85Load", crate::z85_load)?;
  cx.export_function("z85Dump", crate::z85_dump)?;
  cx.export_function("randomBytes", crate::random_bytes)?;
  cx.export_function("xxh3B36", crate::xxh3_b36)?;
  cx.export_function("ipBin", crate::ip_bin)?;
  cx.export_function("tld", crate::tld)?;
  Ok(())
}

#[cfg(feature = "main")]
#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
  crate::init(&mut cx)?;
  Ok(())
}
