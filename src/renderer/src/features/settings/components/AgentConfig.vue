<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import { ChevronDownIcon, ChevronUpIcon, RefreshIcon, SaveIcon } from 'tdesign-icons-vue-next';
import { MessagePlugin } from 'tdesign-vue-next';
import type { AgentConfigGroup, AgentConfigItem, AgentConfigResult, AgentTextModelOption, TextAgentKey } from '@shared/types/agent-config';

type CreativityPreset = 'stable' | 'balanced' | 'creative';
type OutputLengthPreset = 'auto' | 'short' | 'medium' | 'long';

interface AgentDraft {
  overrideEnabled: boolean;
  modelId: string;
  inheritParams: boolean;
  temperatureText: string;
  maxOutputTokensText: string;
}

const CREATIVITY_OPTIONS: Array<{ label: string; value: CreativityPreset; temperature: number }> = [
  { label: '稳', value: 'stable', temperature: 0.4 },
  { label: '平衡', value: 'balanced', temperature: 0.7 },
  { label: '发散', value: 'creative', temperature: 1 },
];

const OUTPUT_LENGTH_OPTIONS: Array<{ label: string; value: OutputLengthPreset; maxOutputTokens: number }> = [
  { label: '自动', value: 'auto', maxOutputTokens: 0 },
  { label: '短', value: 'short', maxOutputTokens: 1024 },
  { label: '中', value: 'medium', maxOutputTokens: 4096 },
  { label: '长', value: 'long', maxOutputTokens: 8192 },
];

const GROUP_LABELS: Record<AgentConfigGroup, string> = {
  main: '主 Agent',
  script: '剧本子 Agent',
  production: '生产子 Agent',
};

const loading = ref(false);
const saving = ref(false);
const advancedVisible = ref(false);
const agents = ref<AgentConfigItem[]>([]);
const availableTextModels = ref<AgentTextModelOption[]>([]);
const defaultTextStatus = ref<AgentConfigResult['defaultTextStatus']>('missing');
const defaultTextStatusText = ref('默认文本模型未配置');
const defaultTextModel = ref<AgentConfigResult['defaultTextModel']>(null);
const globalForm = reactive({
  creativity: 'balanced' as CreativityPreset,
  outputLength: 'auto' as OutputLengthPreset,
});
const drafts = reactive<Record<string, AgentDraft>>({});

const groupedAgents = computed(() => {
  const groups: Record<AgentConfigGroup, AgentConfigItem[]> = {
    main: [],
    script: [],
    production: [],
  };

  for (const agent of agents.value) {
    groups[agent.group].push(agent);
  }

  return groups;
});

const modelOptions = computed(() =>
  availableTextModels.value.map((model) => ({
    value: model.modelId,
    label: `${model.connectionName} / ${model.modelDisplayName} (${model.modelName})`,
  })),
);

function isOk(response: { code: number; msg: string }): boolean {
  return response.code === 200;
}

function getStatusTheme(status: AgentConfigItem['status']): 'success' | 'warning' | 'danger' | 'default' {
  if (status === 'inherited' || status === 'overridden') {
    return 'success';
  }

  if (status === 'disabled') {
    return 'default';
  }

  return status === 'missing-default' ? 'warning' : 'danger';
}

function inferCreativity(temperature: number): CreativityPreset {
  const sorted = [...CREATIVITY_OPTIONS].sort((left, right) => Math.abs(left.temperature - temperature) - Math.abs(right.temperature - temperature));
  return sorted[0]?.value ?? 'balanced';
}

function inferOutputLength(maxOutputTokens: number): OutputLengthPreset {
  return OUTPUT_LENGTH_OPTIONS.find((item) => item.maxOutputTokens === maxOutputTokens)?.value ?? 'auto';
}

function getCreativityTemperature(): number {
  return CREATIVITY_OPTIONS.find((item) => item.value === globalForm.creativity)?.temperature ?? 0.7;
}

