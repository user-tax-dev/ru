#![feature(macro_metavar_expr)]

pub use neon::prelude::*;
use neon::{result::Throw, types::buffer::TypedArray};
use num_traits::AsPrimitive;
use once_cell::sync::OnceCell;
pub use paste::paste;
use tokio::runtime::Runtime;

#[macro_export]
macro_rules! js_fn {
  ($fn:ident |$cx:ident| $body:block) => {
    nlib::paste! {
      pub fn $fn(mut $cx: Cx) -> JsResult<JsValue> {
        let $cx = &mut $cx;
        $body
      }
    }
  };
  ($($fn:ident |$cx:ident| $body:block)+) => {
    $(
      js_fn!($fn |$cx| $body);
    )+
  }
}

#[macro_export]
macro_rules! alias {
  ($cls:ident,$real:ident) => {
    pub struct $cls($real);
    nlib::paste! {
    pub type [<Js $cls>] = JsBox<$cls>;
    }
    impl std::ops::Deref for $cls {
      type Target = $real;

      fn deref(&self) -> &Self::Target {
        return &self.0;
      }
    }
    impl Finalize for $cls {}
  };
}

pub type Cx<'a> = FunctionContext<'a>;

pub fn as_str(cx: &'_ mut Cx, n: usize) -> Result<String, Throw> {
  Ok(cx.argument::<JsString>(n)?.value(cx))
}

pub fn as_f64(cx: &'_ mut Cx, n: usize) -> Result<f64, Throw> {
  Ok(cx.argument::<JsNumber>(n)?.value(cx))
}

pub fn as_bin<'a, 'b>(cx: &'a mut Cx<'b>, n: usize) -> Result<&'a [u8], Throw> {
  Ok(cx.argument::<JsUint8Array>(n)?.as_slice(cx))
}

pub fn to_kvli<V>(
  cx: &'_ mut Cx,
  n: usize,
  to_val: impl FnOnce(&'_ mut Cx, Handle<'_, JsValue>) -> Result<V, Throw> + Copy,
) -> Result<Vec<(String, V)>, Throw> {
  let mut kv = vec![];
  let ob = cx.argument::<JsObject>(n)?;
  for i in ob.get_own_property_names(cx)?.to_vec(cx)? {
    let k = i.downcast_or_throw::<JsString, _>(cx)?.value(cx);
    let v = ob.get_value(cx, k.as_ref())?;
    let v = to_val(cx, v)?;
    kv.push((k, v));
  }
  Ok(kv)
}

pub fn to_bin_li(cx: &'_ mut Cx, n: usize) -> Result<Vec<Box<[u8]>>, Throw> {
  to_li(cx, n, jsval2bin)
}

pub fn to_li<V>(
  cx: &'_ mut Cx,
  n: usize,
  to_val: impl FnOnce(&'_ mut Cx, Handle<'_, JsValue>) -> Result<V, Throw> + Copy,
) -> Result<Vec<V>, Throw> {
  let val = cx.argument::<JsValue>(n)?;
  Ok(if val.is_a::<JsUndefined, _>(cx) {
    vec![]
  } else {
    let val = val.downcast_or_throw::<JsArray, _>(cx)?.to_vec(cx)?;
    let mut li = vec![];
    for v in val {
      let v = to_val(cx, v)?;
      li.push(v);
    }
    li
  })
}

pub fn jsval2num<V: Copy + 'static>(cx: &'_ mut Cx, val: Handle<'_, JsValue>) -> Result<V, Throw>
where
  f64: AsPrimitive<V>,
{
  Ok(val.downcast_or_throw::<JsNumber, _>(cx)?.value(cx).as_())
}

pub fn jsval2str(cx: &'_ mut Cx, val: Handle<'_, JsValue>) -> Result<String, Throw> {
  Ok(if val.is_a::<JsString, _>(cx) {
    val.downcast_or_throw::<JsString, _>(cx)?.value(cx)
  } else if val.is_a::<JsNumber, _>(cx) {
    val
      .downcast_or_throw::<JsNumber, _>(cx)?
      .value(cx)
      .to_string()
  } else {
    let r = val.downcast_or_throw::<JsUint8Array, _>(cx)?;
    String::from_utf8_lossy(r.as_slice(cx)).to_string()
  })
}

pub fn to_str(cx: &'_ mut Cx, n: usize) -> Result<String, Throw> {
  let val = cx.argument::<JsValue>(n)?;
  jsval2str(cx, val)
}

