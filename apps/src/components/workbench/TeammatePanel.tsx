import { useState } from 'react'

import {
  createTeammate,
  sendTeammateMessage,
  useTeammateMessages,
  useTeammates,
} from '../../lib/teammateApi'

export function TeammatePanel() {
  const { teammates } = useTeammates()
  const [selectedId, setSelectedId] = useState('')
  const { messages } = useTeammateMessages(selectedId)

  const activeId = selectedId || teammates[0]?.id || ''
  const active = teammates.find((item) => item.id === activeId)

  return (
    <aside className="rounded-2xl border border-white/10 bg-[#090d18] p-4">
      <div className="mb-3 flex items-center justify-between">
        <div className="text-sm font-medium text-white">Teammates</div>
        <button
          type="button"
          onClick={() => {
            const index = teammates.length + 1
            void createTeammate(`reviewer-${index}`, index % 2 === 0 ? 'implementer' : 'reviewer')
          }}
          className="rounded-xl border border-white/10 px-2 py-1 text-xs text-slate-300 transition hover:bg-white/5 hover:text-white"
        >
          + 添加
        </button>
      </div>

      <div className="mb-3 flex flex-wrap gap-2">
        {teammates.map((teammate) => (
          <button
            key={teammate.id}
            type="button"
            onClick={() => setSelectedId(teammate.id)}
            className={`rounded-xl border px-3 py-2 text-xs transition ${
              teammate.id === activeId
                ? 'border-violet-400/40 bg-violet-500/10 text-violet-200'
                : 'border-white/10 bg-white/5 text-slate-300 hover:bg-white/10'
            }`}
          >
            {teammate.name}
          </button>
        ))}
        {teammates.length === 0 ? (
          <div className="rounded-xl border border-dashed border-white/10 px-3 py-2 text-xs text-slate-500">
            还没有 teammate
          </div>
        ) : null}
      </div>

      {active ? (
        <div className="rounded-2xl border border-white/10 bg-black/20 p-3">
          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium text-white">{active.name}</div>
              <div className="mt-1 text-xs text-slate-500">{active.role} · {active.status}</div>
            </div>
            <button
              type="button"
              onClick={() => {
                void sendTeammateMessage(active.id, 'lead', `请检查当前任务状态：${new Date().toLocaleTimeString('zh-CN')}`)
              }}
              className="rounded-xl border border-sky-400/30 bg-sky-500/10 px-3 py-2 text-xs text-sky-200 transition hover:brightness-110"
            >
              ping
            </button>
          </div>
          <div className="mt-3 max-h-40 space-y-2 overflow-y-auto pr-1">
            {messages.map((message) => (
              <div key={message.id} className="rounded-xl border border-white/10 bg-white/5 p-2 text-xs text-slate-300">
                <div className="text-[11px] text-slate-500">{message.from_name} · {message.status}</div>
                <div className="mt-1 whitespace-pre-wrap text-slate-200">{message.content}</div>
              </div>
            ))}
            {messages.length === 0 ? (
              <div className="rounded-xl border border-dashed border-white/10 p-2 text-xs text-slate-500">
                这个 inbox 还是空的
              </div>
            ) : null}
          </div>
        </div>
      ) : null}
    </aside>
  )
}
