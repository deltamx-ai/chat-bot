import { create } from 'zustand'

type ThemeMode = 'dark' | 'light' | 'system'

interface ThemeState {
  theme: ThemeMode
  setTheme: (theme: ThemeMode) => void
}

const storageKey = 'alma-theme'

function readInitialTheme(): ThemeMode {
  if (typeof window === 'undefined') return 'system'
  const stored = window.localStorage.getItem(storageKey)
  if (stored === 'dark' || stored === 'light' || stored === 'system') return stored
  return 'system'
}

export const useThemeStore = create<ThemeState>((set) => ({
  theme: readInitialTheme(),
  setTheme: (theme) => {
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(storageKey, theme)
    }
    set({ theme })
  },
}))

export function resolveTheme(theme: ThemeMode): 'dark' | 'light' {
  if (theme !== 'system') return theme
  if (typeof window === 'undefined') return 'dark'
  return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark'
}
