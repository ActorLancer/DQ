'use client'

import { motion } from 'framer-motion'

export default function AmbientOrbs() {
  return (
    <div className="pointer-events-none absolute inset-0 overflow-hidden">
      <motion.span
        className="absolute -left-8 top-8 h-24 w-24 rounded-full bg-blue-200/30 blur-xl"
        animate={{ x: [0, 16, 0], y: [0, 8, 0] }}
        transition={{ duration: 8, repeat: Infinity, ease: 'easeInOut' }}
      />
      <motion.span
        className="absolute right-8 top-2 h-20 w-20 rounded-full bg-indigo-200/30 blur-xl"
        animate={{ x: [0, -14, 0], y: [0, 10, 0] }}
        transition={{ duration: 9, repeat: Infinity, ease: 'easeInOut' }}
      />
      <motion.span
        className="absolute right-20 bottom-4 h-16 w-16 rounded-full bg-emerald-200/20 blur-xl"
        animate={{ x: [0, 10, 0], y: [0, -8, 0] }}
        transition={{ duration: 7, repeat: Infinity, ease: 'easeInOut' }}
      />
    </div>
  )
}
