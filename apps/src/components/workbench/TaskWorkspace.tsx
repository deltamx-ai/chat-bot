import { AuthCard } from './AuthCard'

import type { TaskDto, TaskEventDto } from '../../lib/taskApi'

interface TaskWorkspaceProps {
  task?: TaskDto
  events: TaskEventDto[]
}

export function TaskWorkspace({ task, events }: TaskWorkspaceProps) {
  if (!task) {
    return (
      <section className="rounded-[28px] border border-white/10 bg-[#060913] p-6 text-slate-400">
        <AuthCard />
      </section>
    )
  }

  return (
    <section className="flex min-h-[820px] flex-col rounded-[28px] border border-white/10 bg-[#060913] p-4 text-slate-100 shadow-[0_20px_80px_rgba(0,0,0,0.35)]">
      <header className="mb-4 flex items-center justify-between rounded-2xl border border-white/10 bg-[#090d18] px-4 py-3">
        <div>
          <div className="text-sm text-slate-400">{task.kind}</div>
          <div className="mt-1 text-2xl font-semibold text-white">{task.title}</div>
        </div>
        <div className="flex gap-2 text-slate-400">
          <button className="rounded-xl border border-white/10 px-3 py-2 text-sm transition hover:bg-white/5 hover:text-white">
            导出
          </button>
          <button className="rounded-xl border border-white/10 px-3 py-2 text-sm transition hover:bg-white/5 hover:text-white">
            关闭
          </button>
        </div>
      </header>

      <div className="mb-4 grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
        <div className="rounded-3xl border border-white/10 bg-[#080b14] p-5">
          <div className="mb-4 text-sm text-emerald-300">Plan</div>
          <div className="rounded-3xl border border-white/10 bg-[#05070f] p-6">
            <h3 className="text-[32px] font-semibold text-white">{task.goal}</h3>
            <div className="mt-6 space-y-5 text-lg leading-9 text-slate-200">
              {task.steps.map((step, index) => (
                <div key={step.id} className="flex gap-4">
                  <div className="w-8 text-slate-500">{index + 1}.</div>
                  <div>
                    <div>{step.title}</div>
                    <div className="mt-1 text-sm text-slate-500">
                      {step.action} · {step.toolName} · {step.status}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="mt-6 rounded-2xl border border-white/10 bg-[#090d18] p-4 text-sm text-slate-400">
            事件数：{events.length}
          </div>
        </div>

        <div className="space-y-4">
          <AuthCard />

          <aside className="rounded-2xl border border-white/10 bg-[#090d18] p-4">
            <div className="mb-3 text-sm font-medium text-white">事件</div>
            <div className="space-y-3 text-sm text-slate-300">
              {events.map((event) => (
                <div key={event.id} className="rounded-2xl border border-white/10 bg-black/20 p-3">
                  <div className="font-medium text-white">{event.kind}</div>
                  <div className="mt-1 text-xs text-slate-400">
                    {event.stepId ? `step: ${event.stepId}` : 'task event'}
                  </div>
                </div>
              ))}
              {events.length === 0 ? (
                <div className="rounded-2xl border border-dashed border-white/10 p-3 text-xs text-slate-500">
                  暂时还没有事件
                </div>
              ) : null}
            </div>
          </aside>
        </div>
      </div>

      <div className="mt-auto flex items-center gap-3 px-2 text-sm text-slate-500">
        <span>思考中...</span>
        <span>{events.length}s</span>
      </div>

      <div className="mt-4 rounded-3xl border border-white/10 bg-[#070a12]/90 p-4">
        <div className="mb-4 flex items-center gap-3">
          <button className="rounded-full border border-violet-400/40 bg-violet-500/10 px-4 py-2 text-sm text-violet-200 transition hover:brightness-110">
            执行
          </button>
          <button className="rounded-full border border-white/10 px-4 py-2 text-sm text-slate-300 transition hover:bg-white/5 hover:text-white">
            待办
          </button>
        </div>
        <textarea
          className="min-h-40 w-full resize-none rounded-3xl border border-white/10 bg-[#04060d] px-4 py-4 text-slate-100 outline-none placeholder:text-slate-500 focus:border-violet-400/60"
          placeholder="按 Shift + Return 执行"
        />
      </div>
    </section>
  )
}
