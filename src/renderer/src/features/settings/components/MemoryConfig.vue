<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import { RefreshIcon, RollbackIcon, SaveIcon } from 'tdesign-icons-vue-next';
import { DialogPlugin, MessagePlugin } from 'tdesign-vue-next';
import type { AgentNamespace } from '@shared/types/socket';
import type { MemoryClearType } from '@shared/types/memory';
import type { MemorySettingsConfig, MemoryStatsResult } from '@shared/types/memory-settings';

const loading = ref(false);
const saving = ref(false);
const restoring = ref(false);
const clearing = ref(false);
const validating = ref(false);
const modelAvailable = ref(false);
const modelRelativePath = ref('');
const stats = ref<MemoryStatsResult>({ total: 0, messages: 0, summaries: 0, isolations: [] });

const form = reactive({
  modelPathText: '',
  modelDtype: 'fp16',
  messagesPerSummary: '10',
  shortTermLimit: '5',
  summaryMaxLength: '500',
  summaryLimit: '10',
  ragLimit: '3',
  deepRetrieveSummaryLimit: '5',
});

const clearForm = reactive({
  scope: 'isolation' as 'isolation' | 'all',
  projectId: '',
  agentType: 'scriptAgent' as AgentNamespace,
  episodesId: '',
  type: 'all' as MemoryClearType,
  confirmText: '',
});

const modelStatusText = computed(() => (modelAvailable.value ? '模型文件可用' : '模型文件不存在'));

function isOk(response: { code: number; msg: string }): boolean {
  return response.code === 200;
}

function setForm(config: MemorySettingsConfig): void {
  form.modelPathText = config.modelOnnxFile.join('/');
  form.modelDtype = config.modelDtype;
  form.messagesPerSummary = String(config.messagesPerSummary);
  form.shortTermLimit = String(config.shortTermLimit);
  form.summaryMaxLength = String(config.summaryMaxLength);
  form.summaryLimit = String(config.summaryLimit);
  form.ragLimit = String(config.ragLimit);
  form.deepRetrieveSummaryLimit = String(config.deepRetrieveSummaryLimit);
}

function parsePathText(): string[] {
  return form.modelPathText.split('/').map((item) => item.trim()).filter(Boolean);
}

function parseNumber(value: string, label: string): number {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    throw new Error(`${label} 必须是数字`);
  }

  return Math.floor(parsed);
}

function buildPayload(): MemorySettingsConfig {
  return {
    modelOnnxFile: parsePathText(),
    modelDtype: form.modelDtype as MemorySettingsConfig['modelDtype'],
    messagesPerSummary: parseNumber(form.messagesPerSummary, '消息摘要阈值'),
    shortTermLimit: parseNumber(form.shortTermLimit, '短期记忆限制'),
    summaryMaxLength: parseNumber(form.summaryMaxLength, '摘要长度'),
    summaryLimit: parseNumber(form.summaryLimit, '摘要数量'),
    ragLimit: parseNumber(form.ragLimit, 'RAG 数量'),
    deepRetrieveSummaryLimit: parseNumber(form.deepRetrieveSummaryLimit, '深度检索摘要数量'),
  };
}

async function loadConfig(): Promise<void> {
  loading.value = true;
  try {
    const response = await window.vtStudio.settings.memory.get();
    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    setForm(response.data.config);
    modelAvailable.value = response.data.modelStatus.available;
    modelRelativePath.value = response.data.modelStatus.relativePath;
    stats.value = response.data.stats;
  } finally {
    loading.value = false;
  }
}

async function validateModelPath(): Promise<void> {
  validating.value = true;
  try {
    const response = await window.vtStudio.settings.memory.validateModelPath({ modelOnnxFile: parsePathText() });
    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    modelAvailable.value = response.data.available;
    modelRelativePath.value = response.data.relativePath;
    MessagePlugin.info(response.data.available ? '模型文件可用' : '模型文件不存在');
  } catch (error) {
    MessagePlugin.error(error instanceof Error ? error.message : '模型路径校验失败');
  } finally {
    validating.value = false;
  }
}

async function saveConfig(): Promise<void> {
  saving.value = true;
  try {
    const response = await window.vtStudio.settings.memory.save(buildPayload());
    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    setForm(response.data.config);
    modelAvailable.value = response.data.modelStatus.available;
    modelRelativePath.value = response.data.modelStatus.relativePath;
    stats.value = response.data.stats;
    MessagePlugin.success('记忆配置已保存');
  } catch (error) {
    MessagePlugin.error(error instanceof Error ? error.message : '保存失败');
  } finally {
    saving.value = false;
  }
}

async function restoreDefault(): Promise<void> {
  const dialog = DialogPlugin.confirm({
    header: '恢复默认记忆配置',
    body: '恢复默认只重置配置，不清空已有记忆。',
    confirmBtn: '恢复默认',
    cancelBtn: '取消',
    theme: 'warning',
    async onConfirm() {
      restoring.value = true;
      try {
        const response = await window.vtStudio.settings.memory.restoreDefault();
        if (!isOk(response)) {
          MessagePlugin.error(response.msg);
          return;
        }

        setForm(response.data.config);
        modelAvailable.value = response.data.modelStatus.available;
        modelRelativePath.value = response.data.modelStatus.relativePath;
        stats.value = response.data.stats;
        MessagePlugin.success('已恢复默认');
        dialog.destroy();
      } finally {
        restoring.value = false;
      }
    },
  });
}

