import { useGetSWR, usePostSWR } from '../hooks/useSwrRequest'
import { httpClient } from './http'

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
  const key = conversationId ? `/api/conversations/${conversationId}/messages` : null
  const query = useGetSWR<unknown, MessageDto[]>(key, {
    fallbackData: [],
    normalize: (payload) => (Array.isArray(payload) ? (payload as MessageDto[]) : []),
  })

  return {
    ...query,
    messages: query.data ?? [],
  }
}

export function useSendMessage() {
  return usePostSWR<SendMessageResponseDto, SendMessageRequestDto>('/api/messages')
}

export async function uploadFiles(files: File[]) {
  const formData = new FormData()
  for (const file of files) {
    formData.append('files', file)
  }
  return httpClient.postFormData<UploadFilesResponseDto>('/api/uploads', formData)
}
