import { useMemo, useState } from 'react'
import { TaskPanel } from '../components/TaskPanel'
import { demoTaskEvents, demoTasks } from '../lib/taskApi'

const queues = [
  { label: '积压', count: 12, active: false },
  { label: '待办', count: 5, active: false },
  { label: '待审查', count: 2, active: true },
  { label: '完成', count: 18, active: false },
  { label: '已取消', count: 1, active: false },
]

const conversations = [
  {
    title: 'Install Codex Claude Skills',
    time: '0秒',
    active: true,
  },
  {
    title: 'Code Clarity Maintenance',
    time: '24秒',
    active: false,
  },
]

const planSteps = [
  '备份现有 skills 到本地快照目录。',
  '同步 Claude skills 到统一 skills 目录。',
  '同步 Codex skills，并处理同名覆盖规则。',
  '清理无关文件，保留模板与资源。',
  '统计安装结果并校验关键目录结构。',
]

const panelClass =
  'rounded-3xl border border-white/10 bg-[#0a0e18]/88 shadow-[0_24px_80px_rgba(0,0,0,0.32)] backdrop-blur-xl'

const selectedTaskDefaultId = demoTasks[0]?.id ?? ''

const ghostButtonClass =
  'rounded-2xl bg-white/6 px-3.5 py-2.5 text-sm text-slate-200 transition hover:-translate-y-0.5 hover:bg-white/10'

