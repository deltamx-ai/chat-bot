import { describe, expect, it } from 'vitest'

import { consumeSseStream, type SseEvent } from '../messageApi'

function streamFromString(text: string): ReadableStream<Uint8Array> {
  const encoder = new TextEncoder()
  return new ReadableStream<Uint8Array>({
    start(controller) {
      controller.enqueue(encoder.encode(text))
      controller.close()
    },
  })
}

function streamFromChunks(chunks: string[]): ReadableStream<Uint8Array> {
  const encoder = new TextEncoder()
  let i = 0
  return new ReadableStream<Uint8Array>({
    pull(controller) {
      if (i >= chunks.length) {
        controller.close()
        return
      }
      controller.enqueue(encoder.encode(chunks[i]))
      i += 1
    },
  })
}

async function collect(stream: ReadableStream<Uint8Array>): Promise<SseEvent[]> {
  const events: SseEvent[] = []
  await consumeSseStream(stream, (event) => {
    events.push(event)
  })
  return events
}

describe('consumeSseStream', () => {
  it('emits a single delta event with parsed data', async () => {
    const text = 'event: delta\ndata: {"content":"hello"}\n\n'
    const events = await collect(streamFromString(text))
    expect(events).toEqual([{ event: 'delta', data: '{"content":"hello"}' }])
  })

  it('emits multiple events terminated by blank lines', async () => {
    const text =
      'event: delta\ndata: {"content":"a"}\n\n' +
      'event: delta\ndata: {"content":"b"}\n\n' +
      'event: done\ndata: {"ok":true}\n\n'
    const events = await collect(streamFromString(text))
    expect(events.map((e) => e.event)).toEqual(['delta', 'delta', 'done'])
    expect(events[2].data).toBe('{"ok":true}')
  })

  it('joins multi-line data fields per SSE spec', async () => {
    const text = 'event: delta\ndata: line one\ndata: line two\n\n'
    const events = await collect(streamFromString(text))
    expect(events).toEqual([{ event: 'delta', data: 'line one\nline two' }])
  })

  it('ignores comment lines starting with :', async () => {
    const text = ': keep-alive\nevent: delta\ndata: {"content":"x"}\n\n'
    const events = await collect(streamFromString(text))
    expect(events).toEqual([{ event: 'delta', data: '{"content":"x"}' }])
  })

  it('handles events split across multiple read chunks', async () => {
    const events = await collect(
      streamFromChunks([
        'event: delta\nda',
        'ta: {"content":"split"}',
        '\n\n',
        'event: done\ndata: {"ok":true}\n\n',
      ]),
    )
    expect(events).toEqual([
      { event: 'delta', data: '{"content":"split"}' },
      { event: 'done', data: '{"ok":true}' },
    ])
  })

  it('flushes a trailing event without a blank line at end-of-stream', async () => {
    const text = 'event: delta\ndata: {"content":"tail"}\n'
    const events = await collect(streamFromString(text))
    expect(events).toEqual([{ event: 'delta', data: '{"content":"tail"}' }])
  })

  it('defaults event name to "message" when omitted', async () => {
    const text = 'data: {"content":"plain"}\n\n'
    const events = await collect(streamFromString(text))
    expect(events).toEqual([{ event: 'message', data: '{"content":"plain"}' }])
  })

  it('produces no events for an empty stream', async () => {
    const events = await collect(streamFromString(''))
    expect(events).toEqual([])
  })
})
