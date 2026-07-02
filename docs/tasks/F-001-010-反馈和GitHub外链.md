# F-001-010 反馈和 GitHub 外链

状态：等待用户确认  
所属菜单：M-001 全局导航与项目入口  
对应功能文档：`docs/features/M-001-全局导航与项目入口.md`  
原则：先确认本文档，再改代码

## 0. 快速理解

```txt
一句话：主界面提供反馈和 GitHub 外链入口，但只能打开 VT Studio 自己的白名单链接。
为什么现在做：参考项目外链全是 Toonflow 旧地址，必须先定清楚不能照搬。
做完后有什么用：用户能从桌面端安全打开外部链接，不会误跳旧品牌地址。
这一步不碰什么：不做版本检查，版本检查归 F-001-011 和 F-002-015。
```

## 1. 本次做什么

```txt
目标：
  对齐 Toonflow 的外链入口位置，但替换为 VT Studio 白名单外链能力。

只做：
  1. 反馈入口。
  2. GitHub 入口。
  3. 外链白名单。
  4. Electron 通过主进程打开外链。
  5. Web 降级 window.open。
  6. 未配置链接时隐藏入口。

不做：
  1. 不保留 Toonflow 腾讯文档反馈链接。
  2. 不保留 Toonflow GitHub 链接。
  3. 不允许打开任意 URL。
  4. 不做更新检查。
```

## 2. 参考项目怎么做

| 参考文件 | 关键逻辑 |
|---|---|
| `Toonflow-web/src/pages/workbench/index.vue` | 左下角反馈和 GitHub 按钮，Electron 用 `toonflow://openurlwithbrowser` |
| `Toonflow-web/src/components/hello.vue` | 欢迎引导里也有 GitHub 外链 |

参考项目关键事实：

```txt
1. 反馈地址指向 Toonflow 的腾讯文档。
2. GitHub 地址指向 `HBAI-Ltd/Toonflow-app`。
3. Electron 下通过 `fetch("toonflow://openurlwithbrowser?url=...")` 打开系统浏览器。
4. Web 下直接 `window.open(url)`。
5. 源码没有看到外链打开失败的明确用户提示。
```

## 3. 用户操作

```txt
入口：
  主布局左下角辅助入口。

按钮/操作：
  1. 点击反馈。
  2. 点击 GitHub。

弹窗/表单：
  无。

成功反馈：
  系统浏览器打开对应链接。

失败反馈：
  链接未配置、链接不在白名单、打开失败时显示提示。
```

## 4. 要做什么功能

### 1. 反馈入口

怎么做：
- 输入：点击反馈。
- 输出：打开反馈地址。
- 写什么数据：不写。
- 状态怎么变：无。
- 异常怎么处理：未配置则隐藏；打开失败提示。
- 限制：只能打开白名单里的 feedback URL。

### 2. GitHub 入口

怎么做：
- 输入：点击 GitHub。
- 输出：打开 GitHub 地址。
- 写什么数据：不写。
- 状态怎么变：无。
- 异常怎么处理：未配置则隐藏；打开失败提示。
- 限制：不能使用 Toonflow 旧仓库地址。

### 3. Electron 外链打开

怎么做：
- 输入：外链 key。
- 输出：主进程调用系统浏览器打开。
- 写什么数据：不写。
- 状态怎么变：无。
- 异常怎么处理：主进程返回 `{ code: 400, data: {}, msg }` 时提示。
- 限制：renderer 只传 key，不直接传任意 URL。

### 4. Web 降级打开

怎么做：
- 输入：外链 key。
- 输出：浏览器 `window.open` 白名单 URL。
- 写什么数据：不写。
- 状态怎么变：无。
- 异常怎么处理：浏览器阻止弹窗时提示。
- 限制：仍然只能从白名单取 URL。

## 5. 数据和状态

```txt
字段：
  externalLinks.feedback
  externalLinks.github
  settingStore.isElectron

接口/能力：
  window.vtStudio.shell.openExternalByKey
  shell.openExternal

数据读写：
  不写业务数据。

任务状态：
  无。

轮询/Socket：
  无。

模型调用：
  无。

删除影响：
  无。
```

## 6. VT Studio 怎么落

```txt
能力名：
  externalLinks

调用链：
  renderer layout -> window.vtStudio.shell.openExternalByKey -> main/ipc -> shell.openExternal

需要新增：
  外链白名单配置
  openExternalByKey IPC
  外链入口显示控制

需要修改：
  主布局左下角辅助入口
```

## 7. 偏差

```txt
和 Toonflow 不同的地方：
  1. 不使用 `toonflow://openurlwithbrowser`。
  2. 不保留 Toonflow 反馈和 GitHub 地址。
  3. renderer 不直接传 URL 给主进程。

原因：
  清理旧品牌，避免任意外链风险。

是否写入 04：
  是，属于外链打开机制偏差。
```

## 8. 验收

```txt
1. 反馈地址未配置时入口隐藏。
2. GitHub 地址未配置时入口隐藏。
3. 配置后点击能打开系统浏览器。
4. 不在白名单的链接打不开。
5. 打开失败有提示。
6. 不出现 Toonflow 腾讯文档链接。
7. 不出现 Toonflow GitHub 链接。
8. renderer 不直接调用 Node shell。
```

## 9. 用户确认点

| 编号 | 确认点 | 专业建议 |
|---|---|---|
| C-F-001-010-001 | 未配置反馈/GitHub 是否隐藏入口 | 隐藏 |
| C-F-001-010-002 | renderer 是否允许传任意 URL | 不允许，只传 key |

## 10. 执行后记录

```txt
改了哪些文件：（完成后填写）
验证结果：（完成后填写）
未完成事项：（完成后填写）
最终结论：（完成后填写）
```

## 11. 最后大白话

```txt
我这次准备怎么做：
1. 放反馈和 GitHub 入口。
2. 只打开我们自己配置好的链接。
3. 没链接就隐藏，旧链接一律不要。

我不会做什么：
1. 不保留 Toonflow 链接。
2. 不允许页面随便打开 URL。
3. 不做版本更新。

确认规则：
用户确认后才执行；未确认前只改文档。
如果本任务有需要用户确认点，或执行中新增确认点，必须先停下来问用户；未确认前不能开始或继续实现。
```
