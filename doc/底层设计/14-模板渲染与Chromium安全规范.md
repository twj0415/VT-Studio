# 模板渲染与 Chromium 安全规范

> 这篇定义 HTML 模板、字幕/封面渲染、Playwright sidecar + 随包 Chromium、安全限制和模板参数 DSL。模板渲染必须走 TemplateRenderer，不允许业务代码直接打开任意 HTML。

## 一、核心原则

```text
1. 模板用于画面帧、字幕、封面和布局渲染。
2. 默认使用 HTML + Playwright sidecar 控制随包 Chromium 截图。
3. 模板资源只能来自 templates/assets/project/task 白名单目录。
4. 模板默认禁止联网。
5. 用户文本进入 HTML 前必须 escape。
6. 模板参数必须按 schema 校验。
```

---

## 二、模板目录

```text
workspace/templates/
  builtin/
    frame/
      vertical_9_16/default.html
      horizontal_16_9/default.html
      square_1_1/default.html
    cover/
    subtitle/
  user/
    frame/
    cover/
    subtitle/
```

任务渲染输出：

```text
workspace/projects/proj_xxx/tasks/task_xxx/rendered/
  scene_001.png
  scene_002.png
```

---

## 三、模板元数据

每个模板必须登记：

```ts
interface TemplateManifest {
  templateId: string
  templateType: TemplateType
  displayName: string
  version: string
  aspectRatio: AspectRatio
  entryPath: string
  viewport: {
    width: number
    height: number
  }
  params: TemplateParamSchema[]
}
```

---

## 四、模板参数 DSL

HTML 中允许声明：

```html
<!-- @param title text default="默认标题" -->
<!-- @param font_size number default=64 min=12 max=120 -->
<!-- @param subtitle_color color default="#ffffff" -->
<!-- @param show_shadow bool default=true -->
<!-- @param position select options="top,center,bottom" default="bottom" -->
```

支持类型以 `TemplateParamType` 为准：

```text
text / number / color / bool / select / image / font / range / json
```

规则：

```text
1. 参数名 snake_case。
2. select 必须有 options 或引用字典。
3. image/font 参数必须走 Asset/StorageService。
4. json 参数只给高级模板用，必须 schema 校验。
```

---

## 五、渲染输入

```ts
interface RenderTemplateRequest {
  templateId: string
  sceneId?: string
  aspectRatio: AspectRatio
  params: Record<string, unknown>
  data: {
    title?: string
    narration?: string
    subtitleChunks?: string[]
    imagePath?: string
    videoFramePath?: string
    characterNames?: string[]
  }
  outputPath: string
}
```

输出：

```ts
interface RenderTemplateResponse {
  renderedFramePath: string
  width: number
  height: number
}
```

---

## 六、Viewport 规则

由 AspectRatio 字典 meta 决定：

```text
vertical_9_16     1080 x 1920
horizontal_16_9   1920 x 1080
square_1_1        1080 x 1080
```

模板不得自己私造画幅 code。

---

## 七、安全策略

Playwright browser context 必须：

```text
1. 禁止外网请求。
2. 禁止访问非白名单 file://。
3. 禁止下载。
4. 禁止弹窗。
5. 禁止读取剪贴板。
6. 禁止持久化浏览器用户数据。
```

允许资源：

```text
templates/
assets/
workspace/projects/proj_xxx/tasks/task_xxx/
workspace/cache/fonts/
```

---

## 八、HTML 注入防护

用户输入包括：

```text
title
narration
subtitle_chunks
cover_title
template text params
```

必须：

```text
1. 作为 textContent 注入，不拼 innerHTML。
2. 如果必须拼 HTML，先 escape。
3. URL 参数必须经过 PathGuard 转为安全资源 URL。
4. 禁止把用户输入拼进 script。
```

---

## 九、字体和资源

字体来源：

```text
内置字体
用户导入字体
系统字体白名单（可选）
```

规则：

```text
1. 字体文件作为 Asset 管理。
2. 模板引用字体必须使用相对资源 URL。
3. 不允许模板从公网加载 Google Fonts 等外链。
```

---

## 十、字幕安全区

模板必须支持：

```text
subtitle_safe_top
subtitle_safe_bottom
subtitle_safe_left
subtitle_safe_right
```

默认竖屏：

```text
bottom safe area >= 240px
left/right >= 72px
```

字幕样式必须支持：

```text
font_size
font_weight
color
stroke_color
stroke_width
shadow
background
position
line_height
max_chars_per_line
```

---

## 十一、浏览器生命周期

TemplateRenderer 负责：

```text
1. 通过 Playwright sidecar 启动随包 Chromium browser。
2. 复用 context/page 池。
3. 渲染超时控制。
4. browser 崩溃自动重启一次。
5. 应用退出时关闭 browser。
```

默认超时：

```text
单帧渲染 30 秒
封面渲染 30 秒
批量渲染按 StoryboardItem 数量累计
```

---

## 十二、预览与缓存

模板预览：

```text
preview_template
→ 使用示例数据
→ 输出 cache/template_preview_xxx.png
```

缓存规则：

```text
1. 相同 template_id + params_hash + data_hash 可以复用预览缓存。
2. 正式任务产物不依赖缓存，必须写入 task/rendered。
```

---

## 十三、错误码

```text
template.not_found
template.param_invalid
template.resource_denied
template.render_failed
template.browser_crashed
template.timeout
template.output_missing
```

---

## 十四、禁止事项

```text
1. 禁止模板联网加载未知资源。
2. 禁止业务代码直接打开用户任意 HTML。
3. 禁止把用户文本作为 innerHTML。
4. 禁止模板读取工作区外文件。
5. 禁止功能模块私造模板参数类型。
```
