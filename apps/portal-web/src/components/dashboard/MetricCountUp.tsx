'use client'

import { useEffect, useMemo, useState } from 'react'

function parseNumberToken(input: string) {
  const match = input.match(/-?\d[\d,]*(?:\.\d+)?/)
  if (!match) return null
  const token = match[0]
  const start = match.index ?? 0
  const end = start + token.length
  const prefix = input.slice(0, start)
  const suffix = input.slice(end)
  const numeric = Number(token.replace(/,/g, ''))
  const decimals = token.includes('.') ? token.split('.')[1].length : 0
  return { prefix, suffix, numeric, decimals }
}

export default function MetricCountUp({ value, duration = 850 }: { value: string; duration?: number }) {
  const parsed = useMemo(() => parseNumberToken(value), [value])
  const [current, setCurrent] = useState(0)

  useEffect(() => {
    if (!parsed) return
    let raf = 0
    const start = performance.now()
    const from = 0
    const to = parsed.numeric

    const tick = (time: number) => {
      const p = Math.min((time - start) / duration, 1)
      const eased = 1 - Math.pow(1 - p, 3)
      setCurrent(from + (to - from) * eased)
      if (p < 1) raf = requestAnimationFrame(tick)
    }

    raf = requestAnimationFrame(tick)
    return () => cancelAnimationFrame(raf)
  }, [parsed, duration])

  if (!parsed) return <>{value}</>

  const formatted = current.toLocaleString('en-US', {
    minimumFractionDigits: parsed.decimals,
    maximumFractionDigits: parsed.decimals,
  })

  return <>{parsed.prefix}{formatted}{parsed.suffix}</>
}
