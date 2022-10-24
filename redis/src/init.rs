use neon::prelude::*;

pub fn init(cx: &mut ModuleContext) -> NeonResult<()> {
  cx.export_function("serverHostPort", crate::server_host_port)?;
  cx.export_function("serverCluster", crate::server_cluster)?;
  cx.export_function("redisNew", crate::redis_new)?;
  cx.export_function("redisQuit", crate::redis_quit)?;
  cx.export_function("redisGet", crate::redis_get)?;
  cx.export_function("redisGetB", crate::redis_get_b)?;
  cx.export_function("redisSet", crate::redis_set)?;
  cx.export_function("redisSetex", crate::redis_setex)?;
  cx.export_function("redisExpire", crate::redis_expire)?;
  cx.export_function("redisDel", crate::redis_del)?;
  cx.export_function("redisExist", crate::redis_exist)?;
  cx.export_function("redisHget", crate::redis_hget)?;
  cx.export_function("redisHgetB", crate::redis_hget_b)?;
  cx.export_function("redisHgetN", crate::redis_hget_n)?;
  cx.export_function("redisHset", crate::redis_hset)?;
  cx.export_function("redisHincrby", crate::redis_hincrby)?;
  cx.export_function("redisHincr", crate::redis_hincr)?;
  cx.export_function("redisHexist", crate::redis_hexist)?;
  cx.export_function("redisSadd", crate::redis_sadd)?;
  cx.export_function("redisZscore", crate::redis_zscore)?;
  cx.export_function("redisZincrby", crate::redis_zincrby)?;
  cx.export_function("redisZincr", crate::redis_zincr)?;
  cx.export_function("redisZadd", crate::redis_zadd)?;
  cx.export_function("redisFnload", crate::redis_fnload)?;
  cx.export_function("redisFcall", crate::redis_fcall)?;
  cx.export_function("redisFcallR", crate::redis_fcall_r)?;
  cx.export_function("redisFbool", crate::redis_fbool)?;
  cx.export_function("redisFboolR", crate::redis_fbool_r)?;
  cx.export_function("redisFbin", crate::redis_fbin)?;
  cx.export_function("redisFbinR", crate::redis_fbin_r)?;
  cx.export_function("redisFnum", crate::redis_fnum)?;
  cx.export_function("redisFnumR", crate::redis_fnum_r)?;
  cx.export_function("redisFstr", crate::redis_fstr)?;
  cx.export_function("redisFstrR", crate::redis_fstr_r)?;
  Ok(())
}

#[cfg(feature = "main")]
#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
  crate::init(&mut cx)?;
  Ok(())
}
