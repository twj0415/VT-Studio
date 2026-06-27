# 数据库 Schema 与迁移规范

> 这篇定义 SQLite 落地结构、迁移规则和事务边界。字段含义看 `数据结构.md`，枚举取值看 `01-枚举字典与配置规范.md`。代码实现时不得绕过本规范私建表、私造字段。
>
> 注意：本文包含目标 Schema 和阶段性扩展 Schema。当前 Rust migration 的真实状态必须以 `src-tauri/src/db/mod.rs`、`plan/当前实现审计.md` 和当前 TODO 完成记录为准。文档中标注 TODO-08 的字段或表，在对应 migration 落地前不能被代码当成已存在字段读取。

## 一、核心原则

```text
1. SQLite 是本地权威数据源。
2. 文件本体不进库，库里只存相对路径和元数据。
3. 真实 API Key 不进库，只存 key_alias。
4. 任务状态以 tasks / task_steps 为准，ProgressEvent 只做通知。
5. migration 必须可重复执行、可追踪、不可手改历史迁移。
6. Repository 只做 CRUD，不写业务流程。
7. 所有写入业务主表的操作必须有 updated_at。
```

---

## 二、SQLite 基础设置

应用启动建立连接后必须执行：

```sql
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA busy_timeout = 5000;
PRAGMA synchronous = NORMAL;
```

建议：

```text
1. 默认使用 sqlx migration 管理版本。
2. 所有时间字段用毫秒时间戳 INTEGER。
3. 所有枚举字段用 TEXT，存 snake_case code。
4. JSON 字段用 TEXT 存储序列化 JSON，读取后必须反序列化校验。
5. `source_text` 只保存 20KB 以内短文本；超过 20KB 的文章/长文写入项目 `input/source.txt`，库里存 `source_text_path`。
6. 小说整本原文必须写文件，分章内容写入 `novel_chapters.chapter_content`，不把整本小说塞进 `projects.source_text`。
```

---

## 三、表清单

第一版必须具备：

```text
schema_migrations        sqlx 管理
projects                 项目
project_bibles           项目总设定
style_bibles             画风设定
character_bibles         角色设定
location_bibles          场景/环境设定
storyboards              分镜表
storyboard_items         分镜行
image_candidates         生图候选图
video_segments           图生视频片段
composition_tasks        合成任务
novel_chapters           小说章节，仅小说入口
video_packs              视频包，TODO-08 完整落库；当前未落地时不得假装可用
assets                   统一资产库
asset_references         资产引用关系
tasks                    任务主表
task_steps               任务步骤
task_attempts            步骤尝试记录
artifacts                任务产物索引
providers                Provider 配置元数据
provider_models          模型能力矩阵
workflow_presets         ComfyUI / RunningHub 工作流预设
app_configs              应用配置
prompt_templates         Prompt/Skill 元数据
templates                HTML/封面/字幕模板元数据
histories                历史记录
```

说明：第一版不建 `dictionaries` 表。字典项由 Rust 后端集中维护在 `src-tauri/src/domain/dictionary/builtin.rs`，通过 DictionaryService 对前端提供 `get_dictionary / list_dictionaries`。后续如果需要运营配置，再新增数据库字典表迁移。

---

## 四、项目表

### projects

当前代码的 `projects` 表已经支持主线草稿和输入配置；下列 `active_pack_id / rule_refs_json / executable_refs_json` 是视频包和规则引用的目标扩展字段，完整落地放到 TODO-08。实现前必须新增 migration，并同步 Project DTO、Repository、前端类型和测试。

