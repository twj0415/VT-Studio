# 搭建可运行项目框架

## 功能 ID

```txt
FRAME-001
```

## 目标

搭建 VT Studio 的最小可运行桌面端工程，让用户能先看到基础界面，再开始正式业务功能实现。

本任务不是 Toonflow 具体业务功能，不标记任何业务功能为已完成。

## 开工门槛

| 项目 | 结论 |
|---|---|
| 已读 `00-项目规范.md` | 是 |
| 已读 `02-菜单功能清单.md` | 是 |
| 是否实现具体业务功能 | 否 |

## 技术栈落位

| 层 | 采用方案 |
|---|---|
| 桌面端 | Electron |
| 构建 | electron-vite / Vite |
| 前端 | Vue 3 + TypeScript |
| 样式 | Tailwind CSS + SCSS |
| 组件库 | TDesign Vue Next |
| 状态 | Pinia |
| 路由 | Vue Router |

## 目录落位

```txt
src/
  main/
    app/
    ipc/
    services/
  preload/
  renderer/
    pages/
    components/
    stores/
    router/
    styles/
  shared/
    types/
    constants/
```

## 已完成内容

| 内容 | 说明 |
|---|---|
| Electron 主进程入口 | 创建窗口、加载渲染端、注册 IPC |
| preload API | 只暴露 `window.vtStudio` |
| Vue 渲染端入口 | 挂载 Vue、Pinia、Router、TDesign |
| 基础路由 | 按已确认菜单建立页面入口 |
| 工作台布局 | 左侧菜单、顶部上下文、内容区 |
| 占位页面 | 所有已确认菜单都有基础入口 |
| Tailwind + SCSS | Tailwind 用于布局，SCSS 管主题变量和复杂样式 |

## 对齐说明

| 对齐项 | 结论 |
|---|---|
| 菜单入口 | 按 `02-菜单功能清单.md` 建立基础入口 |
| 业务逻辑 | 未实现，等待单功能 tasks 文档 |
| 数据读写 | 未实现 |
| 文件读写 | 未实现 |
| 任务队列 | 未实现 |
| 模型调用 | 未实现 |
| 剪映导出 | 未实现 |

## 偏差记录

| 偏差点 | 原因 | 是否影响业务语义 |
|---|---|---|
| 当前项目上下文使用占位数据 | 框架阶段还没有项目数据库 | 否，正式做 `F-003-001` 时替换 |
| 设置暂时作为独立路由入口 | 方便框架预览；后续可按统一设置弹窗实现 | 否，正式做设置模块时记录最终形态 |

## 验证记录

待安装依赖后执行：

```txt
pnpm install
pnpm run typecheck
pnpm run build
pnpm run dev
```

实际结果：

```txt
pnpm install：通过
pnpm run typecheck：通过
pnpm run build：通过
pnpm run dev：已启动，开发服务地址 http://localhost:5173/
```
