import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export interface AdminUser {
  id: string
  name: string
  email: string
  permissions: string[]
}

interface AdminAuthState {
  user: AdminUser | null
  token: string | null
  isAuthenticated: boolean
  login: (token: string, user: AdminUser) => void
  logout: () => void
  hasPermission: (permission: string) => boolean
}

export const useAdminAuthStore = create<AdminAuthState>()(
  persist(
    (set, get) => ({
      user: null,
      token: null,
      isAuthenticated: false,

      login: (token: string, user: AdminUser) => {
        set({ token, user, isAuthenticated: true })
      },

      logout: () => {
        set({ token: null, user: null, isAuthenticated: false })
      },

      hasPermission: (permission: string) => {
        const { user } = get()
        if (!user) return false
        return user.permissions.includes(permission)
      },
    }),
    {
      name: 'admin-auth-storage',
    }
  )
)