```sql
CREATE TABLE projects (
  project_id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  workflow_type TEXT NOT NULL DEFAULT 'image_to_video',
  input_type TEXT NOT NULL,
  input_process_mode TEXT NOT NULL,
  input_options_json TEXT NOT NULL DEFAULT '{}',
  source_text TEXT,
  source_text_path TEXT,
  topic TEXT,
  content_category TEXT,
  content_language TEXT NOT NULL,
  tone TEXT,
  aspect_ratio TEXT NOT NULL,
  target_duration_seconds INTEGER NOT NULL,
  target_scene_count INTEGER NOT NULL,
  segment_duration_seconds REAL NOT NULL DEFAULT 4,
  style_prompt TEXT,
  active_pack_id TEXT,
  rule_refs_json TEXT NOT NULL DEFAULT '{}',
  executable_refs_json TEXT NOT NULL DEFAULT '{}',
  project_lifecycle TEXT NOT NULL,
  cover_path TEXT,
  cover_title TEXT,
  cover_template_id TEXT,
  cover_source_item_id TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  deleted_at INTEGER
);
```

索引：

```sql
CREATE INDEX idx_projects_updated_at ON projects(updated_at DESC);
CREATE INDEX idx_projects_lifecycle ON projects(project_lifecycle);
CREATE INDEX idx_projects_workflow_type ON projects(workflow_type);
```

规则：

```text
1. workflow_type 表示制作流程，例如 image_to_video / digital_human / material_edit。
2. input_type 表示内容来源，例如 topic / paste / article / novel / material。
3. project_lifecycle 只表示项目生命周期。
4. 项目是否生成中，从 latest task 读取。
5. deleted_at 非空代表软删除。
6. input_process_mode 必须持久化，不能只由 input_type 临时推导。
7. input_options_json 存入口相关参数，例如 paste 的 split_mode。
8. content_category 只表示内容类别，不承担 workflow_type 职责。
9. 点击“开始创作”立即写入 draft，后续编辑通过 update 接口自动保存到 SQLite。
10. 前端 dirty 状态只用于交互提示，不能替代数据库草稿。
11. 应用重启后项目详情必须能从 SQLite + 相对路径文件恢复。
12. active_pack_id 只保存创建作品时选择的视频包 ID；作品内覆盖不反向修改视频包。
13. rule_refs_json 保存当前作品实际使用的创作规则引用，只存 rule_key / rule_id，不复制 prompt 正文。
14. executable_refs_json 保存当前作品实际使用的 provider_model_id / workflow_preset_id，不保存 Provider 连接和密钥。
```

---

## 五、设定集表

### project_bibles

```sql
CREATE TABLE project_bibles (
  project_id TEXT PRIMARY KEY,
  bible_json TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
);
```

### style_bibles

```sql
CREATE TABLE style_bibles (
  style_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  style_name TEXT NOT NULL,
  style_prompt TEXT NOT NULL,
  negative_prompt TEXT,
  reference_image_path TEXT,
  bible_json TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
);
```

### character_bibles

```sql
CREATE TABLE character_bibles (
  character_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  name TEXT NOT NULL,
  role TEXT,
  visual_prompt TEXT NOT NULL,
  negative_prompt TEXT,
  reference_image_path TEXT,
  seed INTEGER,
  bible_json TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
);
```

### location_bibles

```sql
CREATE TABLE location_bibles (
  location_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  name TEXT NOT NULL,
  location_type TEXT NOT NULL,
  visual_prompt TEXT NOT NULL,
  reference_image_path TEXT,
  bible_json TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
);
```

规则：

```text
1. StoryboardItem 只引用 character_id / location_id，不复制整段 Bible。
2. 任务创建时冻结 Bible 快照到 task.snapshot_json。
3. Bible 更新只影响新任务，不影响已运行任务。
```

---

## 六、分镜表与主线产物表

### storyboards

```sql
CREATE TABLE storyboards (
  storyboard_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  task_id TEXT,
  title TEXT NOT NULL,
  workflow_type TEXT NOT NULL DEFAULT 'image_to_video',
  content_language TEXT NOT NULL,
  aspect_ratio TEXT NOT NULL,
  total_duration_seconds REAL NOT NULL DEFAULT 0,
  item_count INTEGER NOT NULL DEFAULT 0,
  story_roles_json TEXT NOT NULL DEFAULT '[]',
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
);
```

### storyboard_items