function WorkbenchPage() {
  const [selectedTaskId, setSelectedTaskId] = useState(selectedTaskDefaultId)
  const taskEvents = useMemo(() => demoTaskEvents, [])
  return (
    <main className="min-h-screen p-4 text-slate-100 md:p-6">
      <div className="grid min-h-[calc(100vh-2rem)] grid-cols-1 gap-4 xl:grid-cols-[280px_360px_minmax(0,1fr)]">
        <aside className={`${panelClass} p-4`}>
          <button className="mb-5 w-full rounded-2xl bg-indigo-500/15 px-4 py-3 text-left text-sm text-white transition hover:-translate-y-0.5 hover:bg-indigo-500/25">
            ✎ 新建会话
          </button>

          <section className="mb-5 border-b border-white/8 pb-3">
            <div className="mb-2.5 text-xs text-slate-400">会话</div>
            <div className="rounded-xl bg-white/8 px-3 py-2.5 text-sm text-white">所有会话</div>
          </section>

          <section className="mb-5 border-b border-white/8 pb-3">
            <div className="mb-2.5 text-xs text-slate-400">队列</div>
            <div className="space-y-1">
              {queues.map((item) => (
                <div
                  key={item.label}
                  className={`flex items-center justify-between rounded-xl px-3 py-2.5 text-sm transition hover:-translate-y-0.5 hover:bg-white/6 ${
                    item.active ? 'bg-white/8 text-white' : 'text-slate-300'
                  }`}
                >
                  <span>{item.label}</span>
                  <span className="min-w-6 rounded-full bg-white/8 px-2 py-0.5 text-center text-xs">
                    {item.count}
                  </span>
                </div>
              ))}
            </div>
          </section>

          <section className="mb-5 border-b border-white/8 pb-3">
            <div className="mb-2.5 text-xs text-slate-400">数据源</div>
            <div className="space-y-1 text-sm text-slate-300">
              {['API', 'MCP', '本地文件夹'].map((item) => (
                <div
                  key={item}
                  className="rounded-xl px-3 py-2.5 transition hover:-translate-y-0.5 hover:bg-white/6"
                >
                  {item}
                </div>
              ))}
            </div>
          </section>

          <section>
            <div className="mb-2.5 text-xs text-slate-400">能力</div>
            <div className="space-y-1 text-sm text-slate-300">
              {['技能', '自动化', '设置'].map((item) => (
                <div
                  key={item}
                  className="rounded-xl px-3 py-2.5 transition hover:-translate-y-0.5 hover:bg-white/6"
                >
                  {item}
                </div>
              ))}
            </div>
          </section>
        </aside>

        <section className={`${panelClass} p-5`}>
          <div className="mb-4 flex items-center justify-between gap-3">
            <h2 className="m-0 text-2xl font-bold text-white">所有会话</h2>
            <button className={ghostButtonClass}>筛选</button>
          </div>

          <div className="mb-3 text-xs text-slate-400">今天</div>

          <div className="grid gap-2.5">
            {conversations.map((item) => (
              <article
                key={item.title}
                className={`grid grid-cols-[14px_minmax(0,1fr)_auto] items-center gap-3 rounded-2xl border px-3.5 py-3.5 transition hover:-translate-y-0.5 ${
                  item.active
                    ? 'border-violet-400/90 bg-violet-500/10'
                    : 'border-transparent bg-white/3'
                }`}
              >
                <div className="h-2.5 w-2.5 rounded-full border-2 border-slate-300/70" />
                <div className="min-w-0 text-base text-slate-100">{item.title}</div>
                <div className="text-xs text-slate-400">{item.time}</div>
              </article>
            ))}
          </div>
        </section>

        <section className={`${panelClass} p-5`}>
          <div className="mb-4 flex flex-col gap-3 xl:flex-row xl:items-center xl:justify-between">
            <h2 className="m-0 text-2xl font-bold text-white">Install Codex Claude Skills</h2>
            <div className="flex gap-2.5">
              <button className={ghostButtonClass}>分享</button>
              <button className={ghostButtonClass}>关闭</button>
            </div>
          </div>

          <div className="mb-4 flex flex-wrap items-center gap-3">
            <span className="inline-flex rounded-full bg-emerald-400/18 px-2.5 py-1 text-xs font-semibold text-emerald-300">
              Plan
            </span>
            <span className="text-xs text-slate-400">Submit Plan · Request approval · Execute</span>
          </div>

          <section className="rounded-3xl border border-white/10 bg-[#070a12]/74 p-6">
            <h3 className="mb-4 text-3xl font-semibold text-white xl:text-4xl">
              安装 Claude / Codex Skills 到 Pi
            </h3>
            <p className="mb-5 leading-7 text-slate-300">
              我会先按规划把流程跑通，再把执行过程、审批节点和结果统一收在右侧详情面板里。
            </p>

            <ol className="list-decimal space-y-2 pl-5 leading-8 text-slate-100">
              {planSteps.map((step) => (
                <li key={step}>{step}</li>
              ))}
            </ol>

            <div className="mt-5 rounded-2xl bg-white/4 p-4 text-sm leading-6 text-slate-400">
              预计影响：只会修改统一 skills 目录，不动源目录；后续结果会同步到任务状态和执行日志。
            </div>
          </section>

          <div className="my-5 flex justify-end">
            <button className="rounded-2xl bg-gradient-to-br from-indigo-600 to-violet-500 px-5 py-3 text-sm text-white transition hover:-translate-y-0.5 hover:brightness-110">
              计划已批准，请执行
            </button>
          </div>

          <div className="rounded-3xl border border-white/10 bg-[#070a12]/74 p-4">
            <div className="mb-3 flex gap-2.5">
              <button className={ghostButtonClass}>执行</button>
              <button className={ghostButtonClass}>待办</button>
            </div>
            <textarea
              className="min-h-32 w-full resize-y rounded-2xl border border-white/10 bg-white/3 px-4 py-3 text-slate-100 outline-none transition placeholder:text-slate-500 focus:border-violet-400/80"
              placeholder="按 Shift + Return 执行"
              rows={5}
            />
          </div>
        </section>

        <TaskPanel
          tasks={demoTasks}
          selectedTaskId={selectedTaskId}
          events={taskEvents}
          onSelectTask={setSelectedTaskId}
        />
      </div>
    </main>
  )
}

export default WorkbenchPage
