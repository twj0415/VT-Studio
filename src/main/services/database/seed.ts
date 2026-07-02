import { randomUUID } from 'node:crypto';
import { existsSync } from 'node:fs';
import type Database from 'better-sqlite3';
import { getRuntimeDirectories, safeJoin } from '../file-system';
import { logger } from '../logger';
import { getDatabase } from './connection';

// ---------------------------------------------------------------------------
// 1. users (1 条)
// ---------------------------------------------------------------------------

function seedUsers(db: Database.Database, now: number): void {
  const exists = db.prepare<[number], { n: number }>(
    'SELECT COUNT(*) as n FROM users WHERE id = ?',
  ).get(1);
  if (!exists || exists.n === 0) {
    db.prepare<[number, string, string, number, number]>(
      'INSERT INTO users (id, name, password, created_at, updated_at) VALUES (?, ?, ?, ?, ?)',
    ).run(1, 'admin', 'admin123', now, now);
  }
}

// ---------------------------------------------------------------------------
// 2. app_settings (10 条，已存在则跳过)
// ---------------------------------------------------------------------------

const DEFAULT_SETTINGS: Array<{ key: string; value: string }> = [
  // tokenKey 在运行时随机生成（见 seedSettings）
  { key: 'messagesPerSummary',      value: '10' },
  { key: 'shortTermLimit',          value: '5' },
  { key: 'summaryMaxLength',        value: '500' },
  { key: 'summaryLimit',            value: '10' },
  { key: 'ragLimit',                value: '3' },
  { key: 'deepRetrieveSummaryLimit',value: '5' },
  { key: 'modelOnnxFile',           value: JSON.stringify(['all-MiniLM-L6-v2', 'onnx', 'model_fp16.onnx']) },
  { key: 'modelDtype',              value: 'fp16' },
  { key: 'switchAiDevTool',         value: '0' },
];

function seedSettings(db: Database.Database, now: number): void {
  // tokenKey: 已存在则跳过（不覆盖，避免每次启动重置）
  const tokenKeyExists = db.prepare<[], { n: number }>(
    "SELECT COUNT(*) as n FROM app_settings WHERE key = 'tokenKey'",
  ).get();
  if (!tokenKeyExists || tokenKeyExists.n === 0) {
    const tokenKey = randomUUID().replace(/-/g, '').slice(0, 8);
    db.prepare<[string, string, number, number]>(
      'INSERT INTO app_settings (key, value, created_at, updated_at) VALUES (?, ?, ?, ?)',
    ).run('tokenKey', tokenKey, now, now);
  }

  for (const { key, value } of DEFAULT_SETTINGS) {
    const exists = db.prepare<[string], { n: number }>(
      'SELECT COUNT(*) as n FROM app_settings WHERE key = ?',
    ).get(key);
    if (!exists || exists.n === 0) {
      db.prepare<[string, string, number, number]>(
        'INSERT INTO app_settings (key, value, created_at, updated_at) VALUES (?, ?, ?, ?)',
      ).run(key, value, now, now);
    }
  }
}

// ---------------------------------------------------------------------------
// 3. model_vendors (8 条，enabled=0)
// ---------------------------------------------------------------------------

const VENDOR_IDS = [
  'toonflow',
  'deepseek',
  'anthropic',
  'gemini',
  'atlascloud',
  'volcengine',
  'minimax',
  'openai',
  'klingai',
  'vidu',
  'comfyui',
] as const;

function seedVendors(db: Database.Database, now: number): void {
  const stmt = db.prepare<[string, string, string, number, number, number]>(
    'INSERT INTO model_vendors (id, input_values, models, enabled, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)',
  );
  for (const id of VENDOR_IDS) {
    const exists = db.prepare<[string], { n: number }>(
      'SELECT COUNT(*) as n FROM model_vendors WHERE id = ?',
    ).get(id);
    if (!exists || exists.n === 0) {
      stmt.run(id, '{}', '[]', 0, now, now);
    }
  }
}

// ---------------------------------------------------------------------------
// 4. agent_model_configs (17 条)
// ---------------------------------------------------------------------------