```sql
CREATE TABLE storyboard_items (
  item_id TEXT PRIMARY KEY,
  storyboard_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  item_index INTEGER NOT NULL,
  item_role TEXT,

  source_text TEXT,
  narration_text TEXT,
  subtitle_chunks_json TEXT NOT NULL DEFAULT '[]',

  visual_description TEXT,
  characters_json TEXT NOT NULL DEFAULT '[]',
  scene_description TEXT,
  character_ids_json TEXT NOT NULL DEFAULT '[]',
  location_id TEXT,
  props_json TEXT NOT NULL DEFAULT '[]',

  image_prompt TEXT,
  negative_prompt TEXT,
  video_prompt TEXT,
  duration_seconds REAL NOT NULL DEFAULT 0,

  selected_image_id TEXT,
  selected_video_segment_id TEXT,

  shot_size TEXT,
  camera_motion TEXT,
  composition TEXT,
  pace TEXT,
  transition_type TEXT,

  image_status TEXT NOT NULL DEFAULT 'pending',
  video_status TEXT NOT NULL DEFAULT 'pending',
  subtitle_status TEXT NOT NULL DEFAULT 'pending',
  audio_status TEXT NOT NULL DEFAULT 'pending',

  lock_flags_json TEXT NOT NULL DEFAULT '{}',
  retry_count INTEGER NOT NULL DEFAULT 0,
  last_error_json TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,

  FOREIGN KEY(storyboard_id) REFERENCES storyboards(storyboard_id) ON DELETE CASCADE,
  FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE,
  UNIQUE(storyboard_id, item_index)
);
```

### image_candidates

```sql
CREATE TABLE image_candidates (
  image_id TEXT PRIMARY KEY,
  item_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  image_path TEXT NOT NULL,
  prompt TEXT NOT NULL,
  negative_prompt TEXT,
  provider_model_id TEXT,
  workflow_preset_id TEXT,
  status TEXT NOT NULL DEFAULT 'pending',
  selected INTEGER NOT NULL DEFAULT 0,
  retry_count INTEGER NOT NULL DEFAULT 0,
  last_error_json TEXT,
  created_at INTEGER NOT NULL,
  FOREIGN KEY(item_id) REFERENCES storyboard_items(item_id) ON DELETE CASCADE,
  FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
);
```

### video_segments

```sql
CREATE TABLE video_segments (
  segment_id TEXT PRIMARY KEY,
  item_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  input_image_id TEXT NOT NULL,
  video_path TEXT NOT NULL,
  video_prompt TEXT,
  duration_seconds REAL NOT NULL DEFAULT 0,
  provider_model_id TEXT,
  workflow_preset_id TEXT,
  status TEXT NOT NULL DEFAULT 'pending',
  selected INTEGER NOT NULL DEFAULT 0,
  retry_count INTEGER NOT NULL DEFAULT 0,
  last_error_json TEXT,
  created_at INTEGER NOT NULL,
  FOREIGN KEY(item_id) REFERENCES storyboard_items(item_id) ON DELETE CASCADE,
  FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE,
  FOREIGN KEY(input_image_id) REFERENCES image_candidates(image_id)
);
```

### composition_tasks

```sql
CREATE TABLE composition_tasks (
  task_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  segment_ids_json TEXT NOT NULL DEFAULT '[]',
  output_path TEXT,
  status TEXT NOT NULL DEFAULT 'pending',
  progress INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
);
```

索引：

```sql
CREATE INDEX idx_storyboard_items_project_id ON storyboard_items(project_id);
CREATE INDEX idx_storyboard_items_storyboard_index ON storyboard_items(storyboard_id, item_index);
CREATE INDEX idx_image_candidates_item_id ON image_candidates(item_id);
CREATE INDEX idx_video_segments_item_id ON video_segments(item_id);
CREATE INDEX idx_composition_tasks_project_id ON composition_tasks(project_id);
```

规则：

