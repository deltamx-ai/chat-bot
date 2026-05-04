import { useState } from 'react'

import { useBeginCopilotAuth, useCopilotAuthState, type AuthSessionDto } from '../../lib/authApi'

async function copyText(text: string) {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text)
    return true
  }
  return false
}

function openExternal(url: string) {
  window.open(url, '_blank', 'noopener,noreferrer')
}

export function AuthCard() {
  const { authState } = useCopilotAuthState()
  const [copied, setCopied] = useState<string | null>(null)
  const [manualSession, setManualSession] = useState<AuthSessionDto | null>(null)
  const [elapsedMs, setElapsedMs] = useState<number | null>(null)
  const { begin, isSubmitting } = useBeginCopilotAuth()

  const session = manualSession ?? authState?.session ?? null
  const challenge = session?.challenge ?? null

  return (
    <div className="rounded-2xl border border-white/10 bg-[#090d18] p-4">
      <div className="mb-3 flex items-center justify-between">
        <div>
          <div className="text-sm font-medium text-white">Copilot GitHub Auth</div>
          <div className="mt-1 text-xs text-slate-500">真实 GitHub device flow</div>
        </div>
        <button
          type="button"
          disabled={isSubmitting}
          onClick={async () => {
            const result = await begin()
            setElapsedMs(result.elapsed_ms ?? null)
            if (result.session) {
              setManualSession(result.session)
            }
          }}
          className="rounded-xl border border-violet-400/40 bg-violet-500/10 px-3 py-2 text-sm text-violet-200 transition hover:brightness-110 disabled:opacity-60"
        >
          {isSubmitting ? '请求中...' : '发起认证'}
        </button>
      </div>

      {challenge ? (
        <div className="space-y-3 text-sm text-slate-300">
          <div className="rounded-xl border border-white/10 bg-black/20 px-3 py-3">
            <div className="text-xs text-slate-500">User code</div>
            <div className="mt-1 font-mono text-base text-white">{challenge.user_code}</div>
          </div>
          <div className="rounded-xl border border-white/10 bg-black/20 px-3 py-3">
            <div className="text-xs text-slate-500">Verification URL</div>
            <div className="mt-1 break-all text-white">{challenge.verification_uri}</div>
          </div>
          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              onClick={async () => {
                if (await copyText(challenge.user_code)) setCopied('code')
              }}
              className="rounded-xl border border-white/10 px-3 py-2 text-xs text-slate-200 transition hover:bg-white/5"
            >
              复制 Code
            </button>
            <button
              type="button"
              onClick={async () => {
                if (await copyText(challenge.verification_uri)) setCopied('url')
              }}
              className="rounded-xl border border-white/10 px-3 py-2 text-xs text-slate-200 transition hover:bg-white/5"
            >
              复制链接
            </button>
            <button
              type="button"
              onClick={() => openExternal(challenge.verification_uri)}
              className="rounded-xl border border-emerald-400/30 bg-emerald-500/10 px-3 py-2 text-xs text-emerald-200 transition hover:brightness-110"
            >
              打开浏览器
            </button>
          </div>
          <div className="flex items-center gap-3 text-xs text-slate-500">
            {copied ? <span className="text-emerald-300">已复制 {copied}</span> : null}
            {elapsedMs !== null ? <span>获取耗时 {elapsedMs}ms</span> : null}
          </div>
        </div>
      ) : (
        <div className="rounded-xl border border-dashed border-white/10 px-3 py-4 text-xs text-slate-500">
          还没有获取到 device code，点上面的“发起认证”。
        </div>
      )}
    </div>
  )
}
