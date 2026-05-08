import { useMemo, useState } from 'react'

import type { MessageAttachmentDto, ModelConfigDto, SendMessageResponseDto } from '../../lib/messageApi'
import { uploadFiles } from '../../lib/messageApi'

interface ComposerPanelProps {
  conversationId: string
  models: ModelConfigDto[]
  onSend: (payload: {
    conversation_id: string
    content: string
    model_id?: string | null
    attachments: MessageAttachmentDto[]
  }) => Promise<SendMessageResponseDto>
  isSubmitting: boolean
}

const ghostButtonClass =
  'rounded-full border border-white/10 bg-white/5 px-3 py-1.5 text-xs font-medium text-slate-300 transition hover:border-violet-400/40 hover:text-white'

export function ComposerPanel({ conversationId, models, onSend, isSubmitting }: ComposerPanelProps) {
  const [content, setContent] = useState('')
  const [selectedModel, setSelectedModel] = useState<string>('gpt-5.5')
  const [attachments, setAttachments] = useState<MessageAttachmentDto[]>([])
  const [uploading, setUploading] = useState(false)

  const canSend = useMemo(
    () => content.trim().length > 0 && !isSubmitting && !uploading,
    [content, isSubmitting, uploading],
  )

  return (
    <section className="rounded-3xl border border-white/10 bg-[#070a12]/74 p-4 transition-colors dark:border-white/10 dark:bg-[#070a12]/74 light:border-slate-200 light:bg-white">
      <div className="mb-3 flex flex-wrap gap-2.5">
        <button className={ghostButtonClass}>执行</button>
        <button className={ghostButtonClass}>待办</button>
        <select
          value={selectedModel}
          onChange={(event) => setSelectedModel(event.target.value)}
          className="rounded-full border border-white/10 bg-white/5 px-3 py-1.5 text-xs text-slate-200 outline-none"
        >
          {models.map((model) => (
            <option key={model.id} value={model.id}>
              {model.label}
            </option>
          ))}
        </select>
        <label className="rounded-full border border-white/10 bg-white/5 px-3 py-1.5 text-xs text-slate-200 transition hover:border-violet-400/40 hover:text-white">
          {uploading ? '上传中...' : '上传附件'}
          <input
            type="file"
            multiple
            className="hidden"
            onChange={async (event) => {
              const files = Array.from(event.target.files ?? [])
              if (files.length === 0) return
              setUploading(true)
              try {
                const result = await uploadFiles(files)
                setAttachments(result.files)
              } finally {
                setUploading(false)
              }
            }}
          />
        </label>
      </div>

      {attachments.length > 0 ? (
        <div className="mb-3 flex flex-wrap gap-2 text-xs text-slate-400">
          {attachments.map((attachment) => (
            <span key={attachment.id} className="rounded-full border border-white/10 px-3 py-1">
              {attachment.name}
            </span>
          ))}
        </div>
      ) : null}

      <textarea
        value={content}
        onChange={(event) => setContent(event.target.value)}
        className="min-h-32 w-full resize-y rounded-2xl border border-white/10 bg-white/3 px-4 py-3 text-slate-100 outline-none transition placeholder:text-slate-500 focus:border-violet-400/80 dark:border-white/10 dark:bg-white/3 dark:text-slate-100 light:border-slate-200 light:bg-slate-50 light:text-slate-900"
        placeholder="按 Shift + Return 执行"
        rows={5}
      />

      <div className="mt-4 flex justify-end">
        <button
          type="button"
          disabled={!canSend}
          onClick={async () => {
            const nextContent = content.trim()
            if (!nextContent) return
            await onSend({
              conversation_id: conversationId,
              content: nextContent,
              model_id: selectedModel,
              attachments,
            })
            setContent('')
            setAttachments([])
          }}
          className="rounded-2xl bg-gradient-to-br from-indigo-600 to-violet-500 px-5 py-3 text-sm text-white transition hover:-translate-y-0.5 hover:brightness-110 disabled:cursor-not-allowed disabled:opacity-50"
        >
          {isSubmitting ? '发送中...' : '发送消息'}
        </button>
      </div>
    </section>
  )
}