```text
1. item_index 是唯一排序依据，不按文件名排序。
2. lock_flags_json 用于细粒度锁定，不再扩展多个零散布尔字段。
3. 所有 *_path 只存任务目录内相对路径。
4. 每个 storyboard_item 可以有多张 image_candidates，但同一 item 只能选中一张。
5. 每个 storyboard_item 可以有多个 video_segments，但同一 item 只能选中一个。
6. 合成读取 selected_video_segment_id 对应的视频片段。
7. shot_size / camera_motion / transition_type 是高级可选字段，不阻塞当前主线。
```

---

## 七、小说章节表

```sql
CREATE TABLE novel_chapters (
  novel_chapter_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  chapter_index INTEGER NOT NULL,
  volume_title TEXT,
  chapter_title TEXT NOT NULL,
  chapter_content TEXT NOT NULL,
  structured_event_json TEXT,
  event_status TEXT NOT NULL DEFAULT 'pending',
  error_reason TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE,
  UNIQUE(project_id, chapter_index)
);
```

禁止使用参考项目表名 `o_novel` 作为本项目表名。

---

## 八、创作资源配置表

### video_packs

视频包是“默认策略组合”，不等于创作规则正文、模型配置、素材库或模板市场。TODO-08 实现视频包管理时必须落到本表或等价 Repository，不允许只写前端静态数组。

当前实现差异：

```text
1. 当前代码已有“创作资源”入口和创作规则管理，视频包仍主要是页面占位 / 文档目标。
2. video_packs 表、Repository、DTO、引用关系检查是 TODO-08 必做项。
3. 在 video_packs migration 落地前，不能让用户保存视频包，也不能让 Project 依赖不存在的 active_pack_id。
```

```sql
CREATE TABLE video_packs (
  pack_id TEXT PRIMARY KEY,
  source_type TEXT NOT NULL,
  name TEXT NOT NULL,
  description TEXT,
  applicable_input_types_json TEXT NOT NULL DEFAULT '[]',
  content_category TEXT,
  default_tone TEXT,
  default_aspect_ratio TEXT NOT NULL,
  default_duration_seconds INTEGER NOT NULL,
  default_scene_count INTEGER NOT NULL,
  rule_refs_json TEXT NOT NULL DEFAULT '{}',
  recommended_executable_refs_json TEXT NOT NULL DEFAULT '{}',
  asset_refs_json TEXT NOT NULL DEFAULT '[]',
  is_enabled INTEGER NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
```

规则：

```text
1. source_type 只能是 builtin / user；builtin 不可直接编辑，用户修改时复制为 user。
2. rule_refs_json 只保存 rule_key / rule_id，不保存 prompt_body。
3. recommended_executable_refs_json 只保存 provider_model_id 或 workflow_preset_id，不保存 Provider 连接配置。
4. asset_refs_json 只保存素材库引用 ID，不保存素材文件内容。
5. 删除 user 视频包前检查 projects.active_pack_id、projects.rule_refs_json、tasks.snapshot_json 和 task_steps.input_json 是否引用。
6. 保存当前作品配置为新视频包时，只保存引用、默认参数和素材引用，不能保存真实密钥、任务产物目录、本地绝对路径或完整 prompt 历史。
```

索引：

```sql
CREATE INDEX idx_video_packs_source_enabled ON video_packs(source_type, is_enabled);
CREATE INDEX idx_video_packs_category ON video_packs(content_category);
```

---

## 九、资产表

### assets

当前代码实现差异：

```text
当前 `src-tauri/src/db/mod.rs` 已落地的 assets 表使用简化字段：
  asset_id
  kind
  relative_path
  source_kind
  mime_type
  size_bytes
  checksum
  is_builtin
  lifecycle
  metadata_json
  created_at / updated_at

文档中的 asset_kind / media_kind / source_type / width / height / duration_seconds 是目标语义。
TODO-08 8.1 不强制重建 assets 表；优先用现有字段表达：
  asset_kind   → kind
  source_type  → source_kind
  media_kind   → mime_type 或 metadata_json 派生
  width/height/duration/preview/displayName → metadata_json

如果后续确实需要把这些语义升成强类型列，必须新增 migration、DTO、Repository 和回填测试，不能只改页面。
```

