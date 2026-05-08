import { AuthCard } from './AuthCard'
import { ComposerPanel } from './ComposerPanel'

import type { TaskDto, TaskEventDto } from '../../lib/taskApi'
import type { MessageDto, ModelConfigDto, SendMessageResponseDto } from '../../lib/messageApi'

interface TaskWorkspaceProps {
  task?: TaskDto
  events: TaskEventDto[]
  messages: MessageDto[]
  models: ModelConfigDto[]
  conversationId: string
  onSendMessage: (payload: {
    conversation_id: string
    content: string
    model_id?: string | null
    attachments: Array<{
      id: string
      name: string
      kind: string
      mime_type?: string | null
      path?: string | null
      size_bytes?: number | null
    }>
  }) => Promise<SendMessageResponseDto>
  isSendingMessage: boolean
}

export function TaskWorkspace({ task, events, messages, models, conversationId, onSendMessage, isSendingMessage }: TaskWorkspaceProps) {
  if (!task) {
    return (
      <section className="rounded-[28px] border border-white/10 bg-[#060913] p-6 text-slate-400">
        <AuthCard />
      </section>
    )
  }

  return (
    <section className="flex min-h-[820px] flex-col rounded-[28px] border border-white/10 bg-[#060913] p-4 text-slate-100 shadow-[0_20px_80px_rgba(0,0,0,0.35)]">
      <header className="mb-4 flex items-center justify-between rounded-2xl border border-white/10 bg-[#090d18] px-4 py-3">
        <div>
          <div className="text-sm text-slate-400">{task.kind}</div>
          <div className="mt-1 text-2xl font-semibold text-white">{task.title}</div>
        </div>
        <div className="flex gap-2 text-slate-400">
          <button className="rounded-xl border border-white/10 px-3 py-2 text-sm transition hover:bg-white/5 hover:text-white">
            导出
          </button>
          <button className="rounded-xl border border-white/10 px-3 py-2 text-sm transition hover:bg-white/5 hover:text-white">
            关闭
          </button>
        </div>
      </header>

      <div className="mb-4 grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
        <div className="rounded-3xl border border-white/10 bg-[#080b14] p-5">
          <div className="mb-4 text-sm text-emerald-300">Messages</div>
          <div className="space-y-4">
            {messages.map((message) => (
              <div key={message.id} className="rounded-2xl border border-white/10 bg-[#05070f] p-4">
                <div className="mb-2 text-xs text-slate-500">
                  {message.role} · {message.model_id ?? '未指定模型'}
                </div>
                <div className="text-sm leading-7 text-slate-100">{message.content}</div>
                {message.attachments.length > 0 ? (
                  <div className="mt-3 flex flex-wrap gap-2 text-xs text-slate-400">
                    {message.attachments.map((attachment) => (
                      <span key={attachment.id} className="rounded-full border border-white/10 px-3 py-1">
                        {attachment.name}
                      </span>
                    ))}
                  </div>
                ) : null}
              </div>
            ))}
          </div>

          <div className="mt-6 rounded-2xl border border-white/10 bg-[#090d18] p-4 text-sm text-slate-400">
            事件数：{events.length}
          </div>
        </div>

        <div className="space-y-4">
          <AuthCard />

          <aside className="rounded-2xl border border-white/10 bg-[#090d18] p-4">
            <div className="mb-3 text-sm font-medium text-white">事件</div>
            <div className="space-y-3 text-sm text-slate-300">
              {events.map((event) => (
                <div key={event.id} className="rounded-2xl border border-white/10 bg-black/20 p-3">
                  <div className="font-medium text-white">{event.kind}</div>
                  <div className="mt-1 text-xs text-slate-400">
                    {event.stepId ? `step: ${event.stepId}` : 'task event'}
                  </div>
                </div>
              ))}
              {events.length === 0 ? (
                <div className="rounded-2xl border border-dashed border-white/10 p-3 text-xs text-slate-500">
                  暂时还没有事件
                </div>
              ) : null}
            </div>
          </aside>
        </div>
      </div>

      <ComposerPanel
        conversationId={conversationId}
        models={models}
        onSend={onSendMessage}
        isSubmitting={isSendingMessage}
      />
    </section>
  )
}
