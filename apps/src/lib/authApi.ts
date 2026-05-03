import useSWR from 'swr'

import { httpClient } from './http'

export interface AuthChallengeDto {
  provider_id: string
  auth_url: string
  user_code: string
  device_code: string
  verification_uri: string
  expires_in_seconds: number
  poll_interval_seconds: number
  can_copy_code: boolean
  can_copy_url: boolean
}

export interface AuthSessionDto {
  provider_id: string
  method: string
  state: string
  challenge?: AuthChallengeDto | null
}

export interface CopilotAuthStateDto {
  provider: {
    id: string
    kind: string
    enabled: boolean
    base_url?: string | null
    capabilities: string[]
  }
  session?: AuthSessionDto | null
}

export interface BeginCopilotAuthResultDto {
  ok: boolean
  session?: AuthSessionDto
  error?: string
}

function isAuthStateLike(payload: unknown): payload is CopilotAuthStateDto {
  return Boolean(payload && typeof payload === 'object' && 'provider' in payload)
}

export function useCopilotAuthState() {
  const query = useSWR<unknown>('/api/auth/copilot')
  return {
    ...query,
    authState: isAuthStateLike(query.data) ? query.data : null,
  }
}

export async function beginCopilotAuth() {
  return httpClient.post<BeginCopilotAuthResultDto>('/api/auth/copilot/begin')
}
