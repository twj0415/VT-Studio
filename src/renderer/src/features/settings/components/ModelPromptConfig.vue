<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import { AddIcon, DeleteIcon, EditIcon, RefreshIcon } from 'tdesign-icons-vue-next';
import { DialogPlugin, MessagePlugin } from 'tdesign-vue-next';
import type {
  ModelPromptConfigResult,
  ModelPromptModelItem,
  ModelPromptModelType,
  ModelPromptTemplate,
  ModelPromptTemplateType,
} from '@shared/types/model-prompt';

const TEMPLATE_TYPE_OPTIONS: Array<{ label: string; value: ModelPromptTemplateType; modelType: ModelPromptModelType }> = [
  { label: '图片模板', value: 'imagePrompt', modelType: 'image' },
  { label: '视频模板', value: 'videoPrompt', modelType: 'video' },
];

const loading = ref(false);
const saving = ref(false);
const deleting = ref(false);
const binding = ref(false);
const config = ref<ModelPromptConfigResult>({
  templates: [],
  connections: [],
  invalidMappings: [],
});
const templateDialogVisible = ref(false);
const bindingDialogVisible = ref(false);
const editingTemplateId = ref<number | null>(null);
const activeModel = ref<ModelPromptModelItem | null>(null);
const selectedTemplateId = ref<number | null>(null);
const templateForm = reactive({
  name: '',
  type: 'imagePrompt' as ModelPromptTemplateType,
  content: '',
});

const currentEditingTemplate = computed(() => config.value.templates.find((item) => item.id === editingTemplateId.value) ?? null);
const compatibleTemplates = computed(() => {
  if (!activeModel.value) {
    return [];
  }

  const type = templateTypeForModel(activeModel.value.modelType);
  return config.value.templates.filter((template) => template.type === type);
});

function isOk(response: { code: number; msg: string }): boolean {
  return response.code === 200;
}

function templateTypeForModel(modelType: ModelPromptModelType): ModelPromptTemplateType {
  return modelType === 'image' ? 'imagePrompt' : 'videoPrompt';
}

function getTemplateTypeLabel(type: ModelPromptTemplateType): string {
  return TEMPLATE_TYPE_OPTIONS.find((item) => item.value === type)?.label ?? type;
}

function getModelTypeLabel(type: ModelPromptModelType): string {
  return type === 'image' ? '图片' : '视频';
}

function getStatusTheme(status: ModelPromptModelItem['status']): 'success' | 'warning' | 'danger' | 'default' {
  if (status === 'bound') {
    return 'success';
  }

  if (status === 'fallback') {
    return 'warning';
  }

  return status === 'invalid-template' || status === 'type-mismatch' ? 'danger' : 'default';
}

function getTemplateSummary(template: ModelPromptTemplate): string {
  return template.content.replace(/\s+/g, ' ').slice(0, 130);
}

async function loadConfig(): Promise<void> {
  loading.value = true;
  try {
    const response = await window.vtStudio.settings.modelPrompt.get();
    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    config.value = response.data;
  } finally {
    loading.value = false;
  }
}

function resetTemplateForm(type: ModelPromptTemplateType = 'imagePrompt'): void {
  editingTemplateId.value = null;
  templateForm.name = '';
  templateForm.type = type;
  templateForm.content = '';
}

function openCreateTemplate(type: ModelPromptTemplateType = 'imagePrompt'): void {
  resetTemplateForm(type);
  templateDialogVisible.value = true;
}

function openEditTemplate(template: ModelPromptTemplate): void {
  editingTemplateId.value = template.id;
  templateForm.name = template.name;
  templateForm.type = template.type;
  templateForm.content = template.content;
  templateDialogVisible.value = true;
}

async function saveTemplate(): Promise<void> {
  if (!templateForm.name.trim() || !templateForm.content.trim()) {
    MessagePlugin.warning('模板名称和内容不能为空');
    return;
  }

  saving.value = true;
  try {
    const response = await window.vtStudio.settings.modelPrompt.saveTemplate({
      id: editingTemplateId.value ?? undefined,
      name: templateForm.name,
      type: templateForm.type,
      content: templateForm.content,
    });

    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    MessagePlugin.success(editingTemplateId.value ? '模板已保存' : '模板已新增');
    templateDialogVisible.value = false;
    await loadConfig();
  } finally {
    saving.value = false;
  }
}

