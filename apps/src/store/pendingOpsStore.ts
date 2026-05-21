import { create } from 'zustand'

export type PendingOpKind = 'message' | 'task'

export interface PendingOp {
  id: string
  kind: PendingOpKind
  conversationId?: string
  taskId?: string
  startedAt: number
  abortController?: AbortController
}

interface PendingOpsState {
  ops: Record<string, PendingOp>
  start: (op: PendingOp) => void
  finish: (id: string) => void
  abortByConversation: (conversationId: string) => void
}

export const usePendingOpsStore = create<PendingOpsState>((set, get) => ({
  ops: {},
  start: (op) =>
    set((state) => ({
      ops: { ...state.ops, [op.id]: op },
    })),
  finish: (id) =>
    set((state) => {
      if (!state.ops[id]) return state
      const next = { ...state.ops }
      delete next[id]
      return { ops: next }
    }),
  abortByConversation: (conversationId) => {
    const ops = get().ops
    for (const op of Object.values(ops)) {
      if (op.kind === 'message' && op.conversationId === conversationId) {
        op.abortController?.abort()
      }
    }
  },
}))

export function pendingByConversation(
  ops: Record<string, PendingOp>,
  conversationId: string,
): PendingOp[] {
  return Object.values(ops).filter(
    (op) => op.kind === 'message' && op.conversationId === conversationId,
  )
}

export function pendingByTask(
  ops: Record<string, PendingOp>,
  taskId: string,
): PendingOp[] {
  return Object.values(ops).filter((op) => op.kind === 'task' && op.taskId === taskId)
}
