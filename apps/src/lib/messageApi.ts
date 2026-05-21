import { mutate } from 'swr'

import { useGetSWR } from '../hooks/useSwrRequest'
import { reportGlobalError } from './errorHandling'
import { httpClient } from './http'
import { usePendingOpsStore } from '../store/pendingOpsStore'

export interface ModelConfigDto {
  id: string
  label: string
  provider: string
}

export interface MessageAttachmentDto {
  id: string
  name: string
  kind: string
  mime_type?: string | null
  path?: string | null
  size_bytes?: number | null
}

export interface ConversationDto {
  id: string
  title: string
  summary?: string | null
  status: string
}

export interface MessageDto {
  id: string
  conversation_id: string
  role: string
  content: string
  model_id?: string | null
  attachments: MessageAttachmentDto[]
}

export interface SendMessageRequestDto {
  conversation_id?: string | null
  content: string
  model_id?: string | null
  attachments: MessageAttachmentDto[]
}

export interface SendMessageResponseDto {
  conversation: ConversationDto
  user_message: MessageDto
  assistant_message: MessageDto
}

export interface UploadFilesResponseDto {
  ok: boolean
  files: MessageAttachmentDto[]
}

export const PENDING_USER_PREFIX = 'pending-user-'
export const PENDING_ASSISTANT_PREFIX = 'pending-assistant-'

export function isPendingAssistant(message: MessageDto): boolean {
  return message.id.startsWith(PENDING_ASSISTANT_PREFIX)
}

export function isPendingUser(message: MessageDto): boolean {
  return message.id.startsWith(PENDING_USER_PREFIX)
}

const DEFAULT_CONVERSATION_ID = 'conv_default'

function messagesKey(conversationId: string): string {
  return `/api/conversations/${conversationId}/messages`
}

function placeholderUser(opId: string, conversationId: string, req: SendMessageRequestDto): MessageDto {
  return {
    id: `${PENDING_USER_PREFIX}${opId}`,
    conversation_id: conversationId,
    role: 'User',
    content: req.content,
    model_id: req.model_id ?? null,
    attachments: req.attachments,
  }
}

function placeholderAssistant(opId: string, conversationId: string, req: SendMessageRequestDto): MessageDto {
  return {
    id: `${PENDING_ASSISTANT_PREFIX}${opId}`,
    conversation_id: conversationId,
    role: 'Assistant',
    content: '',
    model_id: req.model_id ?? null,
    attachments: [],
  }
}

function newOpId(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID()
  }
  return `op-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`
}

export function useModels() {
  const query = useGetSWR<unknown, ModelConfigDto[]>('/api/models', {
    fallbackData: [],
    normalize: (payload) => (Array.isArray(payload) ? (payload as ModelConfigDto[]) : []),
  })

  return {
    ...query,
    models: query.data ?? [],
  }
}

export function useConversations() {
  const query = useGetSWR<unknown, ConversationDto[]>('/api/conversations', {
    fallbackData: [],
    normalize: (payload) => (Array.isArray(payload) ? (payload as ConversationDto[]) : []),
  })

  return {
    ...query,
    conversations: query.data ?? [],
  }
}

export function useMessages(conversationId: string) {
  const key = conversationId ? messagesKey(conversationId) : null
  const query = useGetSWR<unknown, MessageDto[]>(key, {
    fallbackData: [],
    normalize: (payload) => (Array.isArray(payload) ? (payload as MessageDto[]) : []),
  })

  return {
    ...query,
    messages: query.data ?? [],
  }
}

export interface SendMessageOptions {
  taskId?: string
}

export interface UseSendMessageResult {
  send: (req: SendMessageRequestDto, options?: SendMessageOptions) => Promise<SendMessageResponseDto>
}

export interface SseEvent {
  event: string
  data: string
}

export async function consumeSseStream(
  body: ReadableStream<Uint8Array>,
  onEvent: (event: SseEvent) => Promise<void> | void,
): Promise<void> {
  const reader = body.getReader()
  const decoder = new TextDecoder('utf-8')
  let buffer = ''
  let currentEvent = ''
  let currentData: string[] = []

  const flush = async () => {
    if (currentData.length === 0 && currentEvent === '') return
    await onEvent({
      event: currentEvent || 'message',
      data: currentData.join('\n'),
    })
    currentEvent = ''
    currentData = []
  }

  while (true) {
    const { value, done } = await reader.read()
    if (done) break
    buffer += decoder.decode(value, { stream: true })

    let newlineIdx: number
    while ((newlineIdx = buffer.indexOf('\n')) >= 0) {
      const line = buffer.slice(0, newlineIdx).replace(/\r$/, '')
      buffer = buffer.slice(newlineIdx + 1)
      if (line === '') {
        await flush()
      } else if (line.startsWith(':')) {
        // comment / keep-alive
      } else if (line.startsWith('event:')) {
        currentEvent = line.slice(6).trim()
      } else if (line.startsWith('data:')) {
        currentData.push(line.slice(5).replace(/^ /, ''))
      }
    }
  }

  if (buffer.length > 0) {
    // tolerate trailing partial line by treating it as a final event boundary
    const line = buffer.replace(/\r$/, '')
    if (line.startsWith('data:')) {
      currentData.push(line.slice(5).replace(/^ /, ''))
    }
  }
  await flush()
}

function safeJsonParse<T>(text: string): T | null {
  try {
    return JSON.parse(text) as T
  } catch {
    return null
  }
}