pub fn jsval2bin(cx: &'_ mut Cx, val: Handle<'_, JsValue>) -> Result<Box<[u8]>, Throw> {
  Ok(if val.is_a::<JsString, _>(cx) {
    Box::from(val.downcast_or_throw::<JsString, _>(cx)?.value(cx).as_ref())
  } else if val.is_a::<JsNumber, _>(cx) {
    Box::from(
      val
        .downcast_or_throw::<JsNumber, _>(cx)?
        .value(cx)
        .to_string()
        .as_ref(),
    )
  } else {
    let r = val.downcast_or_throw::<JsUint8Array, _>(cx)?;
    Box::from(r.as_slice(cx))
  })
}

pub fn to_bin(cx: &'_ mut Cx, n: usize) -> Result<Box<[u8]>, Throw> {
  let val = cx.argument::<JsValue>(n)?;
  jsval2bin(cx, val)
}

pub fn args_bin_li(cx: &'_ mut Cx, offset: usize) -> Result<Vec<Box<[u8]>>, Throw> {
  let mut li = vec![];
  for i in offset..cx.len() {
    li.push(to_bin(cx, i)?);
  }
  Ok(li)
}

pub fn runtime<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<&'static Runtime> {
  static RUNTIME: OnceCell<Runtime> = OnceCell::new();
  RUNTIME.get_or_try_init(|| Runtime::new().or_else(|err| cx.throw_error(err.to_string())))
}

pub fn r#await<'a, T: Send + 'static, C: Context<'a>>(
  cx: &mut C,
  f: impl std::future::Future<Output = anyhow::Result<T>> + Send + 'static,
  rt: impl FnOnce(TaskContext<'_>, T) -> JsResult<'_, JsValue> + Send + 'static,
) -> JsResult<'a, JsValue> {
  let (deferred, promise) = cx.promise();
  let promise = promise.as_value(cx);
  let channel = cx.channel();
  runtime(cx)?.spawn(async move {
    let r: anyhow::Result<T> = f.await;

    deferred.try_settle_with(&channel, move |mut cx| match r {
      Err(err) => cx.throw_error(err.to_string()),
      Ok(r) => rt(cx, r),
    })
  });
  Ok(promise)
}

pub fn js_undefined<'a, C: Context<'a>>(cx: &mut C) -> JsResult<'a, JsValue> {
  Ok(cx.undefined().as_value(cx))
}

pub fn js_box<'a, C: Context<'a>, T: 'static + Send + Finalize>(
  cx: &mut C,
  t: T,
) -> JsResult<'a, JsValue> {
  Ok(cx.boxed(t).as_value(cx))
}

macro_rules! await_trait {
  ($to:ident, $t:ty, $r:ty) => {
    paste! {
      pub fn [<await_ $to>]<'a, T: 'static + Send + $t, C: Context<'a>>(
        cx: &mut C,
        f: impl std::future::Future<Output = anyhow::Result<$r>> + Send + 'static,
      ) -> JsResult<'a, JsValue> {
        r#await(cx, f, |mut cx, r| [<js_ $to>](&mut cx, r))
      }
      #[macro_export]
      macro_rules! [<await_ $to>] {
        ($$cx:expr, $$r:expr) => {{
          let r = $$r;
          [<await_ $to>]($$cx, async move { Ok(r.await?) })
        }};
      }
    }
  };
  ($($to:ident $t:ty | $cx:ident , $o:ident | $f:block )+) => {
    paste!{
      $(
        await_trait!([<$to>],$t,T);
        await_trait!([<option_ $to>],$t,Option<T>);
        pub fn [<js_option_ $to>]<'a, C: Context<'a>, T:$t>(
          cx: &mut C,
          b: Option<T>,
        ) -> JsResult<'a, JsValue> {
          match b {
            Some(b) => [<js_ $to>](cx, b),
            None => Ok(cx.undefined().as_value(cx)),
          }
        }
        pub fn [<js_ $to>]<'a, C: Context<'a>, T:$t>($cx: &mut C, $o: T) -> JsResult<'a, JsValue> {
          Ok($f.as_value($cx))
        }
      )+
    }
  }
}

await_trait!(
  f64 Into<f64> |cx,t| {cx.number(t)}
  str AsRef<str> |cx,t| { cx.string(t) }
  bin AsRef<[u8]> |cx,t| { JsUint8Array::from_slice(cx, t.as_ref())? }
  bool Into<bool> |cx,t| { cx.boolean(t.into()) }
);

#[macro_export]
macro_rules! await_void {
  ($cx:expr, $r:expr) => {{
    let r = $r;
    await_void($cx, async move { Ok(r.await?) })
  }};
}

pub fn await_void<'a, C: Context<'a>>(
  cx: &mut C,
  f: impl std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
) -> JsResult<'a, JsValue> {
  r#await(cx, f, |mut cx, _| js_undefined(&mut cx))
}