interface AgentConfigRow {
  key: string;
  name: string;
  description: string | null;
  temperature: number | null;
  maxOutputTokens: number | null;
  disabled: boolean;
}

const AGENT_CONFIGS: AgentConfigRow[] = [
  // 主 Agent（4 条）
  { key: 'scriptAgent',     name: '剧本Agent', description: null, temperature: null, maxOutputTokens: null, disabled: false },
  { key: 'productionAgent', name: '生产Agent', description: null, temperature: null, maxOutputTokens: null, disabled: false },
  { key: 'universalAi',     name: '通用AI',   description: null, temperature: null, maxOutputTokens: null, disabled: false },
  { key: 'ttsDubbing',      name: 'TTS配音',  description: null, temperature: null, maxOutputTokens: null, disabled: true  },
  // 子 Agent（13 条，temperature=1, maxOutputTokens=0）
  { key: 'scriptAgent:decisionAgent',          name: '剧本Agent:决策层',   description: null, temperature: 1, maxOutputTokens: 0, disabled: false },
  { key: 'scriptAgent:supervisionAgent',       name: '剧本Agent:监督层',   description: null, temperature: 1, maxOutputTokens: 0, disabled: false },
  { key: 'scriptAgent:storySkeletonAgent',     name: '剧本Agent:故事骨架', description: null, temperature: 1, maxOutputTokens: 0, disabled: false },
  { key: 'scriptAgent:adaptationStrategyAgent',name: '剧本Agent:改编策略', description: null, temperature: 1, maxOutputTokens: 0, disabled: false },
  { key: 'scriptAgent:scriptAgent',            name: '剧本Agent:剧本生成', description: null, temperature: 1, maxOutputTokens: 0, disabled: false },
  { key: 'productionAgent:decisionAgent',      name: '生产Agent:决策层',   description: null, temperature: 1, maxOutputTokens: 0, disabled: false },
  { key: 'productionAgent:supervisionAgent',   name: '生产Agent:监督层',   description: null, temperature: 1, maxOutputTokens: 0, disabled: false },
  { key: 'productionAgent:deriveAssetsAgent',  name: '生产Agent:衍生资产', description: null, temperature: 1, maxOutputTokens: 0, disabled: false },
  { key: 'productionAgent:generateAssetsAgent',name: '生产Agent:生成资产', description: null, temperature: 1, maxOutputTokens: 0, disabled: false },
  { key: 'productionAgent:directorPlanAgent',  name: '生产Agent:导演规划', description: null, temperature: 1, maxOutputTokens: 0, disabled: false },
  { key: 'productionAgent:storyboardGenAgent', name: '生产Agent:分镜生成', description: null, temperature: 1, maxOutputTokens: 0, disabled: false },
  { key: 'productionAgent:storyboardPanelAgent',name:'生产Agent:分镜面板', description: null, temperature: 1, maxOutputTokens: 0, disabled: false },
  { key: 'productionAgent:storyboardTableAgent',name:'生产Agent:分镜表格', description: null, temperature: 1, maxOutputTokens: 0, disabled: false },
];

