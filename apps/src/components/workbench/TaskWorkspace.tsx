import { useEffect, useRef, useState } from 'react'

import { ApprovalPanel } from './ApprovalPanel'
import { AuthCard } from './AuthCard'
import { ComposerPanel } from './ComposerPanel'
import { TeammatePanel } from './TeammatePanel'

import type { TaskDto, TaskEventDto } from '../../lib/taskApi'
import {
  isPendingAssistant,
  isPendingUser,
  type ConversationDto,
  type MessageDto,
  type ModelConfigDto,
  type SendMessageResponseDto,
} from '../../lib/messageApi'
import type { PendingOp } from '../../store/pendingOpsStore'

interface TaskWorkspaceProps {
  task?: TaskDto
  events: TaskEventDto[]
  messages: MessageDto[]
  models: ModelConfigDto[]
  conversationId: string
  conversations: ConversationDto[]
  pendingOps: Record<string, PendingOp>
  onSelectConversation: (conversationId: string) => void
  onCreateConversation: () => Promise<void> | void
  onRenameConversation: (id: string, currentTitle: string) => Promise<void> | void
  onDeleteConversation: (id: string, currentTitle: string) => Promise<void> | void
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
  onRequestRun?: () => Promise<void> | void
  onStopStreaming?: () => void
  isSendingMessage: boolean
}

type MessageRole = 'System' | 'User' | 'Assistant' | 'Tool'

type TimelineTone = {
  dot: string
  border: string
  bg: string
  label: string
}

const ROLE_TONE: Record<MessageRole, string> = {
  System: 'bg-amber-500/10 border-amber-400/20',
  User: 'bg-violet-500/10 border-violet-400/20',
  Assistant: 'bg-white/5 border-white/10',
  Tool: 'bg-sky-500/10 border-sky-400/20',
}

const FALLBACK_TONE = 'bg-black/20 border-white/10'
const PENDING_TONE = 'bg-white/[0.02] border-dashed border-violet-400/30'

function toneFor(message: MessageDto): string {
  if (isPendingAssistant(message) || isPendingUser(message)) {
    return PENDING_TONE
  }
  return (ROLE_TONE as Record<string, string>)[message.role] ?? FALLBACK_TONE
}

function hasPendingForConversation(
  ops: Record<string, PendingOp>,
  conversationId: string,
): boolean {
  return Object.values(ops).some(
    (op) => op.kind === 'message' && op.conversationId === conversationId,
  )
}

function payloadEventName(event: TaskEventDto): string {
  const named = event.payload.event
  return typeof named === 'string' && named.length > 0 ? named : event.kind
}

function toneForEvent(name: string): TimelineTone {
  if (name.includes('failed') || name.includes('rejected')) {
    return {
      dot: 'bg-rose-400',
      border: 'border-rose-400/20',
      bg: 'bg-rose-500/5',
      label: 'text-rose-200',
    }
  }
  if (name.includes('approved') || name.includes('succeeded')) {
    return {
      dot: 'bg-emerald-400',
      border: 'border-emerald-400/20',
      bg: 'bg-emerald-500/5',
      label: 'text-emerald-200',
    }
  }
  if (name.includes('started') || name.includes('resumed') || name.includes('running')) {
    return {
      dot: 'bg-sky-400',
      border: 'border-sky-400/20',
      bg: 'bg-sky-500/5',
      label: 'text-sky-200',
    }
  }
  if (name.includes('approval') || name.includes('blocked')) {
    return {
      dot: 'bg-amber-400',
      border: 'border-amber-400/20',
      bg: 'bg-amber-500/5',
      label: 'text-amber-200',
    }
  }
  return {
    dot: 'bg-violet-400',
    border: 'border-violet-400/20',
    bg: 'bg-violet-500/5',
    label: 'text-violet-200',
  }
}

function eventTitle(event: TaskEventDto): string {
  const name = payloadEventName(event)
  switch (name) {
    case 'run.created':
      return '运行已创建'
    case 'approval.requested':
      return '等待审批'
    case 'approval.approved':
      return '审批已通过'
    case 'approval.rejected':
      return '审批被拒绝'
    case 'run.resumed':
      return '运行恢复'
    case 'run.succeeded':
      return '运行完成'
    case 'run.failed':
      return '运行失败'
    case 'step.ready':
      return '步骤就绪'
    case 'step.started':
      return '步骤开始'
    case 'step.succeeded':
      return '步骤成功'
    case 'step.failed':
      return '步骤失败'
    default:
      return name
  }
}