function confirmDeleteTemplate(template: ModelPromptTemplate): void {
  const dialog = DialogPlugin.confirm({
    header: '删除模型模板',
    body: `删除“${template.name}”后不可恢复。被模型引用的模板会被阻止删除。`,
    confirmBtn: '删除',
    cancelBtn: '取消',
    theme: 'danger',
    async onConfirm() {
      deleting.value = true;
      try {
        const response = await window.vtStudio.settings.modelPrompt.deleteTemplate({ id: template.id });
        if (!isOk(response)) {
          MessagePlugin.error(response.msg);
          return;
        }

        MessagePlugin.success('模板已删除');
        dialog.destroy();
        await loadConfig();
      } finally {
        deleting.value = false;
      }
    },
  });
}

function openBindDialog(model: ModelPromptModelItem): void {
  activeModel.value = model;
  selectedTemplateId.value = model.binding?.templateId ?? null;
  bindingDialogVisible.value = true;
}

async function saveBinding(): Promise<void> {
  if (!activeModel.value || !selectedTemplateId.value) {
    MessagePlugin.warning('请选择模板');
    return;
  }

  binding.value = true;
  try {
    const response = await window.vtStudio.settings.modelPrompt.bind({
      connectionId: activeModel.value.connectionId,
      modelName: activeModel.value.modelName,
      modelType: activeModel.value.modelType,
      modelMode: activeModel.value.modelMode,
      templateId: selectedTemplateId.value,
    });

    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    MessagePlugin.success('模型模板已绑定');
    bindingDialogVisible.value = false;
    await loadConfig();
  } finally {
    binding.value = false;
  }
}

async function clearBinding(model: ModelPromptModelItem): Promise<void> {
  binding.value = true;
  try {
    const response = await window.vtStudio.settings.modelPrompt.clearBinding({
      connectionId: model.connectionId,
      modelName: model.modelName,
      modelType: model.modelType,
      modelMode: model.modelMode,
    });

    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    MessagePlugin.success('绑定已清除');
    await loadConfig();
  } finally {
    binding.value = false;
  }
}

defineExpose({ loadConfig });
onMounted(loadConfig);
</script>

