import { mutate } from 'swr'

import { useGetSWR } from '../hooks/useSwrRequest'
import { httpClient } from './http'

export interface ApprovalRequestDto {
  id: string
  task_id: string
  run_id: string
  step_id?: string | null
  kind: string
  status: string
  title: string
  payload: Record<string, unknown>
  decision_note?: string | null
  created_at: string
  resolved_at?: string | null
}

function approvalsKey(): string {
  return '/api/approval-requests'
}

export function useApprovalRequests() {
  const query = useGetSWR<unknown, ApprovalRequestDto[]>(approvalsKey(), {
    fallbackData: [],
    normalize: (payload) => (Array.isArray(payload) ? (payload as ApprovalRequestDto[]) : []),
  })

  return {
    ...query,
    approvals: query.data ?? [],
  }
}

export async function approveApprovalRequest(
  id: string,
  decision_note?: string,
  activeTaskId?: string,
) {
  const result = await httpClient.post<ApprovalRequestDto>(`/api/approval-requests/${id}/approve`, {
    decision_note,
  })
  await mutate(approvalsKey())
  await mutate('/api/tasks')
  if (activeTaskId) {
    await mutate(`/api/tasks/${activeTaskId}/events`)
  }
  return result
}

export async function rejectApprovalRequest(
  id: string,
  decision_note?: string,
  activeTaskId?: string,
) {
  const result = await httpClient.post<ApprovalRequestDto>(`/api/approval-requests/${id}/reject`, {
    decision_note,
  })
  await mutate(approvalsKey())
  await mutate('/api/tasks')
  if (activeTaskId) {
    await mutate(`/api/tasks/${activeTaskId}/events`)
  }
  return result
}
