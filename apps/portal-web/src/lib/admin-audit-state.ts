'use client'

import { useEffect, useMemo, useState } from 'react'

const STORAGE_KEY = 'admin_audit_state_v1'
const UPDATE_EVENT = 'admin-audit-state-updated'

export interface AuditUiState {
  processed: boolean
  replayCount: number
  lastActionAt?: string
  timeline?: Array<{
    type: 'REPLAY' | 'MARK_PROCESSED' | 'UNMARK_PROCESSED'
    at: string
    note: string
  }>
}

type AuditStateMap = Record<string, AuditUiState>

function readStateMap(): AuditStateMap {
  if (typeof window === 'undefined') return {}
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) return {}
    return JSON.parse(raw) as AuditStateMap
  } catch {
    return {}
  }
}

function writeStateMap(map: AuditStateMap) {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(map))
  window.dispatchEvent(new CustomEvent(UPDATE_EVENT))
}

export function updateAuditState(auditId: string, updater: (prev: AuditUiState) => AuditUiState) {
  const prevMap = readStateMap()
  const prev = prevMap[auditId] ?? { processed: false, replayCount: 0 }
  const next = updater(prev)
  writeStateMap({ ...prevMap, [auditId]: next })
}

export function useAdminAuditStateMap() {
  const [stateMap, setStateMap] = useState<AuditStateMap>({})

  useEffect(() => {
    const sync = () => setStateMap(readStateMap())
    sync()
    window.addEventListener('storage', sync)
    window.addEventListener(UPDATE_EVENT, sync as EventListener)
    return () => {
      window.removeEventListener('storage', sync)
      window.removeEventListener(UPDATE_EVENT, sync as EventListener)
    }
  }, [])

  return stateMap
}

export function useAuditUiState(auditId: string) {
  const map = useAdminAuditStateMap()
  return useMemo(() => map[auditId] ?? { processed: false, replayCount: 0 }, [map, auditId])
}
