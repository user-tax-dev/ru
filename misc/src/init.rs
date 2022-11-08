use neon::prelude::*;

pub fn init(cx: &mut ModuleContext) -> NeonResult<()> {
  cx.export_function("zipU64", crate::zip_u64)?;
  cx.export_function("unzipU64", crate::unzip_u64)?;
  cx.export_function("unb64", crate::unb64)?;
  cx.export_function("b64", crate::b64)?;
  cx.export_function("b64U64", crate::b64_u64)?;
  cx.export_function("u64B64", crate::u64_b64)?;
  cx.export_function("u64Bin", crate::u64_bin)?;
  cx.export_function("binU64", crate::bin_u64)?;
  cx.export_function("passwordHash", crate::password_hash)?;
  cx.export_function("z85Load", crate::z85_load)?;
  cx.export_function("z85Dump", crate::z85_dump)?;
  cx.export_function("randomBytes", crate::random_bytes)?;
  cx.export_function("cookieEncode", crate::cookie_encode)?;
  cx.export_function("cookieDecode", crate::cookie_decode)?;
  cx.export_function("xxh64", crate::xxh64)?;
  cx.export_function("xxh32", crate::xxh32)?;
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