```sql
CREATE TABLE assets (
  asset_id TEXT PRIMARY KEY,
  project_id TEXT,
  asset_kind TEXT NOT NULL,
  display_name TEXT NOT NULL,
  relative_path TEXT NOT NULL,
  preview_path TEXT,
  media_kind TEXT NOT NULL,
  source_type TEXT NOT NULL,
  metadata_json TEXT NOT NULL DEFAULT '{}',
  checksum TEXT,
  file_size INTEGER,
  duration_seconds REAL,
  width INTEGER,
  height INTEGER,
  is_builtin INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  deleted_at INTEGER,
  FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
);
```

### asset_references

当前代码实现差异：

```text
当前 asset_references 已落地字段为 reference_id / asset_id / owner_kind / owner_id / usage_kind / created_at，外键 ON DELETE RESTRICT。
TODO-08 8.1 需要补 delete_asset_reference，用“解除引用 → 再删除资产”的流程替代直接级联删除。
```

```sql
CREATE TABLE asset_references (
  reference_id TEXT PRIMARY KEY,
  asset_id TEXT NOT NULL,
  owner_kind TEXT NOT NULL,
  owner_id TEXT NOT NULL,
  usage_kind TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  FOREIGN KEY(asset_id) REFERENCES assets(asset_id) ON DELETE CASCADE
);
```

规则：

```text
1. 删除资产前检查 asset_references。
2. 用户原始文件不删除，只删除工作区副本。
3. 远程 URL 资源必须下载到 assets 或 task 目录后再入库。
```

---

## 十、任务表

### tasks

```sql
CREATE TABLE tasks (
  task_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  task_kind TEXT NOT NULL,
  task_status TEXT NOT NULL,
  current_step_kind TEXT,
  progress REAL NOT NULL DEFAULT 0,
  snapshot_json TEXT NOT NULL,
  last_error_json TEXT,
  result_json TEXT,
  cancel_requested INTEGER NOT NULL DEFAULT 0,
  worker_id TEXT,
  lease_expires_at INTEGER,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  started_at INTEGER,
  finished_at INTEGER,
  FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
);
```

### task_steps

```sql
CREATE TABLE task_steps (
  task_step_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  step_kind TEXT NOT NULL,
  step_status TEXT NOT NULL,
  order_index INTEGER NOT NULL,
  progress REAL NOT NULL DEFAULT 0,
  retry_count INTEGER NOT NULL DEFAULT 0,
  max_attempts INTEGER NOT NULL DEFAULT 3,
  input_json TEXT NOT NULL DEFAULT '{}',
  output_json TEXT NOT NULL DEFAULT '{}',
  error_json TEXT,
  requires_user_confirmation INTEGER NOT NULL DEFAULT 0,
  idempotency_key TEXT NOT NULL,
  started_at INTEGER,
  finished_at INTEGER,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY(task_id) REFERENCES tasks(task_id) ON DELETE CASCADE,
  UNIQUE(task_id, step_kind),
  UNIQUE(idempotency_key)
);
```

### task_attempts

```sql
CREATE TABLE task_attempts (
  task_attempt_id TEXT PRIMARY KEY,
  task_step_id TEXT NOT NULL,
  attempt_index INTEGER NOT NULL,
  status TEXT NOT NULL,
  input_json TEXT NOT NULL DEFAULT '{}',
  output_json TEXT,
  error_json TEXT,
  started_at INTEGER NOT NULL,
  finished_at INTEGER,
  FOREIGN KEY(task_step_id) REFERENCES task_steps(task_step_id) ON DELETE CASCADE,
  UNIQUE(task_step_id, attempt_index)
);
```

索引：

```sql
CREATE INDEX idx_tasks_project_created ON tasks(project_id, created_at DESC);
CREATE INDEX idx_tasks_status_lease ON tasks(task_status, lease_expires_at);
CREATE INDEX idx_task_steps_task_order ON task_steps(task_id, order_index);
```

