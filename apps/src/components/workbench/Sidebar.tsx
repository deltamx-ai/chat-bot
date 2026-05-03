const sections = [
  {
    title: '所有会话',
    items: ['积压', '待办', '待审查', '完成', '已取消'],
  },
  {
    title: '标签',
    items: ['数据源', 'API', 'MCP', '本地文件夹'],
  },
  {
    title: '技能',
    items: ['自动化', '定时任务', '事件触发', '智能体'],
  },
]

export function Sidebar() {
  return (
    <aside className="flex h-full min-h-[820px] flex-col rounded-[28px] border border-white/10 bg-[#1d2127] p-4 text-slate-200 shadow-[0_20px_80px_rgba(0,0,0,0.35)]">
      <button className="mb-4 flex items-center gap-3 rounded-2xl border border-white/10 bg-[#070b15] px-4 py-3 text-left text-sm font-medium text-white transition hover:border-violet-400/40">
        <span className="text-lg">✎</span>
        <span>新建会话</span>
      </button>

      <div className="mb-4 rounded-2xl bg-white/10 px-4 py-3 text-sm font-medium text-white">所有会话</div>

      <div className="flex-1 space-y-5 overflow-hidden">
        {sections.map((section) => (
          <section key={section.title} className="border-b border-white/10 pb-4 last:border-b-0">
            <div className="mb-3 text-sm text-slate-400">{section.title}</div>
            <div className="space-y-2">
              {section.items.map((item, index) => (
                <button
                  key={item}
                  className={`flex w-full items-center gap-3 rounded-xl px-3 py-2 text-left text-sm transition ${
                    index === 0
                      ? 'bg-white/5 text-white'
                      : 'text-slate-300 hover:bg-white/5 hover:text-white'
                  }`}
                >
                  <span className="h-2 w-2 rounded-full bg-slate-500" />
                  <span>{item}</span>
                </button>
              ))}
            </div>
          </section>
        ))}
      </div>

      <div className="mt-4 space-y-2 border-t border-white/10 pt-4 text-sm text-slate-300">
        <button className="flex w-full items-center gap-3 rounded-xl px-3 py-2 text-left transition hover:bg-white/5 hover:text-white">
          <span>⚙</span>
          <span>设置</span>
        </button>
        <button className="flex w-full items-center gap-3 rounded-xl px-3 py-2 text-left transition hover:bg-white/5 hover:text-white">
          <span>☼</span>
          <span>最新动态</span>
        </button>
      </div>
    </aside>
  )
}