function seedAgentConfigs(db: Database.Database, now: number): void {
  const checkStmt = db.prepare<[string], { n: number }>(
    'SELECT COUNT(*) as n FROM agent_model_configs WHERE key = ?',
  );
  const insertStmt = db.prepare<[string, string, string | null, number | null, number | null, number, number, number]>(
    `INSERT INTO agent_model_configs
       (key, name, description, temperature, max_output_tokens, disabled, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
  );
  for (const cfg of AGENT_CONFIGS) {
    const exists = checkStmt.get(cfg.key);
    if (!exists || exists.n === 0) {
      insertStmt.run(
        cfg.key,
        cfg.name,
        cfg.description,
        cfg.temperature,
        cfg.maxOutputTokens,
        cfg.disabled ? 1 : 0,
        now,
        now,
      );
    }
  }
}

// ---------------------------------------------------------------------------
// 5. prompts (4 条)  — data 内容完整对齐参考项目，不修改
// ---------------------------------------------------------------------------

const EVENT_EXTRACTION_DATA = `# 事件提取指令\n\n你是小说文本分析助手。用户每次提供一个章节的原文，你提取该章的结构化事件信息。\n\n## ⚠️ 输出约束（最高优先级，违反任何一条即为失败）\n\n1. 你的**完整回复**只有一行，以 \`|\` 开头、以 \`|\` 结尾，恰好 7 个字段\n2. 回复的**第一个字符**必须是 \`|\`，**最后一个字符**必须是 \`|\`\n3. \`|\` 之前不许有任何字符——没有引导语、没有解释、没有"根据……"、没有"以下是……"\n4. \`|\` 之后不许有任何字符——没有总结、没有提取说明、没有改编建议\n5. 不输出表头行、分隔线、Markdown 标题、emoji、代码块标记\n\n## 输出格式\n\n\`\`\`\n| 第X章 {章节标题} | {涉及角色} | {核心事件} | {主线关系} | {信息密度} | {预估集长} | {情绪强度} |\n\`\`\`\n\n### 字段规范\n\n| 字段 | 格式要求 | 示例 |\n|------|----------|------|\n| 章节 | \`第X章 {章节标题}\` | \`第1章 职业危机与许愿\` |\n| 涉及角色 | 有实际戏份的角色，顿号分隔 | \`林逸、白有容\` |\n| 核心事件 | 30-60字，必须含动作+结果 | \`林逸因解密风潮事业崩塌，颓废中许愿触发魔法系统绑定\` |\n| 主线关系 | **必须**为 \`强/中/弱（3-8字理由）\` | \`强（动机建立+系统激活）\` |\n| 信息密度 | \`高\` / \`中\` / \`低\` | \`高\` |\n| 预估集长 | **必须**为 \`X秒\`，禁止用分钟 | \`50秒\` |\n| 情绪强度 | 文字标签，\`+\` 连接，禁止星级/数字 | \`转折+悬疑\` |\n\n**主线关系判定**：强＝直接推动主角弧线；中＝补充世界观/人物关系/伏笔；弱＝过渡/气氛。\n\n**预估集长参考**：高密度+高情绪→45-60秒；中→35-45秒；低→25-35秒。\n\n**可用情绪标签**：\`冲突\`、\`恐怖\`、\`情感\`、\`转折\`、\`高潮\`、\`平铺\`、\`喜剧\`、\`悬疑\`、\`情感崩溃\`。\n\n## 输出示例\n\n以下两个示例展示的是**完整回复**——除这一行外没有任何其他内容：\n\n\`\`\`\n| 第1章 职业危机与许愿 | 林逸 | 职业魔术师林逸因解密打假风潮导致事业崩塌，颓废中感慨"如果会魔法就好了"，意外触发神奇魔法系统绑定 | 强（主角动机建立+系统激活） | 高 | 50秒 | 转折+悬疑 |\n\`\`\`\n\`\`\`\n| 第12章 山间小憩 | 凌玄、苏晚卿 | 凌玄与苏晚卿在山间歇脚，苏晚卿回忆幼时往事，两人关系略有缓和但未实质推进 | 弱（气氛过渡） | 低 | 25秒 | 平铺+情感 |\n\`\`\`\n\n## 提取规则\n\n- 忠于原文，不推测、不脑补、不加入原文未出现的情节\n- 角色使用文中主要称呼，保持一致\n- 多条平行事件线时，选对主角影响最大的一条，其余简要带过\n- 对话密集章节，关注对话推动了什么结果，而非复述对话内容`;

const AUDIO_BIND_DATA = `你是一个音色匹配助手。\n你的任务是：根据给定角色资产的名称与描述，从候选音频列表中选出最合适的音色。\n匹配规则：\n1. 优先根据角色性别、年龄、性格等特征与音色描述进行语义匹配；\n2. 同一角色仅可匹配一个音色；\n3. 若候选列表中没有合适的音色，则无需返回 audioId；`;


// ---------------------------------------------------------------------------
// 5 (continued). seedPrompts
// ---------------------------------------------------------------------------
// 长 data 字段抽到独立文件，避免本文件过大
import { SCRIPT_ASSET_EXTRACTION_DATA, VIDEO_PROMPT_GENERATION_DATA } from './seed-prompt-data';

interface PromptRow {
  name: string;
  type: string;
  data: string;
}

// EVENT_EXTRACTION_DATA 和 AUDIO_BIND_DATA 在本文件上方已定义
const PROMPTS: PromptRow[] = [
  { name: '事件提取',     type: 'eventExtraction',        data: EVENT_EXTRACTION_DATA },
  { name: '剧本资产提取', type: 'scriptAssetExtraction',  data: SCRIPT_ASSET_EXTRACTION_DATA },
  { name: '视频提示词生成',type: 'videoPromptGeneration',  data: VIDEO_PROMPT_GENERATION_DATA },
  { name: '音色绑定',     type: 'audioBindPrompt',         data: AUDIO_BIND_DATA },
];

function seedPrompts(db: Database.Database, now: number): void {
  const checkStmt = db.prepare<[string], { n: number }>(
    'SELECT COUNT(*) as n FROM prompts WHERE type = ?',
  );
  const insertStmt = db.prepare<[string, string, string, string, number, number]>(
    'INSERT INTO prompts (name, type, data, use_data, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)',
  );
  for (const p of PROMPTS) {
    const exists = checkStmt.get(p.type);
    if (!exists || exists.n === 0) {
      insertStmt.run(p.name, p.type, p.data, '', now, now);
    }
  }
}

// ---------------------------------------------------------------------------
// 6. skill_list（21 条）+ 7. skill_attributions（14 条）
// ---------------------------------------------------------------------------

interface SkillRow {
  id: string;
  md5: string;
  path: string;
  name: string;
  description: string;
  type: 'main' | 'references';
}

const MAIN_SKILLS: SkillRow[] = [
  { id: '4fb36012e56e395b425569987f5dab0e', md5: 'fca3c269c5f325a65dafa663c9bb9773', path: 'production_agent_decision.md',    name: 'production_agent_decision',    description: '', type: 'main' },
  { id: '017b6338d7aa227cd614ec1fb25fd83e', md5: '2610b80abe4bd048fe61c73adc7388ac', path: 'production_agent_execution.md',   name: 'production_agent_execution',   description: '', type: 'main' },
  { id: 'f03c8e67b61580de9ea5b9d166521b67', md5: 'd41d8cd98f00b204e9800998ecf8427e', path: 'production_agent_supervision.md', name: 'production_agent_supervision', description: '', type: 'main' },
  { id: '50b49d8af5d364665b463c23f6a4d8bb', md5: 'fbba66e0df2426996277b299710c3033', path: 'script_agent_decision.md',         name: 'script_agent_decision',         description: '', type: 'main' },
  { id: '427727727e1095c54b6840cd21382d82', md5: '7e5911242af7233854d533278c6a8ccb', path: 'script_agent_execution.md',        name: 'script_agent_execution',        description: '', type: 'main' },
  { id: '02848fb0dd582fd926502c77ecf9679c', md5: '7a8b6a311b015cd47bf17cc52b935348', path: 'script_agent_supervision.md',      name: 'script_agent_supervision',      description: '', type: 'main' },
  { id: 'a1e818cc03a0b355b239ac1fb0512969', md5: '1fd22029e8047aa30b0dfd703cb837ed', path: 'universal_agent.md',               name: 'universal_agent',               description: '', type: 'main' },
];

const REFERENCE_SKILLS: SkillRow[] = [
  { id: '3e5efec258c8d8e6a39bcef12f8ee058', md5: 'efccb0464cfd472861b49ebf737d4820', path: 'references/event_extract.md',              name: 'event_extract',              description: '专为小说改编短剧设计的文本分析助手，逐章提取涉及角色、核心事件、主线关系、信息密度、预估集长及情绪强度等结构化信息，以Markdown表格形式输出，并附汇总统计，辅助短剧制作的内容规划与时长估算。',                                                                                                                                                    type: 'references' },
  { id: '52c51fa8655f899a1b7aae9b6aad7251', md5: '783678aaab829b34e7c30a414c356bf6', path: 'references/novel_character_extract.md',    name: 'novel_character_extract',    description: '专为小说内容分析设计的角色提取助手，从原文中识别并结构化输出所有重要角色的视觉描述信息，包括外貌、服饰、体态、状态变体等字段，供美术制作和AI角色图生成使用。',                                                                                                                                                                                          type: 'references' },
  { id: '6d46cdca10b2f49e07e515885d1387a0', md5: '10544d12c4ef011e6b3b63a99b8c7fa8', path: 'references/novel_props_extract.md',        name: 'novel_props_extract',        description: '专注于从小说原文中提取道具物品信息的分析助手，能识别武器、法器、药物等各类道具，生成包含外观、材质、尺寸、功能及状态变体的结构化视觉描述表格，供美术制作和AI绘图使用。',                                                                                                                                                                                    type: 'references' },
  { id: '1864df75d1d65f76e275046649ecaef8', md5: '65603aa495a541f54c55b7f30e149f45', path: 'references/novel_scene_extract.md',        name: 'novel_scene_extract',        description: '专注于从小说原文中提取并结构化场景信息的分析助手，可识别各类场景地点，输出包含空间描述、光照氛围、关键陈设、色调基调等字段的标准化场景资产表，用于美术制作和AI绘图的场景概念图生成。',                                                                                                                                                                   type: 'references' },
  { id: '7fbce6f90d7d85496ba9817e9622e640', md5: '830559e8f2cd5d0fa8e6df48a164fe2d', path: 'references/video_dialogue_extract.md',     name: 'video_dialogue_extract',     description: '这是一个专门从视频分镜提示词中提取结构化台词、旁白与音效信息的AI助手配置文档，定义了完整的输出格式（含镜号、角色、台词类型、表演指导等字段）、提取规则及处理流程，用于将视频分镜描述转化为标准化台词表。',                                                                                                                                                   type: 'references' },
  { id: '31fb5c5a1f514ec1e66b4eba9f22d4db', md5: '43e63450efe0c9af8a3a40b036d36cb4', path: 'references/pipeline.md',                  name: 'pipeline',                   description: '面向短剧改编项目的四阶段流水线说明文档，涵盖事件提取、故事骨架、改编策略、剧本编写的串行执行流程，定义了决策层、执行层、监督层的协作规范及派发、审核、修复的交互格式与质量门控标准。',                                                                                                                                                                    type: 'references' },
  { id: '27dc2dfc901de2180227d0269217583a', md5: '7d353be4bab7a794436d9abff2b9c6ee', path: 'references/adaptation_format.md',          name: 'adaptation_format',          description: '本文档规定了改编策略输出的标准格式，包括核心改编原则、删除决策和世界观呈现策略三大模块的书写规范，明确各模块所需涵盖的维度与要素，用于指导竖屏短剧等载体的文学改编工作。',                                                                                                                                                                                   type: 'references' },
  { id: 'd49fa09504fe784a8e6eb102756c6d56', md5: '2ef08a7479f29d74986999ceb02092c8', path: 'references/event_format.md',               name: 'event_format',               description: '本文档规定了影视改编项目中事件表的标准输出格式，包括文件头、事件表格、各字段填写规范（章节、角色、核心事件、主线关系、情绪强度、预估时长）及汇总统计模板，用于指导从原著提取事件并评估改编集数与压缩比的第一阶段工作。',                                                                                                                                        type: 'references' },
  { id: '797906c2ddf0750f050bcdeae23eae3d', md5: 'f5e7fe6db7e05db69d5dc327c4c538f2', path: 'references/script_format.md',             name: 'script_format',             description: '本文档为竖屏短剧剧本的输出格式规范，定义了文件头、节拍结构、分镜脚本、画面描述、台词、转场标注等标准格式要求，并附有时长控制参数与自查清单，供AI视频生成和导演制作使用。',                                                                                                                                                                              type: 'references' },
  { id: '1abd8675c0c3e62b20c0b151d2ec0fb1', md5: 'a587532c737ce15022e1522021f099bb', path: 'references/skeleton_format.md',           name: 'skeleton_format',           description: '本文档定义了故事骨架文件（skeleton.md）的标准化输出格式，涵盖故事核、人物成长隐线、三幕结构、分集决策模板、全局删减记录、付费卡点设计及自查清单，用于指导编剧将章节事件列表转化为结构完整的剧集改编方案。',                                                                                                                                                 type: 'references' },
  { id: '0b7828d7a6ab458a4b201122f08d6c16', md5: '120b3c856f1b2a8a429e11319e8c95fe', path: 'references/quality_criteria.md',          name: 'quality_criteria',          description: '本文档为影视/短剧项目的质量审核标准手册，涵盖事件表、故事骨架、改编策略和剧本四大模块的详细审核规则，规定了格式规范、角色名称统一、时长合理性、画面可执行性及场景氛围一致性等审核要求，用于确保各阶段产出物的内容精确性与制作可行性。', type: 'references' },
  { id: '5c1772b5f9c420d9eae9ca02914ba087', md5: 'c710ab7d237e1f0c5aa3d208e0f5b484', path: 'references/plan.md',                     name: 'plan',                      description: '该文档定义了AI代理生成执行计划的规范，包括任务总览、步骤列表（含编号、名称、详细内容、预期输出及依赖关系）和执行顺序标注，并提供标准回复模板，用于将用户需求拆解为可直接传入子代理工具执行的具体步骤。',                                                                                                                                                   type: 'references' },
  { id: '75a45cf996015ca819582873887ec301', md5: '6045d76873fd58b8b87a914a21a38439', path: 'references/derive_assets_extraction.md',  name: 'derive_assets_extraction',  description: '本文档是一份技术操作指南，说明如何根据剧本内容和已有资产列表，提取每个资产在剧情中出现的不同视觉状态变体（derive），并通过工具函数读取和写入数据，用于后续图片生成参考。',                                                                                                                                                                                type: 'references' },
  { id: 'fce75f69d704c19bebcb356bc1bd6e81', md5: 'a3b3432854970f22949ba47236a6532f', path: 'references/storyboard_generation.md',     name: 'storyboard_generation',     description: '根据剧本和资产列表生成结构化分镜面板的工具指南，涵盖分镜拆分原则、字段填写规范及工具调用流程，用于将剧本转化为含画面描述、镜头语言、台词和AI绘图提示词的分镜数据。',                                                                                                                                                                                     type: 'references' },
];

// ---------------------------------------------------------------------------
// skill_attributions 数据（14 条，来自参考项目 initDB.ts 第 948-1005 行）
// ---------------------------------------------------------------------------

interface SkillAttributionRow {
  skillId: string;
  attribution: string;
}

const SKILL_ATTRIBUTIONS: SkillAttributionRow[] = [
  { skillId: '52c51fa8655f899a1b7aae9b6aad7251', attribution: 'universal_agent.md' },
  { skillId: '6d46cdca10b2f49e07e515885d1387a0', attribution: 'universal_agent.md' },
  { skillId: '1864df75d1d65f76e275046649ecaef8', attribution: 'universal_agent.md' },
  { skillId: '3e5efec258c8d8e6a39bcef12f8ee058', attribution: 'universal_agent.md' },
  { skillId: '7fbce6f90d7d85496ba9817e9622e640', attribution: 'universal_agent.md' },
  { skillId: '31fb5c5a1f514ec1e66b4eba9f22d4db', attribution: 'script_agent_decision.md' },
  { skillId: '27dc2dfc901de2180227d0269217583a', attribution: 'script_agent_execution.md' },
  { skillId: 'd49fa09504fe784a8e6eb102756c6d56', attribution: 'script_agent_execution.md' },
  { skillId: '797906c2ddf0750f050bcdeae23eae3d', attribution: 'script_agent_execution.md' },
  { skillId: '1abd8675c0c3e62b20c0b151d2ec0fb1', attribution: 'script_agent_execution.md' },
  { skillId: '0b7828d7a6ab458a4b201122f08d6c16', attribution: 'script_agent_supervision.md' },
  { skillId: '5c1772b5f9c420d9eae9ca02914ba087', attribution: 'production_agent_decision.md' },
  { skillId: '75a45cf996015ca819582873887ec301', attribution: 'production_agent_execution.md' },
  { skillId: 'fce75f69d704c19bebcb356bc1bd6e81', attribution: 'production_agent_execution.md' },
];

// ---------------------------------------------------------------------------
// skill seeding functions
// ---------------------------------------------------------------------------

function seedSkills(db: Database.Database, now: number): void {
  const checkStmt = db.prepare<[string], { n: number }>(
    'SELECT COUNT(*) as n FROM skill_list WHERE id = ?',
  );
  const insertStmt = db.prepare<[string, string, string, string, string, string, string, number, number, number]>(
    `INSERT INTO skill_list
       (id, md5, path, name, description, embedding, type, created_at, updated_at, state)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
  );

  // main skill: description 为空，embedding 为空，state=-1（直接插入，不调 embedding）
  for (const skill of MAIN_SKILLS) {
    const exists = checkStmt.get(skill.id);
    if (!exists || exists.n === 0) {
      insertStmt.run(skill.id, skill.md5, skill.path, skill.name, '', '', skill.type, now, now, -1);
    }
  }

  // references skill: 有 description
  // CORE-011 完成前无法调用 getEmbedding，一律 state=-1，embedding 为空
  // CORE-011 完成后，设置页"重新初始化 Skill"功能可补生成 embedding
  const hasOnnx = checkOnnxModelExists();
  if (!hasOnnx) {
    logger.warn('记忆功能', 'ONNX 模型文件未安装，已跳过 Skill 向量初始化');
    logger.detail('记忆功能', 'ONNX 缺失详情', {
      state: -1,
      reason: 'CORE-011 完成后可在设置页重新初始化 Skill 向量',
    });
  }

  for (const skill of REFERENCE_SKILLS) {
    const exists = checkStmt.get(skill.id);
    if (!exists || exists.n === 0) {
      // TODO(CORE-011): hasOnnx=true 时调用 getEmbedding(skill.description) 生成向量，state=1
      insertStmt.run(skill.id, skill.md5, skill.path, skill.name, skill.description, '', skill.type, now, now, -1);
    }
  }
}

function seedSkillAttributions(db: Database.Database): void {
  const checkStmt = db.prepare<[string, string], { n: number }>(
    'SELECT COUNT(*) as n FROM skill_attributions WHERE skill_id = ? AND attribution = ?',
  );
  const insertStmt = db.prepare<[string, string]>(
    'INSERT INTO skill_attributions (skill_id, attribution) VALUES (?, ?)',
  );
  for (const row of SKILL_ATTRIBUTIONS) {
    const exists = checkStmt.get(row.skillId, row.attribution);
    if (!exists || exists.n === 0) {
      insertStmt.run(row.skillId, row.attribution);
    }
  }
}

// ---------------------------------------------------------------------------
// ONNX 模型文件是否存在（CORE-011 实现前始终返回 false）
// 路径：userData/models/all-MiniLM-L6-v2/onnx/model_fp16.onnx
// ---------------------------------------------------------------------------

function checkOnnxModelExists(): boolean {
  const modelPath = safeJoin(getRuntimeDirectories().models, 'all-MiniLM-L6-v2/onnx/model_fp16.onnx');
  return existsSync(modelPath);
}

// ---------------------------------------------------------------------------
// 主入口：runSeed
// 调用链：main 启动 -> runMigrations -> runSeed
// 所有写入操作包在一个事务中，任何一步失败则整体回滚并抛错
// ---------------------------------------------------------------------------

export function runSeed(database = getDatabase()): void {
  const now = Date.now();

  const transaction = database.transaction(() => {
    seedUsers(database, now);
    seedSettings(database, now);
    seedVendors(database, now);
    seedAgentConfigs(database, now);
    seedPrompts(database, now);
    seedSkills(database, now);
    seedSkillAttributions(database);
  });

  transaction();
  logger.info('默认数据', '已初始化');
}