---

## 十一、产物表

```sql
CREATE TABLE artifacts (
  artifact_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  owner_kind TEXT,
  owner_id TEXT,
  artifact_kind TEXT NOT NULL,
  relative_path TEXT NOT NULL,
  media_kind TEXT NOT NULL,
  metadata_json TEXT NOT NULL DEFAULT '{}',
  created_at INTEGER NOT NULL,
  FOREIGN KEY(task_id) REFERENCES tasks(task_id) ON DELETE CASCADE
);
```

规则：

```text
1. artifacts 是任务产物索引，不替代 ImageCandidate / VideoSegment / CompositionTask 上的当前产物字段。
2. 重跑任务时保留旧 artifact，方便历史追溯。
3. 当前使用的产物路径仍回写到 ImageCandidate / VideoSegment / CompositionTask / TaskResult。
4. 每次生成或替换产物都插入 artifacts；当前资源表只保存当前使用的产物路径，旧 artifact 保留用于历史追溯。
```

---

## 十二、Provider 与配置表

### providers

```sql
CREATE TABLE providers (
  provider_id TEXT PRIMARY KEY,
  provider_kind TEXT NOT NULL,
  vendor TEXT NOT NULL,
  display_name TEXT NOT NULL,
  base_url TEXT,
  auth_type TEXT NOT NULL,
  key_alias TEXT,
  status TEXT NOT NULL,
  is_enabled INTEGER NOT NULL DEFAULT 1,
  config_json TEXT NOT NULL DEFAULT '{}',
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
```

### provider_models

```sql
CREATE TABLE provider_models (
  provider_model_id TEXT PRIMARY KEY,
  provider_id TEXT NOT NULL,
  model_name TEXT NOT NULL,
  provider_kind TEXT NOT NULL,
  ability_types_json TEXT NOT NULL,
  input_modalities_json TEXT NOT NULL DEFAULT '[]',
  output_modalities_json TEXT NOT NULL DEFAULT '[]',
  feature_flags_json TEXT NOT NULL DEFAULT '[]',
  limits_json TEXT NOT NULL DEFAULT '{}',
  is_default INTEGER NOT NULL DEFAULT 0,
  is_enabled INTEGER NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY(provider_id) REFERENCES providers(provider_id) ON DELETE CASCADE,
  UNIQUE(provider_id, model_name)
);
```

### workflow_presets

```sql
CREATE TABLE workflow_presets (
  workflow_preset_id TEXT PRIMARY KEY,
  provider_id TEXT NOT NULL,
  vendor TEXT NOT NULL,
  workflow_key TEXT NOT NULL,
  workflow_id TEXT,
  display_name TEXT NOT NULL,
  workflow_version TEXT NOT NULL,
  ability_types_json TEXT NOT NULL,
  input_modalities_json TEXT NOT NULL,
  output_modalities_json TEXT NOT NULL,
  limits_json TEXT NOT NULL,
  param_schema_json TEXT NOT NULL,
  node_map_json TEXT NOT NULL,
  output_map_json TEXT NOT NULL,
  is_builtin INTEGER NOT NULL DEFAULT 0,
  is_enabled INTEGER NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY(provider_id) REFERENCES providers(provider_id) ON DELETE CASCADE,
  UNIQUE(provider_id, workflow_key, workflow_version)
);
```

### app_configs

```sql
CREATE TABLE app_configs (
  config_key TEXT PRIMARY KEY,
  config_scope TEXT NOT NULL,
  config_json TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 1,
  updated_at INTEGER NOT NULL
);
```

规则：

```text
1. providers.key_alias 只是钥匙串别名。
2. providers.config_json 不允许存真实密钥。
3. provider_models.ability_types_json / input_modalities_json / output_modalities_json / feature_flags_json / limits_json 必须通过 schema 校验。
4. provider_models 只管理 API 模型能力，不存 ComfyUI / RunningHub workflow。
5. workflow_presets 只管理 ComfyUI / RunningHub 工作流预设，不存 API 模型能力。
6. workflow_presets.param_schema_json / node_map_json / output_map_json 必须在保存和执行前校验。
```

