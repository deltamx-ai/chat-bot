import { mutate } from 'swr'

import { useGetSWR } from '../hooks/useSwrRequest'
import { httpClient } from './http'

export interface TeammateDto {
  id: string
  name: string
  role: string
  status: string
  created_at: string
  updated_at: string
}

export interface TeammateMessageDto {
  id: string
  teammate_id: string
  from_name: string
  content: string
  status: string
  created_at: string
  read_at?: string | null
}

export function useTeammates() {
  const query = useGetSWR<unknown, TeammateDto[]>('/api/teammates', {
    fallbackData: [],
    normalize: (payload) => (Array.isArray(payload) ? (payload as TeammateDto[]) : []),
  })

  return {
    ...query,
    teammates: query.data ?? [],
  }
}

export function useTeammateMessages(teammateId: string) {
  const key = teammateId ? `/api/teammates/${teammateId}/messages` : null
  const query = useGetSWR<unknown, TeammateMessageDto[]>(key, {
    fallbackData: [],
    normalize: (payload) => (Array.isArray(payload) ? (payload as TeammateMessageDto[]) : []),
  })

  return {
    ...query,
    messages: query.data ?? [],
  }
}

export async function createTeammate(name: string, role: string) {
  const result = await httpClient.post<TeammateDto>('/api/teammates', { name, role })
  await mutate('/api/teammates')
  return result
}

export async function sendTeammateMessage(teammateId: string, from_name: string, content: string) {
  const result = await httpClient.post<TeammateMessageDto>(`/api/teammates/${teammateId}/messages`, {
    from_name,
    content,
  })
  await mutate(`/api/teammates/${teammateId}/messages`)
  return result
}
