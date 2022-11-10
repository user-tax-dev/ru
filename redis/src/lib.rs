mod init;
use fred::{
  interfaces::{
    FunctionInterface, HashesInterface, KeysInterface, SetsInterface, SortedSetsInterface,
  },
  pool::RedisPool,
  prelude::{ReconnectPolicy, RedisConfig, ServerConfig as Config},
  types::{Expiration, RedisMap, SetOptions, ZRange, ZRangeBound, ZRangeKind},
};
pub use init::init;
use nlib::*;

alias!(ServerConfig, Config);
alias!(Redis, RedisPool);
as_value_cls!(ServerConfig, Redis);

fn min_max_score(cx: &'_ mut Cx) -> Result<(ZRange, ZRange), Throw> {
  let len = cx.len();
  let min = if len > 2 {
    to_zrange(cx, 2)?
  } else {
    ZRange {
      kind: ZRangeKind::Inclusive,
      range: ZRangeBound::NegInfiniteScore,
    }
  };
  let max = if len > 3 {
    to_zrange(cx, 3)?
  } else {
    ZRange {
      kind: ZRangeKind::Inclusive,
      range: ZRangeBound::InfiniteScore,
    }
  };
  Ok((min, max))
}

fn max_min_score(cx: &'_ mut Cx) -> Result<(ZRange, ZRange), Throw> {
  let len = cx.len();

  let max = if len > 2 {
    to_zrange(cx, 2)?
  } else {
    ZRange {
      kind: ZRangeKind::Inclusive,
      range: ZRangeBound::InfiniteScore,
    }
  };
  let min = if len > 3 {
    to_zrange(cx, 3)?
  } else {
    ZRange {
      kind: ZRangeKind::Inclusive,
      range: ZRangeBound::NegInfiniteScore,
    }
  };
  Ok((max, min))
}

pub fn to_zrange(cx: &'_ mut Cx, n: usize) -> Result<ZRange, Throw> {
  let val = cx.argument::<JsValue>(n)?;
  Ok(if val.is_a::<JsString, _>(cx) {
    val.downcast_or_throw::<JsString, _>(cx)?.value(cx).into()
  } else {
    val
      .downcast_or_throw::<JsNumber, _>(cx)?
      .value(cx)
      .try_into()
      .unwrap()
  })
}

fn limit_offset(cx: &mut FunctionContext, n: usize) -> Result<Option<(i64, i64)>, Throw> {
  let len = cx.len();
  Ok(if len > n {
    let limit = as_f64(cx, n)? as i64;
    let n = n + 1;
    let offset = if len > n { as_f64(cx, n)? as i64 } else { 0 };
    Some((offset, limit))
  } else {
    None
  })
}

macro_rules! this {
  ($cx:ident $this:ident $body:block) => {{
    let $this = &$cx.argument::<JsBox<Redis>>(0)?.0;
    jswait!($cx, $body)?
  }};
}

macro_rules! fcall_ro {
  ($cx:ident, $ty:ty)=>{{
    let name = to_str($cx, 1)?;
    let keys = to_bin_li($cx, 2)?;
    let vals = to_bin_li($cx, 3)?;
    this!($cx this {
      this.fcall_ro::<$ty,_,_,_>(
        name,
        keys,
        vals,
      )
    })
  }}
}

macro_rules! fcall{
  ($cx:ident, $ty:ty)=>{{
    let name = to_str($cx, 1)?;
    let keys = to_bin_li($cx, 2)?;
    let vals = to_bin_li($cx, 3)?;
    if keys.len() > 0{
      this!($cx this {
        this.fcall::<$ty,_,_,_>(
          name,
          keys,
          vals,
        )
      })
    } else {
      this!($cx this {
        this.fcall_ro::<$ty,_,_,_>(
          name,
          keys,
          vals,
        )
      })
    }
  }}
}

