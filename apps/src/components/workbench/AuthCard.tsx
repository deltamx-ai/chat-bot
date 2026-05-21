import { useEffect, useRef, useState } from 'react'

import {
  cancelCopilotFlow,
  logoutCopilot,
  pollCopilotFlow,
  refreshCopilotAuthView,
  useBeginCopilotAuth,
  useCopilotAuthView,
  type FlowSnapshotDto,
  type PublicChallengeDto,
} from '../../lib/authApi'

async function copyText(text: string): Promise<boolean> {
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
  const { view } = useCopilotAuthView()
  const { begin, isSubmitting } = useBeginCopilotAuth()
  const [challenge, setChallenge] = useState<PublicChallengeDto | null>(null)
  const [snapshot, setSnapshot] = useState<FlowSnapshotDto | null>(null)
  const [copied, setCopied] = useState<'code' | 'url' | null>(null)
  const [flashError, setFlashError] = useState<string | null>(null)
  const pollTimer = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    return () => {
      if (pollTimer.current) {
        clearTimeout(pollTimer.current)
        pollTimer.current = null
      }
    }
  }, [])

  useEffect(() => {
    if (!challenge) return

    const intervalMs = Math.max(challenge.poll_interval_seconds, 1) * 1000
    let cancelled = false

    const tick = async () => {
      const result = await pollCopilotFlow(challenge.session_id)
      if (cancelled) return
      setSnapshot(result)
      if (result.status === 'authenticated') {
        setChallenge(null)
        await refreshCopilotAuthView()
        return
      }
      if (
        result.status === 'failed' ||
        result.status === 'cancelled' ||
        result.status === 'expired' ||
        result.status === 'unknown'
      ) {
        setChallenge(null)
        return
      }
      pollTimer.current = setTimeout(tick, intervalMs)
    }

    pollTimer.current = setTimeout(tick, intervalMs)

    return () => {
      cancelled = true
      if (pollTimer.current) {
        clearTimeout(pollTimer.current)
        pollTimer.current = null
      }
    }
  }, [challenge])

  const onBegin = async () => {
    setFlashError(null)
    setSnapshot(null)
    const result = await begin()
    if (!result.ok || !result.challenge) {
      setFlashError(result.error ?? '请求失败')
      return
    }
    setChallenge(result.challenge)
  }

  const onCancel = async () => {
    if (!challenge) return
    await cancelCopilotFlow(challenge.session_id)
    setChallenge(null)
    setSnapshot({ status: 'cancelled' })
  }

  const onLogout = async () => {
    await logoutCopilot()
    setChallenge(null)
    setSnapshot(null)
  }

  const isAuthed = view?.state === 'Authenticated'

  return (
    <div className="rounded-2xl border border-white/10 bg-[#090d18] p-4">
      <div className="mb-3 flex items-center justify-between">
        <div>
          <div className="text-sm font-medium text-white">Copilot GitHub Auth</div>
          <div className="mt-1 text-xs text-slate-500">真实 GitHub device flow</div>
        </div>
        {isAuthed ? (
          <button
            type="button"
            onClick={onLogout}
            className="rounded-xl border border-rose-400/40 bg-rose-500/10 px-3 py-2 text-sm text-rose-200 transition hover:brightness-110"
          >
            退出登录
          </button>
        ) : (
          <button
            type="button"
            disabled={isSubmitting || Boolean(challenge)}
            onClick={onBegin}
            className="rounded-xl border border-violet-400/40 bg-violet-500/10 px-3 py-2 text-sm text-violet-200 transition hover:brightness-110 disabled:opacity-60"
          >
            {isSubmitting ? '请求中...' : '发起认证'}
          </button>
        )}
      </div>

      {isAuthed ? (
        <div className="space-y-2 text-sm text-slate-300">
          <div className="rounded-xl border border-emerald-500/20 bg-emerald-500/5 px-3 py-3">
            <div className="text-xs text-emerald-300">已登录</div>
            {view?.identity ? (
              <div className="mt-1 text-white">
                {view.identity.display_name}
                {view.identity.email ? (
                  <span className="ml-2 text-xs text-slate-400">{view.identity.email}</span>
                ) : null}
              </div>
            ) : (
              <div className="mt-1 text-xs text-slate-400">GitHub Copilot 会话已激活</div>
            )}
            {view?.copilot_token_expires_at ? (
              <div className="mt-1 text-xs text-slate-500">
                Token 到期：{new Date(view.copilot_token_expires_at).toLocaleString()}
              </div>
            ) : null}
          </div>
        </div>
      ) : challenge ? (
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
            <button
              type="button"
              onClick={onCancel}
              className="rounded-xl border border-rose-400/30 bg-rose-500/10 px-3 py-2 text-xs text-rose-200 transition hover:brightness-110"
            >
              取消
            </button>
          </div>
          <div className="flex items-center gap-3 text-xs text-slate-500">
            {copied ? <span className="text-emerald-300">已复制 {copied}</span> : null}
            <span>每 {challenge.poll_interval_seconds}s 自动轮询</span>
          </div>
        </div>
      ) : (
        <div className="rounded-xl border border-dashed border-white/10 px-3 py-4 text-xs text-slate-500">
          还没有获取到 device code，点上面的"发起认证"。
        </div>
      )}

      {snapshot && snapshot.status !== 'pending' && snapshot.status !== 'authenticated' ? (
        <div className="mt-3 rounded-xl border border-amber-400/30 bg-amber-500/10 px-3 py-2 text-xs text-amber-200">
          {snapshotMessage(snapshot)}
        </div>
      ) : null}

      {flashError ? (
        <div className="mt-3 rounded-xl border border-rose-400/30 bg-rose-500/10 px-3 py-2 text-xs text-rose-200">
          {flashError}
        </div>
      ) : null}
    </div>
  )
}

function snapshotMessage(snapshot: FlowSnapshotDto): string {
  switch (snapshot.status) {
    case 'expired':
      return 'Device code 已过期，请重新发起认证。'
    case 'cancelled':
      return '认证已取消。'
    case 'failed':
      return `认证失败：${snapshot.error}`
    case 'unknown':
      return '会话已不存在，请重新发起。'
    default:
      return ''
  }
}
