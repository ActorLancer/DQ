import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export type UserRole = 'buyer' | 'seller' | 'admin'

export interface User {
  id: string
  name: string
  email: string
  roles: UserRole[]
  currentRole: UserRole
  subjectId: string
  subjectName: string
  tenantId: string
  permissions: string[]
}

interface AuthState {
  user: User | null
  token: string | null
  isAuthenticated: boolean
  login: (token: string, user: User) => void
  logout: () => void
  setCurrentRole: (role: UserRole) => void
  updateUser: (user: Partial<User>) => void
  hasPermission: (permission: string) => boolean
  hasRole: (role: UserRole) => boolean
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      token: null,
      isAuthenticated: false,

      login: (token: string, user: User) => {
        set({ token, user, isAuthenticated: true })
      },

      logout: () => {
        set({ token: null, user: null, isAuthenticated: false })
      },

      setCurrentRole: (role: UserRole) => {
        const { user } = get()
        if (!user || !user.roles.includes(role)) return

        set({
          user: {
            ...user,
            currentRole: role,
          },
        })
      },

      updateUser: (userData: Partial<User>) => {
        const { user } = get()
        if (!user) return

        set({
          user: {
            ...user,
            ...userData,
          },
        })
      },

      hasPermission: (permission: string) => {
        const { user } = get()
        if (!user) return false
        return user.permissions.includes(permission)
      },

      hasRole: (role: UserRole) => {
        const { user } = get()
        if (!user) return false
        return user.roles.includes(role)
      },
    }),
    {
      name: 'trade-auth-storage',
    }
  )
)
