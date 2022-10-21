#!/usr/bin/env coffee

> @rmw/thisdir
  path > resolve join
  @iuser/read
  @iuser/write
  @iuser/camel
  utax/render:

DIR = process.argv[2]
SRC = join resolve(DIR), 'src'
ROOT = thisdir import.meta

if DIR == 'ru'
  do =>
    try
      {default:ru} = await import('../ru/lib/lib.js')
    catch err
      console.log err
      return
    keys = [...Object.keys(ru)]
    console.log join(resolve(DIR),'lib/index.js')
    write(
      join(resolve(DIR),'lib/index.js')
      """import _lib from './lib.js'

export const {#{keys.join(',')}} = _lib
"""
    )
    return
else
  li = []
  for i from read(join(SRC,'lib.rs')).split('\n')
    i = i.trim()
    if i.startsWith '//'
      continue
    pos = i.indexOf '|cx|'
    if pos > 0 and i.indexOf('{',pos) > 0
      func = i[...pos].trim()
      li.push """cx.export_function("#{camel func}", crate::#{func})?;"""
  init_rs = 'init.rs'
  init_fp = join SRC,init_rs
  code = (
    read join ROOT,init_rs
  ).render init: li.join('\n  ')
  if read(init_fp) != code
    write(
      init_fp
      code
    )
