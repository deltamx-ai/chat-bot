# Workbench 界面模块划分

根据参考图，界面建议拆成三栏工作台：

## 左栏：导航与分组
承载：
- 新建会话
- 会话总览入口
- 队列分类（积压/待办/待审查/完成/取消）
- 标签
- 数据源
- 技能
- 自动化
- 设置

建议前端模块：
- `widgets/sidebar-nav`
- `features/queue`
- `features/capability`
- `features/settings`

## 中栏：会话列表
承载：
- 所有会话
- 当前筛选条件
- 会话排序/筛选
- 会话摘要

建议前端模块：
- `widgets/conversation-list`
- `features/conversation`

## 右栏：执行详情
承载：
- 当前 plan
- step 状态
- 审批节点
- 执行日志
- 底部输入区/动作区

建议前端模块：
- `widgets/execution-panel`
- `widgets/composer`
- `features/task`

## 前端推荐 src 结构
```text
src/
├─ app/
├─ pages/
├─ widgets/
├─ features/
├─ entities/
└─ shared/
```

## 设计原则
- 页面结构优先按业务域拆，不按视觉零件堆砌
- `widgets` 放大块区域
- `features` 放业务能力
- `shared/ui` 只放无业务语义组件