<template>
  <section class="model-prompt-section">
    <div class="model-prompt-head">
      <div>
        <strong>模型专用模板</strong>
        <p>高级能力，只给图片/视频模型绑定专用提示词模板；未绑定时由生成服务统一 fallback。</p>
      </div>
      <div class="settings-actions">
        <t-button variant="outline" :loading="loading" @click="loadConfig">
          <template #icon><RefreshIcon /></template>
          刷新
        </t-button>
        <t-button theme="primary" @click="openCreateTemplate()">
          <template #icon><AddIcon /></template>
          新增模板
        </t-button>
      </div>
    </div>

    <div class="model-prompt-block">
      <div class="model-prompt-block-title">
        <strong>模板库</strong>
        <span>{{ config.templates.length }} 个模板</span>
      </div>
      <div v-if="config.templates.length > 0" class="model-prompt-template-grid">
        <article v-for="template in config.templates" :key="template.id" class="model-prompt-template-card">
          <div class="model-prompt-card-head">
            <div>
              <strong>{{ template.name }}</strong>
              <small>{{ getTemplateTypeLabel(template.type) }}</small>
            </div>
            <t-tag :theme="template.referenceCount > 0 ? 'success' : 'default'" variant="light">
              {{ template.referenceCount > 0 ? `${template.referenceCount} 引用` : '未引用' }}
            </t-tag>
          </div>
          <p>{{ getTemplateSummary(template) }}</p>
          <div class="model-prompt-card-actions">
            <t-button size="small" variant="outline" @click="openEditTemplate(template)">
              <template #icon><EditIcon /></template>
              编辑
            </t-button>
            <t-button size="small" variant="outline" theme="danger" :loading="deleting" @click="confirmDeleteTemplate(template)">
              <template #icon><DeleteIcon /></template>
              删除
            </t-button>
          </div>
        </article>
      </div>
      <p v-else class="model-empty">{{ loading ? '正在读取模板...' : '还没有模型模板，先新增一个图片或视频模板。' }}</p>
    </div>

    <div class="model-prompt-block">
      <div class="model-prompt-block-title">
        <strong>模型绑定</strong>
        <span>只显示 image/video 模型</span>
      </div>

      <div v-if="config.connections.length > 0" class="model-prompt-connection-list">
        <section v-for="connection in config.connections" :key="connection.connectionId" class="model-prompt-connection">
          <div class="model-prompt-connection-head">
            <div>
              <strong>{{ connection.connectionName }}</strong>
              <small>{{ connection.connectionId }}</small>
            </div>
            <t-tag :theme="connection.connectionStatus === 'ready' ? 'success' : 'warning'" variant="light">{{ connection.connectionStatusText }}</t-tag>
          </div>

          <div class="model-prompt-model-grid">
            <article v-for="model in connection.models" :key="`${model.connectionId}:${model.modelName}:${model.modelType}:${model.modelMode}`" class="model-prompt-model-card">
              <div class="model-prompt-card-head">
                <div>
                  <strong>{{ model.modelDisplayName }}</strong>
                  <small>{{ model.modelName }}</small>
                </div>
                <t-tag :theme="getStatusTheme(model.status)" variant="light">{{ model.statusText }}</t-tag>
              </div>
              <div class="model-prompt-binding-info">
                <span>{{ getModelTypeLabel(model.modelType) }}模型</span>
                <b>{{ model.binding ? model.binding.templateName : '未绑定专用模板' }}</b>
                <small>{{ model.modelMode ? `模式：${model.modelMode}` : '默认模式' }}</small>
              </div>
              <div class="model-prompt-card-actions">
                <t-button size="small" variant="outline" @click="openBindDialog(model)">绑定</t-button>
                <t-button size="small" variant="outline" :disabled="!model.binding" :loading="binding" @click="clearBinding(model)">清除</t-button>
              </div>
            </article>
          </div>
        </section>
      </div>
      <p v-else class="model-empty">{{ loading ? '正在读取模型...' : '还没有 image/video 模型，请先在模型服务中登记模型。' }}</p>
    </div>

    <div v-if="config.invalidMappings.length > 0" class="model-prompt-warning">
      <strong>失效映射</strong>
      <p v-for="mapping in config.invalidMappings" :key="mapping.id">
        {{ mapping.connectionId }} / {{ mapping.modelName }} / {{ mapping.templateName }}：{{ mapping.reasonText }}
      </p>
    </div>

    <t-dialog v-model:visible="templateDialogVisible" :header="editingTemplateId ? '编辑模型模板' : '新增模型模板'" width="820px" confirm-btn="保存" :confirm-loading="saving" @confirm="saveTemplate">
      <t-form class="settings-form model-prompt-form" :data="templateForm" layout="vertical">
        <t-form-item label="模板名称">
          <t-input v-model="templateForm.name" placeholder="例如：通用图片生成模板" />
        </t-form-item>
        <t-form-item label="模板类型">
          <t-select v-model="templateForm.type" :disabled="Boolean(currentEditingTemplate?.referenceCount)">
            <t-option v-for="item in TEMPLATE_TYPE_OPTIONS" :key="item.value" :value="item.value" :label="item.label" />
          </t-select>
        </t-form-item>
        <t-form-item label="模板内容">
          <t-textarea v-model="templateForm.content" class="code-editor model-prompt-textarea" placeholder="请输入模型专用提示词模板" :autosize="{ minRows: 16, maxRows: 26 }" />
        </t-form-item>
      </t-form>
    </t-dialog>

    <t-dialog v-model:visible="bindingDialogVisible" :header="activeModel ? `绑定模板：${activeModel.modelDisplayName}` : '绑定模板'" width="560px" confirm-btn="保存绑定" :confirm-loading="binding" @confirm="saveBinding">
      <div class="model-prompt-bind-panel">
        <div v-if="activeModel" class="model-prompt-binding-info">
          <span>当前模型</span>
          <b>{{ activeModel.connectionName }} / {{ activeModel.modelDisplayName }}</b>
          <small>{{ activeModel.modelName }}</small>
        </div>
        <t-select v-model="selectedTemplateId" placeholder="选择同类型模板">
          <t-option v-for="template in compatibleTemplates" :key="template.id" :value="template.id" :label="`${template.name}（${getTemplateTypeLabel(template.type)}）`" />
        </t-select>
        <p v-if="compatibleTemplates.length === 0" class="settings-hint">当前类型没有可绑定模板，请先新增模板。</p>
      </div>
    </t-dialog>
  </section>
</template>
