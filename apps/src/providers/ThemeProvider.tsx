import type { PropsWithChildren } from 'react'
import { useEffect } from 'react'

import { resolveTheme, useThemeStore } from '../store/themeStore'

export function ThemeProvider({ children }: PropsWithChildren) {
  const theme = useThemeStore((state) => state.theme)

  useEffect(() => {
    const root = document.documentElement
    const apply = () => {
      const nextTheme = resolveTheme(theme)
      root.dataset.theme = nextTheme
      root.classList.toggle('light', nextTheme === 'light')
      root.classList.toggle('dark', nextTheme === 'dark')
    }

    apply()

    if (theme !== 'system') return

    const mediaQuery = window.matchMedia('(prefers-color-scheme: light)')
    mediaQuery.addEventListener('change', apply)
    return () => mediaQuery.removeEventListener('change', apply)
  }, [theme])

  return children
}