function getOutputLengthTokens(): number {
  return OUTPUT_LENGTH_OPTIONS.find((item) => item.value === globalForm.outputLength)?.maxOutputTokens ?? 0;
}

function resetDrafts(nextAgents: AgentConfigItem[]): void {
  for (const key of Object.keys(drafts)) {
    delete drafts[key];
  }

  for (const agent of nextAgents) {
    drafts[agent.key] = {
      overrideEnabled: agent.overrideEnabled,
      modelId: agent.modelId ?? '',
      inheritParams: agent.temperature === null && agent.maxOutputTokens === null,
      temperatureText: String(agent.temperature ?? getCreativityTemperature()),
      maxOutputTokensText: String(agent.maxOutputTokens ?? getOutputLengthTokens()),
    };
  }
}

function getDraft(key: TextAgentKey): AgentDraft {
  return drafts[key] ?? {
    overrideEnabled: false,
    modelId: '',
    inheritParams: true,
    temperatureText: String(getCreativityTemperature()),
    maxOutputTokensText: String(getOutputLengthTokens()),
  };
}

function parseTemperature(text: string): number | null {
  const value = Number(text);
  if (!Number.isFinite(value) || value < 0 || value > 2) {
    return null;
  }

  return value;
}

function parseMaxOutputTokens(text: string): number | null {
  const value = Number(text);
  if (!Number.isFinite(value) || value < 0) {
    return null;
  }

  return Math.floor(value);
}

async function loadConfig(): Promise<void> {
  loading.value = true;
  try {
    const response = await window.vtStudio.settings.agentConfig.get();
    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    agents.value = response.data.agents;
    availableTextModels.value = response.data.availableTextModels;
    defaultTextModel.value = response.data.defaultTextModel;
    defaultTextStatus.value = response.data.defaultTextStatus;
    defaultTextStatusText.value = response.data.defaultTextStatusText;
    globalForm.creativity = inferCreativity(response.data.globalSettings.temperature);
    globalForm.outputLength = inferOutputLength(response.data.globalSettings.maxOutputTokens);
    resetDrafts(response.data.agents);
  } finally {
    loading.value = false;
  }
}

async function saveConfig(): Promise<void> {
  const payloadAgents = agents.value.map((agent) => {
    const draft = getDraft(agent.key);
    if (draft.overrideEnabled && !modelOptions.value.some((item) => item.value === draft.modelId)) {
      throw new Error(`${agent.name} 的覆盖模型无效，请重新选择`);
    }

    if (draft.inheritParams) {
      return {
        key: agent.key,
        modelId: draft.overrideEnabled ? draft.modelId : null,
        temperature: null,
        maxOutputTokens: null,
      };
    }

    const temperature = parseTemperature(draft.temperatureText);
    const maxOutputTokens = parseMaxOutputTokens(draft.maxOutputTokensText);
    if (temperature === null) {
      throw new Error(`${agent.name} 的 temperature 必须在 0-2 之间`);
    }

    if (maxOutputTokens === null) {
      throw new Error(`${agent.name} 的 maxOutputTokens 必须大于等于 0`);
    }

    return {
      key: agent.key,
      modelId: draft.overrideEnabled ? draft.modelId : null,
      temperature,
      maxOutputTokens,
    };
  });

  saving.value = true;
  try {
    const response = await window.vtStudio.settings.agentConfig.save({
      globalSettings: {
        temperature: getCreativityTemperature(),
        maxOutputTokens: getOutputLengthTokens(),
      },
      agents: payloadAgents,
    });

    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    MessagePlugin.success('Agent 配置已保存');
    await loadConfig();
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Agent 配置保存失败';
    MessagePlugin.error(message);
  } finally {
    saving.value = false;
  }
}

defineExpose({ loadConfig });
onMounted(loadConfig);
</script>

