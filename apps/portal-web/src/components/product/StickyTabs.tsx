'use client'

import { useState, useEffect } from 'react'

const TABS = [
  { id: 'overview', label: 'Overview' },
  { id: 'schema', label: 'Schema' },
  { id: 'sample', label: 'Sample' },
  { id: 'pricing', label: 'Pricing' },
  { id: 'docs', label: 'Docs' },
  { id: 'reviews', label: 'Reviews' },
]

interface StickyTabsProps {
  activeTab: string
  onTabChange: (tabId: string) => void
}

export default function StickyTabs({ activeTab, onTabChange }: StickyTabsProps) {
  const [isSticky, setIsSticky] = useState(false)

  useEffect(() => {
    const handleScroll = () => {
      setIsSticky(window.scrollY > 300)
    }

    window.addEventListener('scroll', handleScroll)
    return () => window.removeEventListener('scroll', handleScroll)
  }, [])

  return (
    <div
      className={`${
        isSticky ? 'sticky top-16 z-40 shadow-md' : ''
      } bg-white border-b border-gray-200 transition-shadow`}
    >
      <div className="container-custom">
        <nav className="flex space-x-8">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              onClick={() => onTabChange(tab.id)}
              className={`py-4 px-2 font-medium text-sm border-b-2 transition-colors ${
                activeTab === tab.id
                  ? 'border-primary-600 text-primary-600'
                  : 'border-transparent text-gray-600 hover:text-gray-900 hover:border-gray-300'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </nav>
      </div>
    </div>
  )
}