js_fn! {

  server_host_port |cx| {
    let host = to_str(cx, 0)?;
    let port = as_f64(cx, 1)? as u16;
    ServerConfig(Config::Centralized { host, port })
  }

  server_cluster |cx| {
    ServerConfig(Config::Clustered {
      hosts:to_kvli(
              cx,
              0,
              jsval2num::<u16>
            )?
    })
  }

  redis_new |cx| {
    let mut conf = RedisConfig { version: fred::types::RespVersion::RESP3, ..Default::default() };
    let server = (*cx.argument::<JsBox<ServerConfig>>(0)?).clone();
    conf.server = server;
    let database = as_f64(cx, 1)? as u8;
    if database != 0 {
      conf.database = Some(database);
    }
    conf.username = Some(to_str(cx, 2)?);
    conf.password = Some(to_str(cx, 3)?);
    let policy = ReconnectPolicy::new_exponential(0, 100, 30_000, 2);

    r#await(
      cx,
      async move {
        //let client = RedisClient::new(conf);
        let client = RedisPool::new(conf, 3)?;
        client.connect(Some(policy));
        client.wait_for_connect().await?;
        Ok(client)
      },
      |mut cx, client| Ok(Redis(client).as_value(&mut cx)),
    )?
  }

  redis_quit |cx| {
    this!(cx this {
      this.quit_pool()
    })
  }

  redis_get |cx| {
    this!(cx this {
      this.get::<Option<String>, _>(to_bin(cx, 1)?)
    })
  }

  redis_get_b |cx| {
    this!(cx this {
      this.get::<Option<Vec<u8>>, _>(to_bin(cx, 1)?)
    })
  }

  redis_set |cx| {
    this!(cx this {
      this.set::<(),_,_>(
        to_bin(cx, 1)?,
        to_bin(cx, 2)?,
        None,
        None,
        false
      )
    })
  }

  redis_setex |cx| {
    this!(cx this  {
      this.set::<(),_,_>(
        to_bin(cx, 1)?,
        to_bin(cx, 2)?,
        Some(Expiration::EX(as_f64(cx, 3)? as _)),
        None,
        false
      )
    })
  }

  redis_expire |cx| {
    this!(cx this {
      this.expire::<bool,_>(
        to_bin(cx, 1)?,
        as_f64(cx, 2)? as _
      )
    })
  }

  redis_del |cx| {
    this!(cx this {
      this.del::<u32,_>(args_bin_li(cx,1)?)
    })
  }

  redis_exist |cx| {
    this!(cx this {
      this.exists::<u32,_>(args_bin_li(cx,1)?)
    })
  }

  redis_hget |cx| {
    this!(cx this {
      this.hget::<Option<String>,_,_>(
        to_bin(cx, 1)?,
        to_bin(cx, 2)?,
      )
    })
  }

  redis_hget_b |cx| {
    this!(cx this {
      this.hget::<Option<Vec<u8>>,_,_>(
        to_bin(cx, 1)?,
        to_bin(cx, 2)?,
      )
    })
  }

  redis_hget_n |cx| {
    this!(cx this {
      this.hget::<Option<f64>,_,_>(
        to_bin(cx, 1)?,
        to_bin(cx, 2)?,
      )
    })
  }

  redis_hset |cx| {
    this!(cx this {
      let val: RedisMap;
      if cx.len() == 3 {
        val = ok!(cx,to_kvli(cx, 2, jsval2bin)?.try_into());
      } else {
        val = ok!(cx,(to_bin(cx, 2)?, to_bin(cx, 3)?).try_into());
      }
      this.hset::<(),_,_>(to_bin(cx, 1)?, val)

    })
  }

  redis_hincrby |cx| {
    this!(cx this {
      this.hincrby::<f64,_,_>(
        to_bin(cx, 1)?,
        to_bin(cx, 2)?,
        as_f64(cx, 3)? as _,
      )
    })
  }

  redis_hincr |cx| {
    this!(cx this {
      this.hincrby::<f64,_,_>(
        to_bin(cx, 1)?,
        to_bin(cx, 2)?,
        1
      )
    })
  }

  redis_hexist |cx| {
    this!(cx this {
      this.hexists::<bool,_,_>(
        to_bin(cx, 1)?,
        to_bin(cx, 2)?,
      )
    })
  }

  redis_sadd |cx| {
    this!(cx this {
      this.sadd::<f64,_,_>(
        to_bin(cx, 1)?,
        args_bin_li(cx, 2)?,
      )
    })
  }

  redis_zscore |cx| {
    this!(cx this {
      this.zscore::<Option<f64>,_,_>(
        to_bin(cx, 1)?,
        to_bin(cx, 2)?,
      )
    })
  }

  redis_zincrby |cx| {
    this!(cx this {
      this.zincrby::<f64,_,_>(
        to_bin(cx, 1)?,
        as_f64(cx, 3)?,
        to_bin(cx, 2)?,
      )
    })
  }

  redis_zincr |cx| {
    this!(cx this {
      this.zincrby::<f64,_,_>(
        to_bin(cx, 1)?,
        1.0,
        to_bin(cx, 2)?,
      )
    })
  }

  // args : key,min,max,[limit],[offset]
  redis_zrangebyscore |cx| {
    this!(cx this {
      let (min,max) = min_max_score(cx)?;
      this.zrangebyscore::<Vec<Vec<u8>>,_,_,_>(
        to_bin(cx, 1)?,
        min,
        max,
        false,
        limit_offset(cx,4)?
      )
    })
  }

  redis_zrangebyscore_withscores |cx| {
    this!(cx this {
      let (min,max) = min_max_score(cx)?;
      this.zrangebyscore::<Vec<(Vec<u8>,f64)>,_,_,_>(
        to_bin(cx, 1)?,
        min,
        max,
        true,
        limit_offset(cx,4)?
      )
    })
  }

  redis_zrevrangebyscore |cx| {
    this!(cx this {
      let (max,min) = max_min_score(cx)?;
      this.zrevrangebyscore::<Vec<Vec<u8>>,_,_,_>(
        to_bin(cx, 1)?,
        max,
        min,
        false,
        limit_offset(cx,4)?
      )
    })
  }

  redis_zrevrangebyscore_withscores |cx| {
    this!(cx this {
      let (max,min) = max_min_score(cx)?;
      this.zrevrangebyscore::<Vec<(Vec<u8>,f64)>,_,_,_>(
        to_bin(cx, 1)?,
        max,
        min,
        true,
        limit_offset(cx,4)?
      )
    })
  }

  redis_zadd |cx| {
    this!(cx this {
      this.zadd::<f64,_,_>(
        to_bin(cx, 1)?,
        None,
        None,
        false,
        false,
        (
          as_f64(cx, 3)?,
          to_bin(cx, 2)?,
        )
      )
    })
  }

  redis_zadd_xx |cx| {
    this!(cx this {
      this.zadd::<f64,_,_>(
        to_bin(cx, 1)?,
        Some(SetOptions::XX),
        None,
        false,
        false,
        (
          as_f64(cx, 3)?,
          to_bin(cx, 2)?,
        )
      )
    })
  }

  redis_fnload |cx| {
    this!(cx this {
      this.function_load::<String,_>(
        true,
        to_str(cx, 1)?,
      )
    })
  }

  redis_fcall |cx| { fcall!(cx,()) }
  redis_fcall_r |cx| { fcall_ro!(cx,()) }
  redis_fbool |cx| { fcall!(cx,Option<bool>) }
  redis_fbool_r |cx| { fcall_ro!(cx,Option<bool>) }
  redis_fbin |cx| { fcall!(cx,Option<Vec<u8>>) }
  redis_fbin_r |cx| { fcall_ro!(cx,Option<Vec<u8>>) }
  redis_fnum |cx| { fcall!(cx,Option<f64>) }
  redis_fnum_r |cx| { fcall_ro!(cx,Option<f64>) }
  redis_fstr |cx| { fcall!(cx,Option<String>) }
  redis_fstr_r |cx| { fcall_ro!(cx,Option<String>) }

  // redis_testz |cx| {
  //  use fred::prelude::RedisValue;
  //  let this = cx.argument::<JsBox<Redis>>(0)?;
  //  let (max,min) = max_min_score(cx)?;
  //  let key = to_bin(cx, 1)?;
  //  let lo = limit_offset(cx,4)?;
  //  let this = &this.0;
  //  let r = this.zrevrangebyscore::<Vec<(RedisValue,RedisValue)>,_,_,_>(
  //    key,
  //    max,
  //    min,
  //    true,
  //    lo
  //  );
  //  let r = async move {
  //    let r = r.await?;
  //    dbg!(&r);
  //    Ok::<_,anyhow::Error>(r)
  //  };

  //  jswait(cx, async move {
  //    let r = r.await?;
  //    Ok(())
  //  })?
  // }
}