---

## 十三、模板和 Prompt 表

```sql
CREATE TABLE prompt_templates (
  prompt_template_id TEXT PRIMARY KEY,
  template_key TEXT NOT NULL,
  version TEXT NOT NULL,
  module_kind TEXT NOT NULL,
  model_kind TEXT,
  content_path TEXT NOT NULL,
  schema_json TEXT,
  is_builtin INTEGER NOT NULL DEFAULT 1,
  is_enabled INTEGER NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  UNIQUE(template_key, version)
);
```

`prompt_templates` 是当前创作规则 / PromptSkill 的元数据落点之一。当前代码也允许以文件系统 `creative-rules/builtin` 和 `creative-rules/user` 管理规则正文；不论采用 DB 还是文件，用户侧统一叫“创作规则”，DTO 字段必须对齐 `CreativeRule`：

```text
rule_id / key / name / module / provider_kind / output_schema / description / source_type / enabled / body / relative_path
```

规则：

```text
1. Vue 页面不得写大段 prompt 正文。
2. builtin 规则不可直接编辑。
3. user 规则保存前必须校验 output_schema，并拒绝疑似密钥。
4. 视频包和 Project 只引用 rule_key / rule_id，不复制 body。
5. 创作规则不能执行 JS / TS / Python，也不能直接写业务表。
```

```sql
CREATE TABLE templates (
  template_id TEXT PRIMARY KEY,
  template_type TEXT NOT NULL,
  display_name TEXT NOT NULL,
  aspect_ratio TEXT NOT NULL,
  entry_path TEXT NOT NULL,
  params_schema_json TEXT NOT NULL DEFAULT '{}',
  version TEXT NOT NULL,
  is_builtin INTEGER NOT NULL DEFAULT 1,
  is_enabled INTEGER NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
```

---

## 十四、历史记录表

```sql
CREATE TABLE histories (
  history_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  task_id TEXT NOT NULL,
  final_video_path TEXT,
  cover_path TEXT,
  final_duration_seconds REAL,
  output_file_size INTEGER,
  metadata_json TEXT NOT NULL DEFAULT '{}',
  created_at INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE,
  FOREIGN KEY(task_id) REFERENCES tasks(task_id) ON DELETE CASCADE
);
```

---

## 十五、迁移规范

迁移文件命名：

```text
YYYYMMDDHHMMSS_describe_change.sql
```

规则：

```text
1. 已提交的 migration 不允许修改，只能新增 migration。
2. 每次 schema 改动必须同步更新本文档。
3. migration 必须包含向前兼容的数据修复逻辑。
4. 禁止在业务代码里偷偷 CREATE/ALTER 表。
5. 应用启动先 migration，再初始化内置字典、模板、Provider 默认项。
```

---

## 十六、事务边界

必须事务化：

```text
1. 创建项目 + 默认 Bible + 默认 Storyboard。
2. 创建任务 + 创建 task_steps + 创建任务目录。
3. Step 成功后回写 StoryboardItem / ImageCandidate / VideoSegment + task_step + artifacts。
4. 删除/归档项目。
5. 导入项目包。
```

禁止：

```text
1. 一半写库，一半写文件成功后不记录补偿策略。
2. Provider 调用放在长事务内。
3. FFmpeg 运行放在长事务内。
```

正确做法：

```text
短事务记录状态 → 执行外部耗时操作 → 短事务回写结果
```

---

## 十七、Repository 规则

```text
1. Repository 只接受已经校验过的 DTO/Model。
2. Repository 不调用 Provider、不读文件、不发事件。
3. Repository 方法名使用 insert/update/get/list/delete/archive。
4. 所有 list 查询必须有 limit。
5. 所有跨项目查询必须带 project_id。
```
