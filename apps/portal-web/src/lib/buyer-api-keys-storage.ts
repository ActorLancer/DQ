import { ApiKey, MOCK_API_KEYS } from '@/lib/buyer-api-keys-data'

const STORAGE_KEY = 'buyer-api-keys-storage-v1'
const SYNC_EVENT = 'buyer-api-keys-updated'

function canUseStorage() {
  return typeof window !== 'undefined' && typeof window.localStorage !== 'undefined'
}

export function getBuyerApiKeys(): ApiKey[] {
  if (!canUseStorage()) return MOCK_API_KEYS
  const raw = window.localStorage.getItem(STORAGE_KEY)
  if (!raw) return MOCK_API_KEYS
  try {
    const parsed = JSON.parse(raw) as ApiKey[]
    return Array.isArray(parsed) ? parsed : MOCK_API_KEYS
  } catch {
    return MOCK_API_KEYS
  }
}

export function saveBuyerApiKeys(keys: ApiKey[]) {
  if (!canUseStorage()) return
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(keys))
  window.dispatchEvent(new Event(SYNC_EVENT))
}

export function bootstrapBuyerApiKeys() {
  if (!canUseStorage()) return
  const raw = window.localStorage.getItem(STORAGE_KEY)
  if (raw) return
  saveBuyerApiKeys(MOCK_API_KEYS)
}

export function rotateBuyerApiKey(keyId: string): ApiKey | null {
  const keys = getBuyerApiKeys()
  let changed: ApiKey | null = null
  const next = keys.map((key) => {
    if (key.id !== keyId) return key
    changed = {
      ...key,
      keyPrefix: key.keyPrefix.startsWith('sk_live_') ? 'sk_live_rot_' : 'sk_test_rot_',
      createdAt: new Date().toISOString().slice(0, 19).replace('T', ' '),
      lastUsedAt: null,
    }
    return changed
  })
  saveBuyerApiKeys(next)
  return changed
}

export function disableBuyerApiKey(keyId: string): ApiKey | null {
  const keys = getBuyerApiKeys()
  let changed: ApiKey | null = null
  const next = keys.map((key) => {
    if (key.id !== keyId) return key
    changed = { ...key, status: 'DISABLED' }
    return changed
  })
  saveBuyerApiKeys(next)
  return changed
}

export function enableBuyerApiKey(keyId: string): ApiKey | null {
  const keys = getBuyerApiKeys()
  let changed: ApiKey | null = null
  const next = keys.map((key) => {
    if (key.id !== keyId) return key
    changed = { ...key, status: 'ACTIVE' }
    return changed
  })
  saveBuyerApiKeys(next)
  return changed
}

export function deleteBuyerApiKey(keyId: string): boolean {
  const keys = getBuyerApiKeys()
  const next = keys.filter((key) => key.id !== keyId)
  const deleted = next.length !== keys.length
  if (deleted) saveBuyerApiKeys(next)
  return deleted
}

export function onBuyerApiKeysUpdated(listener: () => void) {
  if (typeof window === 'undefined') return () => {}
  window.addEventListener(SYNC_EVENT, listener)
  window.addEventListener('storage', listener)
  return () => {
    window.removeEventListener(SYNC_EVENT, listener)
    window.removeEventListener('storage', listener)
  }
}
