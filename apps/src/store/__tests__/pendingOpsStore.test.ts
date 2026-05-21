import { beforeEach, describe, expect, it, vi } from 'vitest'

import { usePendingOpsStore } from '../pendingOpsStore'

beforeEach(() => {
  usePendingOpsStore.setState({ ops: {} })
})

describe('pendingOpsStore', () => {
  it('stores ops by id on start and removes on finish', () => {
    const { start, finish } = usePendingOpsStore.getState()
    start({ id: 'op1', kind: 'message', conversationId: 'c1', startedAt: 0 })
    expect(Object.keys(usePendingOpsStore.getState().ops)).toEqual(['op1'])
    finish('op1')
    expect(usePendingOpsStore.getState().ops).toEqual({})
  })

  it('finish is a no-op for unknown ids', () => {
    const before = usePendingOpsStore.getState().ops
    usePendingOpsStore.getState().finish('does-not-exist')
    expect(usePendingOpsStore.getState().ops).toBe(before)
  })

  it('abortByConversation calls abort() only on matching ops', () => {
    const controllerA = new AbortController()
    const controllerB = new AbortController()
    const controllerC = new AbortController()
    const abortA = vi.spyOn(controllerA, 'abort')
    const abortB = vi.spyOn(controllerB, 'abort')
    const abortC = vi.spyOn(controllerC, 'abort')

    const { start, abortByConversation } = usePendingOpsStore.getState()
    start({
      id: 'a',
      kind: 'message',
      conversationId: 'conv-1',
      startedAt: 0,
      abortController: controllerA,
    })
    start({
      id: 'b',
      kind: 'message',
      conversationId: 'conv-1',
      startedAt: 1,
      abortController: controllerB,
    })
    start({
      id: 'c',
      kind: 'message',
      conversationId: 'conv-2',
      startedAt: 2,
      abortController: controllerC,
    })

    abortByConversation('conv-1')

    expect(abortA).toHaveBeenCalledOnce()
    expect(abortB).toHaveBeenCalledOnce()
    expect(abortC).not.toHaveBeenCalled()
  })

  it('abortByConversation does not crash on ops missing an abortController', () => {
    const { start, abortByConversation } = usePendingOpsStore.getState()
    start({ id: 'no-ctrl', kind: 'message', conversationId: 'c1', startedAt: 0 })
    expect(() => abortByConversation('c1')).not.toThrow()
  })

  it('abortByConversation ignores task ops', () => {
    const controller = new AbortController()
    const aborted = vi.spyOn(controller, 'abort')
    const { start, abortByConversation } = usePendingOpsStore.getState()
    start({
      id: 't1',
      kind: 'task',
      conversationId: 'c1',
      taskId: 'task-x',
      startedAt: 0,
      abortController: controller,
    })
    abortByConversation('c1')
    expect(aborted).not.toHaveBeenCalled()
  })
})
