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

function WorkbenchPage() {
  return (
    <main className="workbench-shell">
      <aside className="sidebar card-panel">
        <button className="new-chat-btn">✎ 新建会话</button>

        <section className="nav-section">
          <div className="section-title">会话</div>
          <div className="nav-item active">所有会话</div>
        </section>

        <section className="nav-section">
          <div className="section-title">队列</div>
          {queues.map((item) => (
            <div key={item.label} className={`nav-item ${item.active ? 'active' : ''}`}>
              <span>{item.label}</span>
              <span className="badge">{item.count}</span>
            </div>
          ))}
        </section>

        <section className="nav-section">
          <div className="section-title">数据源</div>
          <div className="nav-item">API</div>
          <div className="nav-item">MCP</div>
          <div className="nav-item">本地文件夹</div>
        </section>

        <section className="nav-section">
          <div className="section-title">能力</div>
          <div className="nav-item">技能</div>
          <div className="nav-item">自动化</div>
          <div className="nav-item">设置</div>
        </section>
      </aside>

      <section className="conversation-pane card-panel">
        <div className="pane-header">
          <h2>所有会话</h2>
          <button className="ghost-btn">筛选</button>
        </div>

        <div className="conversation-group-label">今天</div>

        <div className="conversation-list">
          {conversations.map((item) => (
            <article key={item.title} className={`conversation-item ${item.active ? 'active' : ''}`}>
              <div className="conversation-dot" />
              <div className="conversation-main">
                <div className="conversation-title">{item.title}</div>
              </div>
              <div className="conversation-time">{item.time}</div>
            </article>
          ))}
        </div>
      </section>

      <section className="detail-pane card-panel">
        <div className="pane-header detail-header">
          <h2>Install Codex Claude Skills</h2>
          <div className="detail-actions">
            <button className="ghost-btn">分享</button>
            <button className="ghost-btn">关闭</button>
          </div>
        </div>

        <div className="status-line">
          <span className="status-tag success">Plan</span>
          <span className="muted-text">Submit Plan · Request approval · Execute</span>
        </div>

        <section className="content-card">
          <h3>安装 Claude / Codex Skills 到 Pi</h3>
          <p className="lead">
            我会先按规划把流程跑通，再把执行过程、审批节点和结果统一收在右侧详情面板里。
          </p>

          <ol className="plan-list">
            {planSteps.map((step) => (
              <li key={step}>{step}</li>
            ))}
          </ol>

          <div className="impact-box">
            预计影响：只会修改统一 skills 目录，不动源目录；后续结果会同步到任务状态和执行日志。
          </div>
        </section>

        <div className="approval-bar">
          <button className="primary-btn">计划已批准，请执行</button>
        </div>

        <div className="composer-box">
          <div className="composer-toolbar">
            <button className="ghost-btn">执行</button>
            <button className="ghost-btn">待办</button>
          </div>
          <textarea
            className="composer-input"
            placeholder="按 Shift + Return 执行"
            rows={5}
          />
        </div>
      </section>
    </main>
  )
}

export default WorkbenchPage
