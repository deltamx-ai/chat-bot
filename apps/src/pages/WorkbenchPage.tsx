import { useMemo, useState } from 'react'

import { GlobalErrorToast } from '../components/GlobalErrorToast'
import { TaskPanel } from '../components/TaskPanel'
import { useGlobalErrorToast } from '../hooks/useGlobalErrorToast'
import { useTaskEvents, useTasks } from '../hooks/useTasks'

const selectedTaskDefaultId = ''

const ghostButtonClass =
  'rounded-full border border-white/10 bg-white/5 px-3 py-1.5 text-xs font-medium text-slate-300 transition hover:border-violet-400/40 hover:text-white'

const planSteps = [
  '读取当前项目的 skills 目录与配置',
  '把仓库专用 skill 同步到 Alma skills',
  '验证运行时能正确识别新的 skill',
  '同步更新任务状态、日志与最终说明',
]

function WorkbenchPage() {
  const [selectedTaskId, setSelectedTaskId] = useState(selectedTaskDefaultId)
  const { tasks } = useTasks()
  const activeTaskId = selectedTaskId || tasks[0]?.id || ''
  const { events } = useTaskEvents(activeTaskId)
  const { message, clear } = useGlobalErrorToast()
  const taskEvents = useMemo(() => events, [events])

  return (
    <main className="min-h-screen bg-[#02040a] px-4 py-6 text-slate-100 sm:px-6 lg:px-8">
      <div className="mx-auto flex max-w-[1600px] flex-col gap-4">
        <section className="rounded-[28px] border border-white/10 bg-[#070a12]/74 p-4 shadow-[0_20px_80px_rgba(0,0,0,0.45)] backdrop-blur xl:p-5">
          <header className="mb-5 flex flex-col gap-4 border-b border-white/10 pb-5 lg:flex-row lg:items-center lg:justify-between">
            <div>
              <div className="mb-2 inline-flex items-center rounded-full border border-violet-400/30 bg-violet-500/10 px-3 py-1 text-xs tracking-[0.3em] text-violet-200 uppercase">
                Draft session
              </div>
              <h1 className="text-2xl font-semibold text-white sm:text-3xl">统一 skills 目录到 Alma</h1>
              <p className="mt-2 max-w-3xl text-sm leading-7 text-slate-400 sm:text-base">
                把执行计划、任务状态、运行日志和输入输出放到同一个工作台里，方便在开发前先审阅、批准、再执行。
              </p>
            </div>

            <div className="grid gap-3 sm:grid-cols-3 lg:w-[480px]">
              {[
                { label: '会话', value: '01' },
                { label: '步骤', value: '04' },
                { label: '风险', value: '低' },
              ].map((item) => (
                <div
                  key={item.label}
                  className="rounded-2xl border border-white/10 bg-white/[0.03] p-4 text-center"
                >
                  <div className="text-xs uppercase tracking-[0.25em] text-slate-500">{item.label}</div>
                  <div className="mt-2 text-2xl font-semibold text-white">{item.value}</div>
                </div>
              ))}
            </div>
          </header>

          <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_420px]">
            <section className="rounded-3xl border border-white/10 bg-[#070a12]/78 p-5">
              <div className="mb-4 flex flex-wrap items-center gap-2 text-xs text-slate-400">
                <span className="rounded-full border border-emerald-400/25 bg-emerald-500/10 px-3 py-1 text-emerald-200">
                  Plan ready
                </span>
                <span className="rounded-full border border-white/10 px-3 py-1">来源：项目技能管理</span>
                <span className="rounded-full border border-white/10 px-3 py-1">更新于刚刚</span>
              </div>

              <h2 className="text-lg font-medium text-white">实施概览</h2>
              <p className="mt-3 text-sm leading-7 text-slate-400 sm:text-[15px]">
                当前建议把 skill 文件统一收敛到 Alma 的技能目录，避免仓库副本和运行时目录不同步。整个执行过程、审批节点和结果统一收在右侧详情面板里。
              </p>

              <ol className="mt-5 list-decimal space-y-2 pl-5 leading-8 text-slate-100">
                {planSteps.map((step) => (
                  <li key={step}>{step}</li>
                ))}
              </ol>

              <div className="mt-5 rounded-2xl bg-white/4 p-4 text-sm leading-6 text-slate-400">
                预计影响：只会修改统一 skills 目录，不动源目录；后续结果会同步到任务状态和执行日志。
              </div>
            </section>

            <section className="rounded-3xl border border-white/10 bg-[#070a12]/74 p-4">
              <div className="mb-3 flex gap-2.5">
                <button className={ghostButtonClass}>执行</button>
                <button className={ghostButtonClass}>待办</button>
              </div>
              <textarea
                className="min-h-32 w-full resize-y rounded-2xl border border-white/10 bg-white/3 px-4 py-3 text-slate-100 outline-none transition placeholder:text-slate-500 focus:border-violet-400/80"
                placeholder="按 Shift + Return 执行"
                rows={5}
              />
            </section>
          </div>
        </section>

        <TaskPanel
          tasks={tasks}
          selectedTaskId={activeTaskId}
          events={taskEvents}
          onSelectTask={setSelectedTaskId}
        />
      </div>

      <GlobalErrorToast message={message} onClose={clear} />
    </main>
  )
}

export default WorkbenchPage