async function clearMemory(): Promise<void> {
  const dialog = DialogPlugin.confirm({
    header: clearForm.scope === 'all' ? '清空全部记忆' : '清空指定记忆',
    body: clearForm.scope === 'all' ? '该操作会删除全部 Agent 记忆。' : '该操作会删除指定隔离范围内的记忆。',
    confirmBtn: '确认清空',
    cancelBtn: '取消',
    theme: 'danger',
    async onConfirm() {
      clearing.value = true;
      try {
        const response = await window.vtStudio.settings.memory.clear({
          scope: clearForm.scope,
          type: clearForm.type,
          projectId: clearForm.projectId,
          agentType: clearForm.agentType,
          episodesId: clearForm.episodesId,
          confirmText: clearForm.confirmText,
        });
        if (!isOk(response)) {
          MessagePlugin.error(response.msg);
          return;
        }

        stats.value = response.data.stats;
        MessagePlugin.success(`已清理 ${response.data.deleted} 条记忆`);
        dialog.destroy();
      } finally {
        clearing.value = false;
      }
    },
  });
}

defineExpose({ loadConfig });
onMounted(loadConfig);
</script>

<template>
  <section class="memory-config-section">
    <div class="memory-config-head">
      <div>
        <strong>记忆配置</strong>
        <p>配置本地 embedding、摘要和 RAG 参数；清空操作只处理 memories 表。</p>
      </div>
      <div class="settings-actions">
        <t-button variant="outline" :loading="loading" @click="loadConfig">
          <template #icon><RefreshIcon /></template>
          刷新
        </t-button>
        <t-button variant="outline" theme="warning" :loading="restoring" @click="restoreDefault">
          <template #icon><RollbackIcon /></template>
          恢复默认
        </t-button>
        <t-button theme="primary" :loading="saving" @click="saveConfig">
          <template #icon><SaveIcon /></template>
          保存
        </t-button>
      </div>
    </div>

    <div class="memory-status-row">
      <div>
        <span>ONNX 模型</span>
        <b>{{ modelRelativePath || form.modelPathText }}</b>
      </div>
      <t-tag :theme="modelAvailable ? 'success' : 'warning'" variant="light">{{ modelStatusText }}</t-tag>
    </div>

    <t-form class="memory-config-form" layout="vertical">
      <t-form-item label="ONNX 相对路径">
        <div class="memory-path-row">
          <t-input v-model="form.modelPathText" placeholder="all-MiniLM-L6-v2/onnx/model_fp16.onnx" />
          <t-button variant="outline" :loading="validating" @click="validateModelPath">校验</t-button>
        </div>
      </t-form-item>
      <t-form-item label="dtype">
        <t-select v-model="form.modelDtype">
          <t-option value="fp16" label="fp16" />
          <t-option value="fp32" label="fp32" />
          <t-option value="q8" label="q8" />
        </t-select>
      </t-form-item>
      <div class="memory-number-grid">
        <t-form-item label="消息摘要阈值">
          <t-input v-model="form.messagesPerSummary" />
        </t-form-item>
        <t-form-item label="短期记忆限制">
          <t-input v-model="form.shortTermLimit" />
        </t-form-item>
        <t-form-item label="摘要长度">
          <t-input v-model="form.summaryMaxLength" />
        </t-form-item>
        <t-form-item label="摘要数量">
          <t-input v-model="form.summaryLimit" />
        </t-form-item>
        <t-form-item label="RAG 数量">
          <t-input v-model="form.ragLimit" />
        </t-form-item>
        <t-form-item label="深度检索摘要数量">
          <t-input v-model="form.deepRetrieveSummaryLimit" />
        </t-form-item>
      </div>
    </t-form>

    <div class="memory-stats-grid">
      <div>
        <span>总记忆</span>
        <b>{{ stats.total }}</b>
      </div>
      <div>
        <span>消息</span>
        <b>{{ stats.messages }}</b>
      </div>
      <div>
        <span>摘要</span>
        <b>{{ stats.summaries }}</b>
      </div>
      <div>
        <span>隔离范围</span>
        <b>{{ stats.isolations.length }}</b>
      </div>
    </div>

    <div class="memory-clear-panel">
      <div>
        <strong>清空记忆</strong>
        <p>指定范围清理；全部清空必须输入确认短语。</p>
      </div>
      <div class="memory-clear-grid">
        <t-radio-group v-model="clearForm.scope">
          <t-radio-button value="isolation">指定范围</t-radio-button>
          <t-radio-button value="all">全部</t-radio-button>
        </t-radio-group>
        <t-select v-model="clearForm.type">
          <t-option value="all" label="全部类型" />
          <t-option value="message" label="消息" />
          <t-option value="summary" label="摘要" />
        </t-select>
        <template v-if="clearForm.scope === 'isolation'">
          <t-input v-model="clearForm.projectId" placeholder="项目 ID" />
          <t-select v-model="clearForm.agentType">
            <t-option value="scriptAgent" label="剧本 Agent" />
            <t-option value="productionAgent" label="生产 Agent" />
          </t-select>
          <t-input v-if="clearForm.agentType === 'productionAgent'" v-model="clearForm.episodesId" placeholder="分集 ID" />
        </template>
        <t-input v-if="clearForm.scope === 'all'" v-model="clearForm.confirmText" placeholder="输入：清空全部记忆" />
        <t-button theme="danger" variant="outline" :loading="clearing" @click="clearMemory">清空</t-button>
      </div>
    </div>
  </section>
</template>
