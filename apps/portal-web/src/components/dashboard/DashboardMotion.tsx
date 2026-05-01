'use client'

import { ReactNode } from 'react'
import { motion } from 'framer-motion'

export function DashboardStagger({ children, className = '' }: { children: ReactNode; className?: string }) {
  return (
    <motion.div
      className={className}
      initial="hidden"
      animate="show"
      variants={{
        hidden: {},
        show: {
          transition: { staggerChildren: 0.06, delayChildren: 0.04 },
        },
      }}
    >
      {children}
    </motion.div>
  )
}

export function DashboardFadeItem({ children, className = '' }: { children: ReactNode; className?: string }) {
  return (
    <motion.div
      layout
      className={className}
      variants={{
        hidden: { opacity: 0, y: 10, filter: 'blur(2px)' },
        show: { opacity: 1, y: 0, filter: 'blur(0px)' },
      }}
      transition={{ duration: 0.28, ease: 'easeOut', layout: { type: 'spring', bounce: 0.16, duration: 0.45 } }}
    >
      {children}
    </motion.div>
  )
}
