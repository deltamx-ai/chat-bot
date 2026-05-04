import { mutate } from 'swr'

import { useGetSWR, usePostSWR } from '../hooks/useSwrRequest'

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
  elapsed_ms?: number
}

function isAuthStateLike(payload: unknown): payload is CopilotAuthStateDto {
  return Boolean(payload && typeof payload === 'object' && 'provider' in payload)
}

export function useCopilotAuthState() {
  const query = useGetSWR<unknown, CopilotAuthStateDto | null>('/api/auth/copilot', {
    fallbackData: null,
    normalize: (payload) => (isAuthStateLike(payload) ? payload : null),
  })

  return {
    ...query,
    authState: query.data,
  }
}

export function useBeginCopilotAuth() {
  const request = usePostSWR<BeginCopilotAuthResultDto>('/api/auth/copilot/begin')

  const begin = async () => {
    const result = await request.post()
    if (result.session) {
      await mutate('/api/auth/copilot', (current: unknown) => {
        if (isAuthStateLike(current)) {
          return {
            ...current,
            session: result.session,
          }
        }
        return {
          provider: {
            id: 'copilot-github',
            kind: 'Copilot',
            enabled: true,
            base_url: result.session.challenge?.verification_uri ?? 'https://github.com/login/device',
            capabilities: ['Authentication', 'Chat'],
          },
          session: result.session,
        }
      }, false)
    }
    return result
  }

  return {
    ...request,
    begin,
  }
}
