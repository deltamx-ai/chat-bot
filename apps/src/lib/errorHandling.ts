import { ApiError } from './http'

export interface ErrorWhitelistRule {
  path?: string
  status?: number
  code?: string
}

const errorWhitelist: ErrorWhitelistRule[] = [
  { path: '/api/tasks', status: 404 },
  { path: '/api/tasks/demo/events', status: 404 },
]

export function shouldIgnoreError(error: unknown, path?: string): boolean {
  if (!(error instanceof ApiError)) return false

  return errorWhitelist.some((rule) => {
    const pathMatched = rule.path ? rule.path === path : true
    const statusMatched = rule.status ? rule.status === error.status : true
    const codeMatched = rule.code ? rule.code === error.code : true
    return pathMatched && statusMatched && codeMatched
  })
}

export function reportGlobalError(error: unknown) {
  const message = error instanceof Error ? error.message : '未知请求错误'
  if (typeof window !== 'undefined') {
    window.dispatchEvent(new CustomEvent('alma:http-error', { detail: { message } }))
  }
}
