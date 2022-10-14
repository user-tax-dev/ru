#!/usr/bin/env coffee

> ..:lib

{
  u64Bin
  binU64
  b64
  unb64
  xxh3B36
  xxh3
  passwordHash
  tld
  randomBytes
  cookieDecode
  cookieEncode
} = lib

sleep = =>
  new Promise((resolve) => setTimeout(resolve, 10))

minute = =>
  parseInt new Date()/6e4

main = =>
  Promise.all [
    passwordHash u64Bin(1)
    passwordHash Buffer.from([0]),Buffer.from([2])
    passwordHash 'a'
    passwordHash Buffer.from([97])
    passwordHash new Uint8Array([97])
  ]

cost = (p)=>
  begin = new Date
  r = await p
  console.log r, 'cost',Math.round(new Date()-begin)/1000

  r

do =>
  sk = '123'
  c = randomBytes(8)
  console.log c, xxh3(sk, c)
  console.log cookieEncode c, xxh3(sk, c)
  console.log cookieDecode cookieEncode c, xxh3(sk, c)

  return
  console.log tld 'github.io'
  console.log tld 'worri.3.1.github.io'
  console.log tld 'worri.3.1.github.com.cn'
  console.log tld 'worri.3.1.github.com'
  console.log tld 'worri.3.1.123.io'
  console.log tld 'worri.3.1.123.中国'
  console.log tld 'worri.3.1.123.中国.pro.typeform.com
'
  console.log xxh3B36 '1234'
  hash = await passwordHash new Uint8Array([97])
  console.log hash
  console.log b64 hash
  console.log unb64 b64 hash
  return
  await cost main()
  await cost passwordHash new Uint8Array([97])
  begin = minute()
  {rss} = process.memoryUsage()
  n = 0
  pre = 0
  loop
    await main()
    if ++n%100 == 1
      gc()
      await sleep()

      leak = parseInt((process.memoryUsage().rss-rss)/1024/1024)
      if leak != pre
        pre = leak
        console.log(
          minute()-begin,'minute'
          n,'loop'
          'leak', leak,'MB'
        )
  return
