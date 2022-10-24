mod init;
use fred::{
  interfaces::{
    FunctionInterface, HashesInterface, KeysInterface, SetsInterface, SortedSetsInterface,
  },
  pool::RedisPool,
  prelude::{ReconnectPolicy, RedisConfig, ServerConfig as Config},
  types::{Expiration, RedisMap},
};
pub use init::init;
use nlib::*;

alias!(ServerConfig, Config);
alias!(Redis, RedisPool);

macro_rules! this {
  ($cx:ident $this:ident $await:ident $body:block) => {{
    let $this = &$cx.argument::<JsBox<Redis>>(0)?.0;
    paste! {
      [<await_ $await>]!($cx,$body)
    }
  }};
}

macro_rules! fcall_ro{
  ($cx:ident, $rt:ident, $ty:ty)=>{{
    let name = to_str($cx, 1)?;
    let keys = to_bin_li($cx, 2)?;
    let vals = to_bin_li($cx, 3)?;
    this!($cx this $rt {
      this.fcall_ro::<$ty,_,_,_>(
        name,
        keys,
        vals,
      )
    })
  }}
}

macro_rules! fcall{
  ($cx:ident, $rt:ident, $ty:ty)=>{{
    let name = to_str($cx, 1)?;
    let keys = to_bin_li($cx, 2)?;
    let vals = to_bin_li($cx, 3)?;
    if keys.len() > 0{
      this!($cx this $rt {
        this.fcall::<$ty,_,_,_>(
          name,
          keys,
          vals,
        )
      })
    } else {
      this!($cx this $rt {
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
    js_box(cx, ServerConfig(Config::Centralized { host, port }))
  }

  server_cluster |cx| {
    let conf = ServerConfig(Config::Clustered {
      hosts:to_kvli(
              cx,
              0,
              jsval2num::<u16>
            )?
    });
    js_box(cx,conf)
  }

  redis_new |cx| {
    let mut conf = RedisConfig { version: fred::types::RespVersion::RESP3, ..Default::default() };
    let server = (*cx.argument::<JsBox<ServerConfig>>(0)?).clone();
    conf.server = server;
    conf.database = Some(as_f64(cx, 1)? as u8);
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
      |mut cx, client| Ok(cx.boxed(Redis(client)).as_value(&mut cx)),
    )
  }

  redis_quit |cx| {
    this!(cx this void {
      this.quit_pool()
    })
  }

  redis_get |cx| {
    this!(cx this option_str {
      this.get::<Option<String>, _>(to_bin(cx, 1)?)
    })
  }

  redis_get_b |cx| {
    this!(cx this option_bin {
      this.get::<Option<Vec<u8>>, _>(to_bin(cx, 1)?)
    })
  }

  redis_set |cx| {
    this!(cx this void {
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
    this!(cx this void {
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
    this!(cx this bool {
      this.expire::<bool,_>(
        to_bin(cx, 1)?,
        as_f64(cx, 2)? as _
      )
    })
  }

  redis_del |cx| {
    this!(cx this f64 {
      this.del::<u32,_>(args_bin_li(cx,1)?)
    })
  }

  redis_exist |cx| {
    this!(cx this f64 {
      this.exists::<u32,_>(args_bin_li(cx,1)?)
    })
  }

  redis_hget |cx| {
    this!(cx this option_str {
      this.hget::<Option<String>,_,_>(
        to_bin(cx, 1)?,
        to_bin(cx, 2)?,
      )
    })
  }

  redis_hget_b |cx| {
    this!(cx this option_bin {
      this.hget::<Option<Vec<u8>>,_,_>(
        to_bin(cx, 1)?,
        to_bin(cx, 2)?,
      )
    })
  }

  redis_hget_n |cx| {
    this!(cx this option_f64 {
      this.hget::<Option<f64>,_,_>(
        to_bin(cx, 1)?,
        to_bin(cx, 2)?,
      )
    })
  }

  redis_hset |cx| {
    this!(cx this void {
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
    this!(cx this f64 {
      this.hincrby::<f64,_,_>(
        to_bin(cx, 1)?,
        to_bin(cx, 2)?,
        as_f64(cx, 3)? as _,
      )
    })
  }

  redis_hincr |cx| {
    this!(cx this f64 {
      this.hincrby::<f64,_,_>(
        to_bin(cx, 1)?,
        to_bin(cx, 2)?,
        1
      )
    })
  }

  redis_hexist |cx| {
    this!(cx this bool {
      this.hexists::<bool,_,_>(
        to_bin(cx, 1)?,
        to_bin(cx, 2)?,
      )
    })
  }

  redis_sadd |cx| {
    this!(cx this f64 {
      this.sadd::<f64,_,_>(
        to_bin(cx, 1)?,
        args_bin_li(cx, 2)?,
      )
    })
  }

  redis_zscore |cx| {
    this!(cx this option_f64 {
      this.zscore::<Option<f64>,_,_>(
        to_bin(cx, 1)?,
        to_bin(cx, 2)?,
      )
    })
  }

  redis_zincrby |cx| {
    this!(cx this f64 {
      this.zincrby::<f64,_,_>(
        to_bin(cx, 1)?,
        as_f64(cx, 3)?,
        to_bin(cx, 2)?,
      )
    })
  }

  redis_zadd |cx| {
    this!(cx this f64 {
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

  redis_fnload |cx| {
    this!(cx this str {
      this.function_load::<String,_>(
        true,
        to_str(cx, 1)?,
      )
    })
  }

  redis_fcall |cx| { fcall!(cx,void,()) }
  redis_fcall_r |cx| { fcall_ro!(cx,void,()) }
  redis_fbool |cx| { fcall!(cx,option_bool,Option<bool>) }
  redis_fbool_r |cx| { fcall_ro!(cx,option_bool,Option<bool>) }
  redis_fbin |cx| { fcall!(cx,option_bin,Option<Vec<u8>>) }
  redis_fbin_r |cx| { fcall_ro!(cx,option_bin,Option<Vec<u8>>) }
  redis_fnum |cx| { fcall!(cx,option_f64,Option<f64>) }
  redis_fnum_r |cx| { fcall_ro!(cx,option_f64,Option<f64>) }
  redis_fstr |cx| { fcall!(cx,option_str,Option<String>) }
  redis_fstr_r |cx| { fcall_ro!(cx,option_str,Option<String>) }

}
