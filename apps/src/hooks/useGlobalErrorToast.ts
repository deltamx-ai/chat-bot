import { useEffect, useState } from 'react'

export function useGlobalErrorToast() {
  const [message, setMessage] = useState<string | null>(null)

  useEffect(() => {
    const handler = (event: Event) => {
      const customEvent = event as CustomEvent<{ message?: string }>
      setMessage(customEvent.detail?.message ?? '未知错误')
    }

    window.addEventListener('alma:http-error', handler)
    return () => window.removeEventListener('alma:http-error', handler)
  }, [])

  return {
    message,
    clear: () => setMessage(null),
  }
}
