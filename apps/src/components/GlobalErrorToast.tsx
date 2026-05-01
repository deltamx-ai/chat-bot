interface GlobalErrorToastProps {
  message: string | null
  onClose: () => void
}

export function GlobalErrorToast({ message, onClose }: GlobalErrorToastProps) {
  if (!message) return null

  return (
    <div className="fixed bottom-6 right-6 z-50 max-w-sm rounded-2xl border border-red-400/30 bg-red-500/10 px-4 py-3 text-sm text-red-100 shadow-2xl backdrop-blur">
      <div className="flex items-start justify-between gap-4">
        <div>
          <div className="font-medium">请求失败</div>
          <div className="mt-1 text-xs leading-5 text-red-100/80">{message}</div>
        </div>
        <button
          type="button"
          onClick={onClose}
          className="rounded-full border border-white/10 px-2 py-0.5 text-xs text-red-50/90 transition hover:bg-white/10"
        >
          关闭
        </button>
      </div>
    </div>
  )
}
