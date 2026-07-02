<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { EditIcon, RefreshIcon, RollbackIcon, SaveIcon } from 'tdesign-icons-vue-next';
import { DialogPlugin, MessagePlugin } from 'tdesign-vue-next';
import type { PromptItem, PromptValidationWarning } from '@shared/types/prompt';

const loading = ref(false);
const saving = ref(false);
const restoring = ref(false);
const prompts = ref<PromptItem[]>([]);
const editorVisible = ref(false);
const activePrompt = ref<PromptItem | null>(null);
const editorText = ref('');

const activeStatusText = computed(() => (activePrompt.value?.isCustomized ? '已自定义' : '默认提示词'));
const activeDataLength = computed(() => editorText.value.trim().length);

function isOk(response: { code: number; msg: string }): boolean {
  return response.code === 200;
}

function getPromptSummary(prompt: PromptItem): string {
  return prompt.effectiveData
    .replace(/\s+/g, ' ')
    .slice(0, 150);
}

function getPromptTheme(prompt: PromptItem): 'success' | 'default' {
  return prompt.isCustomized ? 'success' : 'default';
}

function formatUpdatedAt(value: number): string {
  if (!value) {
    return '未记录';
  }

  return new Date(value).toLocaleString('zh-CN', { hour12: false });
}

function formatWarnings(warnings: PromptValidationWarning[]): string {
  return warnings.map((item) => `- ${item.message}`).join('\n');
}

async function loadPrompts(): Promise<void> {
  loading.value = true;
  try {
    const response = await window.vtStudio.settings.prompt.list();
    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    prompts.value = response.data.prompts;
  } finally {
    loading.value = false;
  }
}

function openEditor(prompt: PromptItem): void {
  activePrompt.value = prompt;
  editorText.value = prompt.effectiveData;
  editorVisible.value = true;
}

function syncActivePrompt(nextPrompt: PromptItem): void {
  activePrompt.value = nextPrompt;
  editorText.value = nextPrompt.effectiveData;
  prompts.value = prompts.value.map((item) => (item.id === nextPrompt.id ? nextPrompt : item));
}

async function savePrompt(force = false): Promise<void> {
  if (!activePrompt.value) {
    return;
  }

  if (!editorText.value.trim()) {
    MessagePlugin.warning('提示词内容不能为空');
    return;
  }

  saving.value = true;
  try {
    const response = await window.vtStudio.settings.prompt.update({
      id: activePrompt.value.id,
      useData: editorText.value,
      force,
    });

    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    if (!response.data.saved) {
      const dialog = DialogPlugin.confirm({
        header: '提示词结构风险',
        body: `检测到这些风险，继续保存可能影响后续生成：\n\n${formatWarnings(response.data.warnings)}`,
        confirmBtn: '仍然保存',
        cancelBtn: '返回修改',
        theme: 'warning',
        async onConfirm() {
          dialog.destroy();
          await savePrompt(true);
        },
      });
      return;
    }

    if (response.data.prompt) {
      syncActivePrompt(response.data.prompt);
    }
    editorVisible.value = false;
    MessagePlugin.success('提示词已保存');
    await loadPrompts();
  } finally {
    saving.value = false;
  }
}

async function restoreDefault(): Promise<void> {
  if (!activePrompt.value) {
    return;
  }

  const prompt = activePrompt.value;
  const dialog = DialogPlugin.confirm({
    header: '恢复默认提示词',
    body: `恢复后会清空“${prompt.name}”的自定义内容，并重新使用内置默认提示词。`,
    confirmBtn: '恢复默认',
    cancelBtn: '取消',
    theme: 'warning',
    async onConfirm() {
      restoring.value = true;
      try {
        const response = await window.vtStudio.settings.prompt.restoreDefault({ id: prompt.id });
        if (!isOk(response)) {
          MessagePlugin.error(response.msg);
          return;
        }

        syncActivePrompt(response.data.prompt);
        MessagePlugin.success('已恢复默认');
        await loadPrompts();
        dialog.destroy();
      } finally {
        restoring.value = false;
      }
    },
  });
}

defineExpose({ loadPrompts });
onMounted(loadPrompts);
</script>

<template>
  <section class="settings-section prompt-section">
    <div class="settings-section-head">
      <div>
        <p class="eyebrow">F-002-006</p>
        <h3>提示词管理</h3>
      </div>
      <t-button variant="outline" :loading="loading" @click="loadPrompts">
        <template #icon><RefreshIcon /></template>
        刷新
      </t-button>
    </div>
    <p class="settings-hint">只管理内置核心提示词；保存写入自定义内容，恢复默认会回到内置默认值。</p>

    <div v-if="prompts.length > 0" class="prompt-grid">
      <article v-for="prompt in prompts" :key="prompt.id" class="prompt-card">
        <div class="prompt-card-head">
          <div>
            <strong>{{ prompt.name }}</strong>
            <small>{{ prompt.type }}</small>
          </div>
          <t-tag :theme="getPromptTheme(prompt)" variant="light">{{ prompt.isCustomized ? '已自定义' : '默认' }}</t-tag>
        </div>
        <p>{{ getPromptSummary(prompt) }}</p>
        <div class="prompt-card-foot">
          <span>{{ formatUpdatedAt(prompt.updatedAt) }}</span>
          <t-button size="small" variant="outline" @click="openEditor(prompt)">
            <template #icon><EditIcon /></template>
            编辑
          </t-button>
        </div>
      </article>
    </div>
    <p v-else class="model-empty">{{ loading ? '正在读取提示词...' : '没有读取到内置提示词，请检查默认数据初始化。' }}</p>

    <t-dialog v-model:visible="editorVisible" :header="activePrompt ? `编辑 ${activePrompt.name}` : '编辑提示词'" width="860px" confirm-btn="保存" :confirm-loading="saving" @confirm="() => savePrompt(false)">
      <div v-if="activePrompt" class="prompt-editor">
        <div class="prompt-editor-meta">
          <div>
            <span>类型</span>
            <b>{{ activePrompt.type }}</b>
          </div>
          <div>
            <span>状态</span>
            <b>{{ activeStatusText }}</b>
          </div>
          <div>
            <span>字符数</span>
            <b>{{ activeDataLength }}</b>
          </div>
        </div>

        <div class="prompt-editor-toolbar">
          <p>保存前会阻止空内容；关键结构缺失时会要求二次确认。</p>
          <t-button variant="outline" theme="warning" :loading="restoring" :disabled="!activePrompt.isCustomized" @click="restoreDefault">
            <template #icon><RollbackIcon /></template>
            恢复默认
          </t-button>
        </div>

        <t-textarea v-model="editorText" class="code-editor prompt-textarea" placeholder="请输入提示词内容" :autosize="{ minRows: 18, maxRows: 28 }" />
      </div>

      <template #footer>
        <div class="prompt-dialog-footer">
          <t-button variant="outline" @click="editorVisible = false">取消</t-button>
          <t-button theme="primary" :loading="saving" @click="savePrompt(false)">
            <template #icon><SaveIcon /></template>
            保存
          </t-button>
        </div>
      </template>
    </t-dialog>
  </section>
</template>
