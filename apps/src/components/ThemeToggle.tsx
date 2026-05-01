import { useThemeStore } from '../store/themeStore'

const themeOptions = [
  { label: 'Dark', value: 'dark' },
  { label: 'Light', value: 'light' },
  { label: 'System', value: 'system' },
] as const

export function ThemeToggle() {
  const theme = useThemeStore((state) => state.theme)
  const setTheme = useThemeStore((state) => state.setTheme)

  return (
    <div className="inline-flex rounded-full border border-white/10 bg-white/[0.03] p-1 text-xs text-slate-300">
      {themeOptions.map((option) => {
        const active = option.value === theme
        return (
          <button
            key={option.value}
            type="button"
            onClick={() => setTheme(option.value)}
            className={`rounded-full px-3 py-1 transition ${
              active ? 'bg-violet-500 text-white' : 'hover:bg-white/10'
            }`}
          >
            {option.label}
          </button>
        )
      })}
    </div>
  )
}