function isAbortError(error: unknown): boolean {
  if (error instanceof DOMException && error.name === 'AbortError') return true
  if (typeof error === 'object' && error !== null) {
    const named = error as { name?: unknown }
    if (named.name === 'AbortError') return true
  }
  return false
}

export function useSendMessage(): UseSendMessageResult {
  const startOp = usePendingOpsStore((state) => state.start)
  const finishOp = usePendingOpsStore((state) => state.finish)

  const send = async (
    req: SendMessageRequestDto,
    options: SendMessageOptions = {},
  ): Promise<SendMessageResponseDto> => {
    const opId = newOpId()
    const conversationId = req.conversation_id ?? DEFAULT_CONVERSATION_ID
    const key = messagesKey(conversationId)
    const userPlaceholder = placeholderUser(opId, conversationId, req)
    const assistantPlaceholder = placeholderAssistant(opId, conversationId, req)
    const abortController = new AbortController()

    startOp({
      id: opId,
      kind: 'message',
      conversationId,
      taskId: options.taskId,
      startedAt: Date.now(),
      abortController,
    })

    await mutate(
      key,
      (current?: MessageDto[]) => [...(current ?? []), userPlaceholder, assistantPlaceholder],
      { revalidate: false },
    )

    let finalResult: SendMessageResponseDto | null = null
    let streamError: string | null = null

    try {
      const response = await fetch('/api/messages/stream', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Accept: 'text/event-stream',
        },
        body: JSON.stringify(req),
        signal: abortController.signal,
      })

      if (!response.ok || !response.body) {
        const text = await response.text().catch(() => '')
        throw new Error(text || `request failed: ${response.status}`)
      }

      await consumeSseStream(response.body, async (event) => {
        if (event.event === 'delta') {
          const parsed = safeJsonParse<{ content: string }>(event.data)
          if (!parsed || !parsed.content) return
          await mutate(
            key,
            (current?: MessageDto[]) =>
              (current ?? []).map((message) =>
                message.id === assistantPlaceholder.id
                  ? { ...message, content: message.content + parsed.content }
                  : message,
              ),
            { revalidate: false },
          )
        } else if (event.event === 'done') {
          const parsed = safeJsonParse<SendMessageResponseDto>(event.data)
          if (parsed) {
            finalResult = parsed
          }
        } else if (event.event === 'error') {
          const parsed = safeJsonParse<{ error: string }>(event.data)
          streamError = parsed?.error ?? 'unknown stream error'
        }
      })

      if (streamError) {
        throw new Error(streamError)
      }
      if (!finalResult) {
        throw new Error('stream ended without a done event')
      }

      const result: SendMessageResponseDto = finalResult
      const realConversationId = result.conversation.id
      const realKey = messagesKey(realConversationId)

      await mutate(
        realKey,
        (current?: MessageDto[]) => {
          const filtered = (current ?? []).filter(
            (message) =>
              message.id !== userPlaceholder.id && message.id !== assistantPlaceholder.id,
          )
          return [...filtered, result.user_message, result.assistant_message]
        },
        { revalidate: false },
      )

      if (realConversationId !== conversationId) {
        await mutate(
          key,
          (current?: MessageDto[]) =>
            (current ?? []).filter(
              (message) =>
                message.id !== userPlaceholder.id && message.id !== assistantPlaceholder.id,
            ),
          { revalidate: false },
        )
      }

      await mutate('/api/conversations')

      return result
    } catch (error) {
      await mutate(
        key,
        (current?: MessageDto[]) =>
          (current ?? []).filter(
            (message) =>
              message.id !== userPlaceholder.id && message.id !== assistantPlaceholder.id,
          ),
        { revalidate: false },
      )
      if (isAbortError(error)) {
        // user-initiated stop — resync with the persisted server state
        await mutate(key)
      } else {
        reportGlobalError(error)
      }
      throw error
    } finally {
      finishOp(opId)
    }
  }

  return { send }
}

export async function uploadFiles(files: File[]) {
  const formData = new FormData()
  for (const file of files) {
    formData.append('files', file)
  }
  return httpClient.postFormData<UploadFilesResponseDto>('/api/uploads', formData)
}

export async function createConversation(title?: string): Promise<ConversationDto> {
  const body = title === undefined ? undefined : { title }
  const result = await httpClient.post<ConversationDto>('/api/conversations', body)
  await mutate('/api/conversations', (current?: ConversationDto[]) => [...(current ?? []), result], {
    revalidate: false,
  })
  return result
}

export async function renameConversation(id: string, title: string): Promise<ConversationDto> {
  const result = await httpClient.patch<ConversationDto>(
    `/api/conversations/${encodeURIComponent(id)}`,
    { title },
  )
  await mutate(
    '/api/conversations',
    (current?: ConversationDto[]) =>
      (current ?? []).map((conv) => (conv.id === id ? result : conv)),
    { revalidate: false },
  )
  return result
}

export async function deleteConversation(id: string): Promise<void> {
  await httpClient.delete<{ ok: boolean }>(
    `/api/conversations/${encodeURIComponent(id)}`,
  )
  await mutate(
    '/api/conversations',
    (current?: ConversationDto[]) => (current ?? []).filter((conv) => conv.id !== id),
    { revalidate: false },
  )
  await mutate(messagesKey(id), [], { revalidate: false })
}

