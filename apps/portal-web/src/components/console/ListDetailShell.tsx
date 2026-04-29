'use client'

import { AnimatePresence, motion } from 'framer-motion'
import { ReactNode } from 'react'
import { X } from 'lucide-react'

interface ListDetailShellProps {
  children: ReactNode
  isOpen: boolean
  title: string
  onClose: () => void
  detail: ReactNode
  mobileSheet?: boolean
}

export default function ListDetailShell({ children, isOpen, title, onClose, detail, mobileSheet = false }: ListDetailShellProps) {
  return (
    <>
      <div>{children}</div>

      <AnimatePresence>
        {isOpen && (
          <>
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.15 }}
              className="fixed inset-0 z-40 bg-black/20"
              onClick={onClose}
            />
            {mobileSheet && (
              <motion.aside
                initial={{ y: '100%' }}
                animate={{ y: 0 }}
                exit={{ y: '100%' }}
                transition={{ duration: 0.25, ease: 'easeOut' }}
                drag="y"
                dragConstraints={{ top: 0, bottom: 0 }}
                dragElastic={0.12}
                onDragEnd={(_, info) => {
                  if (info.offset.y > 120) onClose()
                }}
                className="fixed inset-x-0 bottom-0 z-50 h-[78vh] rounded-t-2xl border border-gray-200 bg-white shadow-2xl md:hidden"
              >
                <div className="flex h-full flex-col">
                  <div className="px-6 pt-3 pb-2">
                    <div className="mx-auto mb-3 h-1.5 w-12 rounded-full bg-gray-300" />
                    <div className="flex items-center justify-between border-b border-gray-200 pb-3">
                      <h3 className="text-lg font-bold text-gray-900">{title}</h3>
                      <button onClick={onClose} className="rounded-lg p-2 text-gray-500 hover:bg-gray-100 hover:text-gray-700">
                        <X className="h-5 w-5" />
                      </button>
                    </div>
                  </div>
                  <div className="flex-1 overflow-y-auto px-6 pb-6">{detail}</div>
                </div>
              </motion.aside>
            )}
            <motion.aside
              initial={{ x: 28, opacity: 0 }}
              animate={{ x: 0, opacity: 1 }}
              exit={{ x: 28, opacity: 0 }}
              transition={{ duration: 0.22, ease: 'easeOut' }}
              className={`fixed right-0 top-16 bottom-0 z-50 w-full max-w-[520px] border-l border-gray-200 bg-white shadow-2xl ${mobileSheet ? 'hidden md:block' : ''}`}
            >
              <div className="flex h-full flex-col">
                <div className="flex items-center justify-between border-b border-gray-200 px-6 py-4">
                  <h3 className="text-xl font-bold text-gray-900">{title}</h3>
                  <button onClick={onClose} className="rounded-lg p-2 text-gray-500 hover:bg-gray-100 hover:text-gray-700">
                    <X className="h-5 w-5" />
                  </button>
                </div>
                <div className="flex-1 overflow-y-auto p-6">{detail}</div>
              </div>
            </motion.aside>
          </>
        )}
      </AnimatePresence>
    </>
  )
}
