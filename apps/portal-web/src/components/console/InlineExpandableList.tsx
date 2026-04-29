'use client'

import { AnimatePresence, motion } from 'framer-motion'
import { ReactNode } from 'react'

interface InlineExpandableListProps<T> {
  items: T[]
  getKey: (item: T) => string
  selectedKey: string | null
  onSelect: (key: string | null) => void
  onOpenDetail?: (item: T) => void
  highlightedKey?: string | null
  getItemId?: (item: T) => string
  renderSummary: (item: T, isSelected: boolean) => ReactNode
  renderExpanded: (item: T) => ReactNode
}

export default function InlineExpandableList<T>({
  items,
  getKey,
  selectedKey,
  onSelect,
  onOpenDetail,
  highlightedKey,
  getItemId,
  renderSummary,
  renderExpanded,
}: InlineExpandableListProps<T>) {
  return (
    <div className="space-y-4">
      {items.map((item) => {
        const key = getKey(item)
        const isSelected = selectedKey === key

        return (
          <motion.div
            layout
            key={key}
            id={getItemId ? getItemId(item) : undefined}
            onClick={() => onSelect(isSelected ? null : key)}
            onDoubleClick={() => onOpenDetail?.(item)}
            className={`bg-white rounded-xl border-2 p-6 cursor-pointer transition-all ${
              isSelected ? 'border-primary-500 shadow-lg' : 'border-gray-200 hover:border-primary-300 hover:shadow-md'
            } ${highlightedKey === key ? 'ring-2 ring-primary-400 ring-offset-2' : ''}`}
          >
            {renderSummary(item, isSelected)}

            <AnimatePresence initial={false}>
              {isSelected && (
                <motion.div
                  initial={{ height: 0, opacity: 0 }}
                  animate={{ height: 'auto', opacity: 1 }}
                  exit={{ height: 0, opacity: 0 }}
                  transition={{ duration: 0.22, ease: 'easeOut' }}
                  className="overflow-hidden"
                >
                  <div className="mt-5 border-t border-gray-200 pt-5">{renderExpanded(item)}</div>
                </motion.div>
              )}
            </AnimatePresence>
          </motion.div>
        )
      })}
    </div>
  )
}
