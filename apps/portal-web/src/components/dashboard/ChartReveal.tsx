'use client'

import { ReactNode } from 'react'
import { AnimatePresence, motion } from 'framer-motion'
import ChartSkeleton from './ChartSkeleton'

export default function ChartReveal({
  ready,
  heightClass = 'h-64',
  children,
}: {
  ready: boolean
  heightClass?: string
  children: ReactNode
}) {
  return (
    <div className={`relative ${heightClass}`}>
      <AnimatePresence mode="wait" initial={false}>
        {ready ? (
          <motion.div
            key="chart"
            className="h-full w-full"
            initial={{ opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.28, ease: 'easeOut' }}
          >
            {children}
          </motion.div>
        ) : (
          <motion.div
            key="skeleton"
            className="h-full w-full"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2 }}
          >
            <ChartSkeleton heightClass={heightClass} />
          </motion.div>
        )}
      </AnimatePresence>
      <AnimatePresence>
        {ready ? (
          <motion.div
            key="shine"
            className="pointer-events-none absolute inset-0 rounded-xl"
            initial={{ opacity: 0.22 }}
            animate={{ opacity: 0 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.45, ease: 'easeOut' }}
            style={{ background: 'linear-gradient(110deg, rgba(255,255,255,0) 0%, rgba(255,255,255,0.42) 46%, rgba(255,255,255,0) 100%)' }}
          />
        ) : null}
      </AnimatePresence>
    </div>
  )
}
