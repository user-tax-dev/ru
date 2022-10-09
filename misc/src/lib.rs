mod init;
use std::{
  hash::{BuildHasher, Hasher},
  net::IpAddr,
};

pub use init::init;
use nlib::*;
use xxhash_rust::xxh3::Xxh3Builder;

const XXHASHER: Xxh3Builder = Xxh3Builder::new();

js_fn! {

password_hash |cx| {
  let mut hasher = blake3::Hasher::new();
  for i in 0..cx.len() {
    let bin = to_bin(cx, i)?;
    hasher.update(&bin);
  }
  await_bin(cx, async move {
    let mut output = [0; 2048];
    for _ in 1..2048 {
      hasher.finalize_xof().fill(&mut output);
      hasher.update(&output);
    }
    Ok(Box::from(&hasher.finalize().as_bytes()[..]))
  })
}

u64_bin |cx| {
  let x = as_f64(cx, 0)? as u64;
  js_bin(cx, &x.to_le_bytes())
}

bin_u64 |cx| {
  let x = as_bin(cx, 0)?;
  if x.len() == 8 {
    let x = u64::from_le_bytes(x.try_into().unwrap()) as f64;
    js_f64(cx, x)
  } else {
    js_undefined(cx)
  }
}

z85_load |cx| {
  let s = to_bin(cx,0)?;
  if let Ok(r) = z85::decode(s){
    js_bin(cx, r)
  }else {
    js_undefined(cx)
  }
}

z85_dump |cx| {
  let bin = as_bin(cx,0)?;
  let r = z85::encode(bin);
  js_str(cx,r)
}

random_bytes |cx| {
  let n = as_f64(cx,0)? as usize;
  js_bin(cx,(0..n).map(
    |_| rand::random::<u8>()
  ).collect::<Vec<u8>>())
}

xxh3_b36 |cx| {
 let li = args_bin_li(cx,0)?;
 let mut h64 = XXHASHER.build_hasher();
 for i in li {
   h64.update(i.as_ref());
 }
 let r = &h64.finish().to_le_bytes();
 let r = base_x::encode("0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ",r);
 js_str(cx,r)
}

ip_bin |cx| {
  let ip = as_str(cx,0)?;
  match ip.parse() {
    Ok(ip)=>match ip{
      IpAddr::V4(ip) => {
        let o = ip.octets();
        js_bin(cx,&[o[0], o[1], o[2], o[3]])
      }
      IpAddr::V6(ip) => {
        let o = ip.octets();
        js_bin(cx,&[
          o[0], o[1], o[2], o[3], o[4], o[5], o[6], o[7], o[8], o[9], o[10], o[11], o[12], o[13],
          o[14], o[15],
        ])
      }
    }
    Err(_) => js_undefined(cx)
  }
}

}