function eventDescription(event: TaskEventDto): string {
  const payload = event.payload
  const tool = typeof payload.tool === 'string' ? payload.tool : null
  const note = typeof payload.note === 'string' ? payload.note : null
  const error = typeof payload.error === 'string' ? payload.error : null
  const decision = typeof payload.decision === 'string' ? payload.decision : null
  const reason = typeof payload.reason === 'string' ? payload.reason : null

  if (error) return error
  if (note) return note
  if (tool && decision) return `${tool} · ${decision}`
  if (tool) return `工具：${tool}`
  if (decision) return `结果：${decision}`
  if (reason) return `原因：${reason}`
  return event.stepId ? `步骤 ${event.stepId}` : '任务事件'
}

export function TaskWorkspace({
  task,
  events,
  messages,
  models,
  conversationId,
  conversations,
  pendingOps,
  onSelectConversation,
  onCreateConversation,
  onRenameConversation,
  onDeleteConversation,
  onSendMessage,
  onRequestRun,
  onStopStreaming,
  isSendingMessage,
}: TaskWorkspaceProps) {
  const messagesRef = useRef<HTMLDivElement | null>(null)
  const [creating, setCreating] = useState(false)

  useEffect(() => {
    const node = messagesRef.current
    if (!node) return
    node.scrollTo({ top: node.scrollHeight, behavior: 'smooth' })
  }, [messages.length, conversationId])

  const handleCreate = async () => {
    setCreating(true)
    try {
      await onCreateConversation()
    } finally {
      setCreating(false)
    }
  }

  if (!task) {
    return (
      <section className="rounded-[28px] border border-white/10 bg-[#060913] p-6 text-slate-400">
        <AuthCard />
      </section>
    )
  }

  const hasConversations = conversations.length > 0
  const activeConversation = conversations.find((conv) => conv.id === conversationId)
  const displayConversationId = activeConversation?.id ?? conversationId
  const canManageActive = Boolean(activeConversation)

  return (
    <section className="flex min-h-[820px] flex-col rounded-[28px] border border-white/10 bg-[#060913] p-4 text-slate-100 shadow-[0_20px_80px_rgba(0,0,0,0.35)]">
      <header className="mb-4 flex items-center justify-between rounded-2xl border border-white/10 bg-[#090d18] px-4 py-3">
        <div>
          <div className="text-sm text-slate-400">{task.kind}</div>
          <div className="mt-1 text-2xl font-semibold text-white">{task.title}</div>
        </div>
        <div className="flex gap-2 text-slate-400">
          <button
            type="button"
            onClick={() => {
              void onRequestRun?.()
            }}
            className="rounded-xl border border-violet-400/40 bg-violet-500/10 px-3 py-2 text-sm text-violet-200 transition hover:brightness-110"
          >
            发起运行
          </button>
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
          <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <select
                value={displayConversationId}
                onChange={(event) => onSelectConversation(event.target.value)}
                disabled={!hasConversations}
                className="max-w-[260px] rounded-xl border border-white/10 bg-[#090d18] px-3 py-2 text-sm text-slate-100 outline-none transition focus:border-violet-400/60 disabled:opacity-60"
              >
                {hasConversations ? (
                  conversations.map((conv) => {
                    const pending = hasPendingForConversation(pendingOps, conv.id)
                    return (
                      <option key={conv.id} value={conv.id}>
                        {pending ? '● ' : ''}
                        {conv.title || conv.id}
                      </option>
                    )
                  })
                ) : (
                  <option value={displayConversationId}>{displayConversationId}</option>
                )}
              </select>
              <button
                type="button"
                onClick={handleCreate}
                disabled={creating}
                className="rounded-xl border border-violet-400/40 bg-violet-500/10 px-3 py-2 text-sm text-violet-200 transition hover:brightness-110 disabled:opacity-60"
              >
                {creating ? '创建中…' : '+ 新建会话'}
              </button>
              <button
                type="button"
                onClick={() => {
                  if (!activeConversation) return
                  void onRenameConversation(activeConversation.id, activeConversation.title)
                }}
                disabled={!canManageActive}
                title="重命名当前会话"
                className="rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm text-slate-200 transition hover:bg-white/10 disabled:opacity-40"
              >
                ✏️
              </button>
              <button
                type="button"
                onClick={() => {
                  if (!activeConversation) return
                  void onDeleteConversation(activeConversation.id, activeConversation.title)
                }}
                disabled={!canManageActive}
                title="删除当前会话"
                className="rounded-xl border border-rose-400/30 bg-rose-500/5 px-3 py-2 text-sm text-rose-300 transition hover:bg-rose-500/10 disabled:opacity-40"
              >
                🗑️
              </button>
            </div>
            <div className="text-xs text-slate-500">{messages.length} 条消息</div>
          </div>
          <div
            ref={messagesRef}
            className="max-h-[60dvh] space-y-4 overflow-y-auto pr-1"
            role="log"
            aria-live="polite"
          >
            {messages.length === 0 ? (
              <div className="rounded-2xl border border-dashed border-white/10 p-4 text-xs text-slate-500">
                这是一段新的会话，发送一条消息开始吧。
              </div>
            ) : null}
            {messages.map((message) => {
              const pendingAssistant = isPendingAssistant(message)
              const showThinking = pendingAssistant && message.content.length === 0
              return (
                <div
                  key={message.id}
                  className={`rounded-2xl border p-4 ${toneFor(message)}`}
                  aria-busy={pendingAssistant || isPendingUser(message) ? true : undefined}
                >
                  <div className="mb-2 text-xs text-slate-500">
                    {message.role} · {message.model_id ?? '未指定模型'}
                  </div>
                  {showThinking ? (
                    <div className="flex items-center gap-2 text-sm text-slate-400">
                      <span className="inline-flex gap-1">
                        <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-violet-400 [animation-delay:-0.3s]" />
                        <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-violet-400 [animation-delay:-0.15s]" />
                        <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-violet-400" />
                      </span>
                      <span>正在思考…</span>
                    </div>
                  ) : (
                    <div className="text-sm leading-7 text-slate-100 whitespace-pre-wrap">
                      {message.content}
                      {pendingAssistant ? (
                        <span className="ml-1 inline-block h-3 w-2 animate-pulse bg-violet-400/70 align-middle" />
                      ) : null}
                    </div>
                  )}
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
              )
            })}
          </div>

          <div className="mt-6 rounded-2xl border border-white/10 bg-[#090d18] p-4 text-sm text-slate-400">
            当前状态：{task.status} · 步骤 {task.steps.filter((step) => step.status === 'Succeeded').length}/{task.steps.length}
          </div>
        </div>

        <div className="space-y-4">
          <AuthCard />
          <ApprovalPanel activeTaskId={task.id} />
          <TeammatePanel />

          <aside className="rounded-2xl border border-white/10 bg-[#090d18] p-4">
            <div className="mb-3 flex items-center justify-between">
              <div className="text-sm font-medium text-white">执行时间线</div>
              <div className="text-xs text-slate-500">{events.length} 条</div>
            </div>
            <div className="max-h-[40dvh] space-y-3 overflow-y-auto pr-1 text-sm text-slate-300">
              {events.map((event, index) => {
                const name = payloadEventName(event)
                const tone = toneForEvent(name)
                return (
                  <div key={event.id} className="relative pl-6">
                    {index < events.length - 1 ? (
                      <div className="absolute left-[9px] top-5 h-[calc(100%+12px)] w-px bg-white/10" />
                    ) : null}
                    <div className={`absolute left-0 top-1 h-[18px] w-[18px] rounded-full border ${tone.border} ${tone.bg} flex items-center justify-center`}>
                      <span className={`h-2.5 w-2.5 rounded-full ${tone.dot}`} />
                    </div>
                    <div className={`rounded-2xl border ${tone.border} ${tone.bg} p-3`}>
                      <div className="flex items-start justify-between gap-3">
                        <div>
                          <div className={`font-medium ${tone.label}`}>{eventTitle(event)}</div>
                          <div className="mt-1 text-xs text-slate-400">{eventDescription(event)}</div>
                        </div>
                        <div className="text-[10px] uppercase tracking-wide text-slate-500">{event.kind}</div>
                      </div>
                      {event.stepId ? (
                        <div className="mt-2 text-[11px] text-slate-500">step: {event.stepId}</div>
                      ) : null}
                    </div>
                  </div>
                )
              })}
              {events.length === 0 ? (
                <div className="rounded-2xl border border-dashed border-white/10 p-3 text-xs text-slate-500">
                  暂时还没有执行轨迹
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
        onStop={onStopStreaming}
        isSubmitting={isSendingMessage}
      />
    </section>
  )
}
