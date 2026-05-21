import { mutate } from 'swr'

import { useGetSWR, usePostSWR } from '../hooks/useSwrRequest'

export type AuthStateValue =
  | 'Unauthenticated'
  | 'Pending'
  | 'Authenticated'
  | 'Expired'
  | 'Revoked'

export type FlowStatusValue =
  | 'pending'
  | 'authenticated'
  | 'failed'
  | 'cancelled'
  | 'expired'
  | 'unknown'

export interface IdentityDto {
  id: string
  display_name: string
  email?: string | null
  provider: string
}

export interface CopilotProviderDto {
  id: string
  display_name: string
  verification_uri: string
}

export interface CopilotAuthViewDto {
  provider: CopilotProviderDto
  state: AuthStateValue
  identity?: IdentityDto | null
  copilot_token_expires_at?: string | null
}

export interface PublicChallengeDto {
  session_id: string
  user_code: string
  verification_uri: string
  expires_in_seconds: number
  poll_interval_seconds: number
}

export interface BeginCopilotAuthResultDto {
  ok: boolean
  challenge?: PublicChallengeDto
  error?: string
}

export type FlowSnapshotDto =
  | {
      status: 'pending'
      user_code: string
      verification_uri: string
      expires_in_seconds: number
      poll_interval_seconds: number
    }
  | { status: 'authenticated'; identity?: IdentityDto | null }
  | { status: 'failed'; error: string }
  | { status: 'cancelled' }
  | { status: 'expired' }
  | { status: 'unknown' }

const COPILOT_STATE_KEY = '/api/auth/copilot'

function isAuthView(payload: unknown): payload is CopilotAuthViewDto {
  return Boolean(
    payload &&
      typeof payload === 'object' &&
      'provider' in payload &&
      'state' in payload,
  )
}

function isFlowSnapshot(payload: unknown): payload is FlowSnapshotDto {
  return Boolean(payload && typeof payload === 'object' && 'status' in payload)
}

export function useCopilotAuthView() {
  const query = useGetSWR<unknown, CopilotAuthViewDto | null>(COPILOT_STATE_KEY, {
    fallbackData: null,
    normalize: (payload) => (isAuthView(payload) ? payload : null),
  })

  return {
    ...query,
    view: query.data,
  }
}

export function useBeginCopilotAuth() {
  const request = usePostSWR<BeginCopilotAuthResultDto>('/api/auth/copilot/begin')

  const begin = async () => {
    const result = await request.post()
    return result
  }

  return {
    ...request,
    begin,
  }
}

export function refreshCopilotAuthView() {
  return mutate(COPILOT_STATE_KEY)
}

export async function pollCopilotFlow(sessionId: string): Promise<FlowSnapshotDto> {
  const response = await fetch(`/api/auth/copilot/poll/${encodeURIComponent(sessionId)}`)
  if (!response.ok) {
    return { status: 'unknown' }
  }
  const payload: unknown = await response.json()
  return isFlowSnapshot(payload) ? payload : { status: 'unknown' }
}

export async function cancelCopilotFlow(sessionId: string): Promise<boolean> {
  const response = await fetch(`/api/auth/copilot/cancel/${encodeURIComponent(sessionId)}`, {
    method: 'POST',
  })
  return response.ok
}

export async function logoutCopilot(): Promise<boolean> {
  const response = await fetch('/api/auth/copilot/logout', { method: 'POST' })
  if (response.ok) {
    await refreshCopilotAuthView()
  }
  return response.ok
}
