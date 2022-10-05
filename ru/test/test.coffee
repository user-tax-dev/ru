#!/usr/bin/env coffee

> .. > u64Bin binU64 passwordHash

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
  console.log u64Bin
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
