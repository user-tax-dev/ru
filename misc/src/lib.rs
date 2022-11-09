mod init;
use std::{
  hash::{BuildHasher, Hasher},
  net::IpAddr,
};

use base64_simd::Base64;
pub use init::init;
use nlib::*;
use ordered_varint::Variable;
use xxhash_rust::{xxh3::Xxh3Builder, xxh32::Xxh32};

const XXHASHER: Xxh3Builder = Xxh3Builder::new();
const BASE64: Base64 = Base64::URL_SAFE_NO_PAD;

const COOKIE_SAFE_CHAR: &str =
  "!#$%&'()*+-./0123456789:<>?@ABDEFGHIJKLMNQRSTUVXYZ[]^_`abdefghijklmnqrstuvxyz{|}~";

fn _b64_f64(bytes: &[u8]) -> anyhow::Result<f64> {
  let r = BASE64.decode_to_boxed_bytes(bytes)?;
  let mut x = [0u8; 8];
  x[..r.len()].clone_from_slice(&r);
  Ok(u64::from_le_bytes(x) as f64)
}

pub fn is_ascii_digit(bytes: &[u8]) -> bool {
  bytes.iter().all(|i| {
    let i = *i;
    (b'0'..=b'9').contains(&i)
  })
}

js_fn! {
  zip_u64 |cx| {
    let mut li = vec![];
    let len = cx.len();
    for i in 0..len {
      li.push( as_f64(cx, i)? as u64 );
    }
    let data_len = stream_vbyte64::max_compressed_len(len);
    let mut buf = vec![0; data_len];
    let len = stream_vbyte64::encode(&li, &mut buf);
    Box::<[u8]>::from(&buf[..len])
  }

  unzip_u64 |cx| {
    let len = as_f64(cx, 0)? as usize;
    let bin = as_bin(cx, 1)?;
    let mut decoded = vec![0; len];
    stream_vbyte64::decode(&mut decoded, bin);
    decoded
  }

  unb64 |cx| {
    let s = to_bin(cx,0)?;
    ok!(cx, BASE64.decode_to_boxed_bytes(&s))
  }

  b64 |cx| {
    let mut li = vec![];
    for i in 0..cx.len() {
      let bin = to_bin(cx, i)?;
      li.extend_from_slice(&bin);
    }
    BASE64.encode_to_boxed_str(&li)
  }

  b64_u64 |cx| {
    let s = to_bin(cx,0)?;
    ok!(cx,_b64_f64(&s))
  }

  u64_b64 |cx| {
    let x = (as_f64(cx, 0)? as u64).to_le_bytes();
    let x = &x;
    let mut n = x.len();
    while n != 0 {
      n-=1;
      if x[n] != 0 {
        break;
      }
    }
    BASE64.encode_to_boxed_str(&x[..n+1])
  }

  u64_bin |cx| {
    let x = as_f64(cx, 0)? as u64;
    ok!(cx,x.to_variable_vec())
  }

  bin_u64 |cx| {
    let x = as_bin(cx, 0)?;
    ok!(cx, u64::decode_variable(x))
  }

  password_hash |cx| {
    let mut hasher = blake3::Hasher::new();
    for i in 0..cx.len() {
      let bin = to_bin(cx, i)?;
      hasher.update(&bin);
    }
    const N: usize = 512;
    jswait(cx, async move {
      let mut output = [0; N];
      for _ in 1..N {
        hasher.finalize_xof().fill(&mut output);
        hasher.update(&output);
      }
      let mut output = [0; 16];
      hasher.finalize_xof().fill(&mut output);
      Ok(Box::<[u8]>::from(&output[..]))
    })?
  }

  z85_load |cx| {
    let s = to_bin(cx,0)?;
    ok!(cx, z85::decode(s))
  }

  z85_dump |cx| {
    let bin = as_bin(cx,0)?;
    z85::encode(bin)
  }

  random_bytes |cx| {
    let n = as_f64(cx,0)? as usize;
    (0..n).map(
        |_| rand::random::<u8>()
    ).collect::<Vec<u8>>()
  }

  cookie_encode |cx| {
    let li = args_bin_li(cx,0)?;
    let li = li.concat();
    base_x::encode(COOKIE_SAFE_CHAR,&li)
  }

  cookie_decode |cx| {
    let bin = to_str(cx, 0)?;
    ok!(cx, base_x::decode(COOKIE_SAFE_CHAR,&bin))
  }

  xxh64 |cx| {
    let li = args_bin_li(cx,0)?;
    let mut h64 = XXHASHER.build_hasher();
    for i in li {
      h64.update(i.as_ref());
    }
    h64.finish().to_le_bytes()
  }

  xxh32 |cx| {
    let li = args_bin_li(cx,0)?;
    let mut h = Xxh32::new(0);
    for i in li {
      h.update(i.as_ref());
    }
    h.digest().to_le_bytes()
  }

  xxh3_b36 |cx| {
    let li = args_bin_li(cx,0)?;
    let mut h64 = XXHASHER.build_hasher();
    for i in li {
      h64.update(i.as_ref());
    }
    let r = h64.finish().to_le_bytes();
    let mut n = 0;
    while n < 6 {
      if r[n]!=0 {
        break;
      }
      n+=1;
    }
    base_x::encode("123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz",&r[n..])
  }

  ip_bin |cx| {
    let ip = as_str(cx,0)?;
    let ip:IpAddr = ok!(cx,ip.parse());
    match ip{
      IpAddr::V4(ip) => {
        let o = ip.octets();
        Box::<[u8]>::from([o[0], o[1], o[2], o[3]])
      }
      IpAddr::V6(ip) => {
        let o = ip.octets();
        Box::<[u8]>::from([
          o[0], o[1], o[2], o[3], o[4], o[5], o[6], o[7], o[8], o[9], o[10], o[11], o[12], o[13],
          o[14], o[15],
        ])
      }
    }
  }

  tld |cx| {
    let mut domain = &to_bin(cx, 0)?[..];
    if let Some(d) = psl::domain(domain){
      let bytes = d.suffix().as_bytes();
      let len = bytes.len();
      if len > 0 && !is_ascii_digit(bytes) {
        let mut n = domain.len()-len;
        n = n.saturating_sub(1);
        while n > 0 {
          let t=n-1;
          if domain[t] == b'.' {
            break;
          }
          n=t;
        }
        domain = &domain[n..]
      }
    }
    unsafe { String::from_utf8_unchecked(domain.into()) }
  }

}