<template>
  <section class="agent-config-panel">
    <div class="agent-config-head">
      <div>
        <strong>Agent 高级设置</strong>
        <p>普通 Agent 继承默认文本模型；高级模式才单独覆盖模型和参数。</p>
      </div>
      <div class="settings-actions">
        <t-button variant="outline" :loading="loading" @click="loadConfig">
          <template #icon><RefreshIcon /></template>
          刷新
        </t-button>
        <t-button theme="primary" :loading="saving" @click="saveConfig">
          <template #icon><SaveIcon /></template>
          保存
        </t-button>
      </div>
    </div>

    <div class="agent-default-row">
      <div class="agent-default-model">
        <span>默认文本模型</span>
        <b>{{ defaultTextModel ? `${defaultTextModel.connectionName} / ${defaultTextModel.modelDisplayName}` : '未配置' }}</b>
        <small v-if="defaultTextModel">{{ defaultTextModel.modelName }}</small>
      </div>
      <t-tag :theme="defaultTextStatus === 'configured' ? 'success' : 'warning'" variant="light">{{ defaultTextStatusText }}</t-tag>
    </div>

    <div class="agent-simple-controls">
      <t-form layout="inline">
        <t-form-item label="创作稳定性">
          <t-radio-group v-model="globalForm.creativity" variant="default-filled">
            <t-radio-button v-for="item in CREATIVITY_OPTIONS" :key="item.value" :value="item.value">{{ item.label }}</t-radio-button>
          </t-radio-group>
        </t-form-item>
        <t-form-item label="输出长度">
          <t-radio-group v-model="globalForm.outputLength" variant="default-filled">
            <t-radio-button v-for="item in OUTPUT_LENGTH_OPTIONS" :key="item.value" :value="item.value">{{ item.label }}</t-radio-button>
          </t-radio-group>
        </t-form-item>
      </t-form>
    </div>

    <button type="button" class="agent-advanced-toggle" @click="advancedVisible = !advancedVisible">
      <span>高级覆盖</span>
      <ChevronUpIcon v-if="advancedVisible" />
      <ChevronDownIcon v-else />
    </button>

    <div v-if="advancedVisible" class="agent-group-list">
      <section v-for="(items, group) in groupedAgents" :key="group" class="agent-group">
        <div class="agent-group-title">{{ GROUP_LABELS[group] }}</div>
        <div class="agent-card-grid">
          <article v-for="agent in items" :key="agent.key" class="agent-card">
            <div class="agent-card-head">
              <div>
                <strong>{{ agent.name }}</strong>
                <small>{{ agent.key }}</small>
              </div>
              <t-tag :theme="getStatusTheme(agent.status)" variant="light">{{ agent.statusText }}</t-tag>
            </div>

            <div class="agent-effective-model">
              <span>当前生效</span>
              <b>{{ agent.effectiveModel ? `${agent.effectiveModel.connectionName} / ${agent.effectiveModel.modelDisplayName}` : '未配置' }}</b>
              <small v-if="agent.effectiveModel">{{ agent.effectiveModel.modelName }}</small>
            </div>

            <div class="agent-field-row">
              <span>覆盖模型</span>
              <t-switch v-model="getDraft(agent.key).overrideEnabled" />
            </div>
            <t-select v-if="getDraft(agent.key).overrideEnabled" v-model="getDraft(agent.key).modelId" placeholder="选择已启用文本模型">
              <t-option v-for="option in modelOptions" :key="option.value" :value="option.value" :label="option.label" />
            </t-select>

            <div class="agent-field-row">
              <span>继承全局参数</span>
              <t-switch v-model="getDraft(agent.key).inheritParams" />
            </div>
            <div v-if="!getDraft(agent.key).inheritParams" class="agent-param-grid">
              <label>
                <span>temperature</span>
                <t-input v-model="getDraft(agent.key).temperatureText" placeholder="0-2" />
              </label>
              <label>
                <span>maxOutputTokens</span>
                <t-input v-model="getDraft(agent.key).maxOutputTokensText" placeholder="0 为自动" />
              </label>
            </div>
          </article>
        </div>
      </section>
    </div>
  </section>
</template>
