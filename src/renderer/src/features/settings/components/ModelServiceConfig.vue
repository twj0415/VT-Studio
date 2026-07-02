<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue';
import { AddIcon, DeleteIcon, EditIcon, PlayCircleIcon, RefreshIcon, SwapIcon } from 'tdesign-icons-vue-next';
import { DialogPlugin, MessagePlugin } from 'tdesign-vue-next';
import type { ApiConnection, ApiConnectionDraft, ApiServiceType, CapabilitySummary, ModelCapability, RegisteredModel } from '@shared/types/model-config';
import AgentConfig from './AgentConfig.vue';

interface ServiceTemplate {
  serviceType: ApiServiceType;
  name: string;
  defaultBaseUrl: string;
  capabilities: ModelCapability[];
  models: RegisteredModel[];
}

const CAPABILITY_OPTIONS: Array<{ label: string; value: ModelCapability }> = [
  { label: '文本', value: 'text' },
  { label: '图片', value: 'image' },
  { label: '视频', value: 'video' },
  { label: '语音', value: 'tts' },
];

const SERVICE_ORDER: ApiServiceType[] = ['openai-gateway', 'openai-official', 'claude', 'deepseek', 'gemini', 'local-workflow', 'advanced'];

const loading = ref(false);
const saving = ref(false);
const testingConnectionId = ref('');
const testingCapability = ref<ModelCapability | ''>('');
const connections = ref<ApiConnection[]>([]);
const capabilities = ref<CapabilitySummary[]>([]);
const templates = ref<ServiceTemplate[]>([]);
const serviceDialogVisible = ref(false);
const bindingDialogVisible = ref(false);
const agentConfigRef = ref<InstanceType<typeof AgentConfig> | null>(null);
const editingId = ref('');
const activeCapability = ref<ModelCapability>('text');
const testResult = ref('');
const testPrompt = ref('请用一句话回复：模型配置测试成功。');
const serviceForm = reactive({
  name: '',
  serviceType: 'openai-gateway' as ApiServiceType,
  baseUrl: '',
  apiKey: '',
  selectedModelNames: [] as string[],
});
const bindingForm = reactive({
  connectionId: '',
  modelName: '',
});

const orderedTemplates = computed(() =>
  [...templates.value].sort((left, right) => SERVICE_ORDER.indexOf(left.serviceType) - SERVICE_ORDER.indexOf(right.serviceType)),
);
const serviceOptions = computed(() => orderedTemplates.value.map((item) => ({ label: item.name, value: item.serviceType })));
const selectedTemplate = computed(() => templates.value.find((item) => item.serviceType === serviceForm.serviceType) ?? null);
const editingConnection = computed(() => connections.value.find((connection) => connection.id === editingId.value) ?? null);
const selectableModels = computed(() => {
  const models = new Map<string, RegisteredModel>();
  selectedTemplate.value?.models.forEach((model) => models.set(model.modelName, model));
  editingConnection.value?.models.forEach((model) => models.set(model.modelName, model));
  return [...models.values()];
});
const selectedModels = computed(() => selectableModels.value.filter((model) => serviceForm.selectedModelNames.includes(model.modelName)));
const activeSummary = computed(() => capabilities.value.find((item) => item.capability === activeCapability.value) ?? null);
const availableConnections = computed(() => connections.value.filter((connection) => connection.models.some((model) => model.type === activeCapability.value)));
const selectedBindingConnection = computed(() => availableConnections.value.find((connection) => connection.id === bindingForm.connectionId) ?? null);
const availableModels = computed(() => selectedBindingConnection.value?.models.filter((model) => model.type === activeCapability.value) ?? []);

function isOk(response: { code: number; msg: string }): boolean {
  return response.code === 200;
}

function getCapabilityLabel(capability: ModelCapability): string {
  return CAPABILITY_OPTIONS.find((item) => item.value === capability)?.label ?? capability;
}

function getServiceName(serviceType: ApiServiceType): string {
  return templates.value.find((item) => item.serviceType === serviceType)?.name ?? serviceType;
}

function getModelsByType(type: ModelCapability): RegisteredModel[] {
  return selectableModels.value.filter((model) => model.type === type);
}

function getConnectionCapabilities(connection: ApiConnection): ModelCapability[] {
  return [...new Set(connection.models.map((model) => model.type))] as ModelCapability[];
}

function getStatusTheme(status: CapabilitySummary['status']): 'success' | 'warning' | 'danger' {
  if (status === 'configured') {
    return 'success';
  }

  return status === 'missing' ? 'warning' : 'danger';
}

function getModelOptionLabel(connection: ApiConnection, modelDisplayName: string, modelName: string): string {
  return `${connection.name} / ${modelDisplayName} (${modelName})`;
}

function applyTemplate(template: ServiceTemplate): void {
  serviceForm.name = template.name;
  serviceForm.baseUrl = template.defaultBaseUrl;
  serviceForm.selectedModelNames = template.models.map((model) => model.modelName);
}

async function loadModelService(): Promise<void> {
  loading.value = true;
  try {
    const [templateResponse, listResponse, resourceResponse] = await Promise.all([
      window.vtStudio.settings.api.templates(),
      window.vtStudio.settings.api.list(),
      window.vtStudio.settings.resource.get(),
    ]);

    if (!isOk(templateResponse)) {
      MessagePlugin.error(templateResponse.msg);
      return;
    }

    if (!isOk(listResponse)) {
      MessagePlugin.error(listResponse.msg);
      return;
    }

    if (!isOk(resourceResponse)) {
      MessagePlugin.error(resourceResponse.msg);
      return;
    }

    templates.value = templateResponse.data.services as ServiceTemplate[];
    connections.value = listResponse.data.connections;
    capabilities.value = resourceResponse.data.capabilities;
    await agentConfigRef.value?.loadConfig();
  } finally {
    loading.value = false;
  }
}

function openCreateServiceDialog(): void {
  editingId.value = '';
  const template = templates.value.find((item) => item.serviceType === 'openai-gateway') ?? orderedTemplates.value[0];
  if (template) {
    serviceForm.serviceType = template.serviceType;
    applyTemplate(template);
  }
  serviceForm.apiKey = '';
  testResult.value = '';
  serviceDialogVisible.value = true;
}

function openEditServiceDialog(connection: ApiConnection): void {
  editingId.value = connection.id;
  serviceForm.name = connection.name;
  serviceForm.serviceType = connection.serviceType;
  serviceForm.baseUrl = connection.baseUrl;
  serviceForm.apiKey = connection.apiKey;
  serviceForm.selectedModelNames = connection.models.map((model) => model.modelName);
  testResult.value = '';
  serviceDialogVisible.value = true;
}

function openBindingDialog(summary: CapabilitySummary): void {
  activeCapability.value = summary.capability;
  bindingForm.connectionId = summary.binding?.connectionId ?? availableConnections.value[0]?.id ?? '';
  const connection = availableConnections.value.find((item) => item.id === bindingForm.connectionId);
  bindingForm.modelName = summary.binding?.modelName ?? connection?.models.find((model) => model.type === summary.capability)?.modelName ?? '';
  testResult.value = '';
  bindingDialogVisible.value = true;
}

function buildDraft(): ApiConnectionDraft | null {
  if (!serviceForm.name.trim()) {
    MessagePlugin.warning('服务名称不能为空');
    return null;
  }

  if (!serviceForm.apiKey.trim() && serviceForm.serviceType !== 'local-workflow') {
    MessagePlugin.warning('API Key 不能为空');
    return null;
  }

  if (selectedModels.value.length === 0) {
    MessagePlugin.warning('至少需要启用一个模型');
    return null;
  }

  const capabilitiesFromModels = [...new Set(selectedModels.value.map((model) => model.type))] as ModelCapability[];
  return {
    id: editingId.value || undefined,
    name: serviceForm.name.trim(),
    serviceType: serviceForm.serviceType,
    baseUrl: serviceForm.baseUrl.trim(),
    apiKey: serviceForm.apiKey.trim(),
    capabilities: capabilitiesFromModels,
    models: selectedModels.value.map((model) => ({ ...model })),
  };
}

async function saveService(): Promise<void> {
  const draft = buildDraft();
  if (!draft) {
    return;
  }

  saving.value = true;
  try {
    const response = await window.vtStudio.settings.api.save({ connection: draft });
    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    MessagePlugin.success(editingId.value ? '模型服务已保存' : '模型服务已创建');
    serviceDialogVisible.value = false;
    await loadModelService();
  } catch (error) {
    const message = error instanceof Error ? error.message : '模型服务保存失败';
    MessagePlugin.error(message);
  } finally {
    saving.value = false;
  }
}

async function saveBinding(): Promise<void> {
  if (!bindingForm.connectionId || !bindingForm.modelName) {
    MessagePlugin.warning('请选择服务和模型');
    return;
  }

  const response = await window.vtStudio.settings.resource.saveBinding({
    capability: activeCapability.value,
    binding: {
      connectionId: bindingForm.connectionId,
      modelName: bindingForm.modelName,
    },
  });

  if (!isOk(response)) {
    MessagePlugin.error(response.msg);
    return;
  }

  MessagePlugin.success('默认模型已更新');
  bindingDialogVisible.value = false;
  await loadModelService();
}

async function testConnection(connection: ApiConnection): Promise<void> {
  const model = connection.models.find((item) => item.type === 'text');
  if (!model) {
    MessagePlugin.warning('当前服务没有文本模型，暂不能测试');
    return;
  }

  testingConnectionId.value = connection.id;
  testResult.value = '';
  try {
    const response = await window.vtStudio.settings.api.test({
      connectionId: connection.id,
      modelName: model.modelName,
      prompt: testPrompt.value,
    });

    if (!isOk(response)) {
      testResult.value = response.msg;
      MessagePlugin.error(response.msg);
      return;
    }

    testResult.value = `${connection.name} / ${model.displayName}\n${response.data.content}\n\n耗时：${response.data.durationMs}ms`;
    MessagePlugin.success('文本模型测试成功');
  } finally {
    testingConnectionId.value = '';
  }
}

async function runCapabilityTest(summary: CapabilitySummary): Promise<void> {
  if (summary.capability !== 'text') {
    MessagePlugin.warning('第一版先打通文本测试，媒体测试会在后续能力任务接入');
    return;
  }

  testingCapability.value = summary.capability;
  testResult.value = '';
  try {
    const response = await window.vtStudio.settings.resource.test({
      capability: summary.capability,
      prompt: testPrompt.value,
    });

    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      testResult.value = response.msg;
      return;
    }

    testResult.value = `${summary.label} / ${summary.modelDisplayName}\n${response.data.content}\n\n耗时：${response.data.durationMs}ms`;
    MessagePlugin.success('文本模型测试成功');
  } finally {
    testingCapability.value = '';
  }
}

function confirmDeleteConnection(connection: ApiConnection): void {
  const dialog = DialogPlugin.confirm({
    header: '删除模型服务',
    body: `将删除 ${connection.name}。如果默认模型或 Agent 仍在引用，系统会阻止删除。`,
    confirmBtn: '删除',
    cancelBtn: '取消',
    theme: 'danger',
    async onConfirm() {
      const response = await window.vtStudio.settings.api.delete({ connectionId: connection.id });
      if (!isOk(response)) {
        MessagePlugin.error(response.msg);
        return;
      }

      dialog.destroy();
      MessagePlugin.success('模型服务已删除');
      await loadModelService();
    },
  });
}

watch(
  () => serviceForm.serviceType,
  (serviceType, oldServiceType) => {
    if (editingId.value || serviceType === oldServiceType) {
      return;
    }

    const template = selectedTemplate.value;
    if (template) {
      applyTemplate(template);
    }
  },
);

watch(
  () => bindingForm.connectionId,
  () => {
    if (!availableModels.value.some((model) => model.modelName === bindingForm.modelName)) {
      bindingForm.modelName = availableModels.value[0]?.modelName ?? '';
    }
  },
);

onMounted(loadModelService);
</script>

<template>
  <section class="settings-section model-service-section">
    <div class="settings-section-head">
      <div>
        <p class="eyebrow">模型服务</p>
        <h3>模型接入与默认模型</h3>
      </div>
      <div class="settings-actions">
        <t-button variant="outline" :loading="loading" @click="loadModelService">
          <template #icon><RefreshIcon /></template>
          刷新
        </t-button>
        <t-button theme="primary" @click="openCreateServiceDialog">
          <template #icon><AddIcon /></template>
          新增模型服务
        </t-button>
      </div>
    </div>

    <div class="model-service-summary">
      <article v-for="summary in capabilities" :key="summary.capability" class="capability-card">
        <div class="capability-card-head">
          <div>
            <strong>{{ summary.label }}</strong>
            <small>{{ summary.connectionName }}</small>
          </div>
          <t-tag :theme="getStatusTheme(summary.status)" variant="light">{{ summary.statusText }}</t-tag>
        </div>

        <div class="capability-model">
          <span>默认模型</span>
          <b>{{ summary.modelDisplayName }}</b>
          <small v-if="summary.modelName">{{ summary.modelName }}</small>
        </div>

        <div class="capability-actions">
          <t-button variant="outline" @click="openBindingDialog(summary)">
            <template #icon><SwapIcon /></template>
            更换
          </t-button>
          <t-button theme="primary" :loading="testingCapability === summary.capability" :disabled="summary.status !== 'configured'" @click="runCapabilityTest(summary)">
            <template #icon><PlayCircleIcon /></template>
            测试
          </t-button>
        </div>
      </article>
    </div>

    <div class="model-service-block-head">
      <div>
        <strong>已连接服务</strong>
        <p>模型能力由启用的模型决定；保存服务后会自动补齐可用默认模型。</p>
      </div>
    </div>

    <div v-if="connections.length" class="connection-grid">
      <article v-for="connection in connections" :key="connection.id" class="connection-card">
        <div class="connection-card-head">
          <div>
            <strong>{{ connection.name }}</strong>
            <small>{{ getServiceName(connection.serviceType) }} · {{ connection.baseUrl || '默认地址' }}</small>
          </div>
          <t-tag :theme="connection.status === 'ready' ? 'success' : 'warning'" variant="light">{{ connection.statusText }}</t-tag>
        </div>

        <div class="connection-meta">
          <t-tag v-for="capability in getConnectionCapabilities(connection)" :key="capability" variant="light">{{ getCapabilityLabel(capability) }}</t-tag>
        </div>

        <div class="connection-models">
          <span v-for="model in connection.models" :key="model.modelName">{{ model.displayName }}</span>
        </div>

        <div class="connection-actions">
          <t-button shape="square" variant="text" title="测试文本模型" :loading="testingConnectionId === connection.id" @click="testConnection(connection)">
            <PlayCircleIcon />
          </t-button>
          <t-button shape="square" variant="text" title="编辑模型服务" @click="openEditServiceDialog(connection)">
            <EditIcon />
          </t-button>
          <t-button shape="square" variant="text" theme="danger" title="删除模型服务" @click="confirmDeleteConnection(connection)">
            <DeleteIcon />
          </t-button>
        </div>
      </article>
    </div>

    <t-empty v-else description="还没有模型服务，先新增一个 OpenAI 中转或官方服务" />

    <div v-if="testResult" class="resource-test-result">
      <strong>测试结果</strong>
      <pre>{{ testResult }}</pre>
    </div>

    <AgentConfig ref="agentConfigRef" />
  </section>

  <t-dialog v-model:visible="serviceDialogVisible" :header="editingId ? '编辑模型服务' : '新增模型服务'" width="720px" confirm-btn="保存" :confirm-loading="saving" @confirm="saveService">
    <t-form layout="vertical">
      <div class="model-form-grid">
        <t-form-item label="服务类型">
          <t-select v-model="serviceForm.serviceType" :disabled="Boolean(editingId)">
            <t-option v-for="option in serviceOptions" :key="option.value" :value="option.value" :label="option.label" />
          </t-select>
        </t-form-item>
        <t-form-item label="服务名称">
          <t-input v-model="serviceForm.name" placeholder="例如 我的 OpenAI 中转" />
        </t-form-item>
      </div>

      <t-form-item label="Base URL">
        <t-input v-model="serviceForm.baseUrl" placeholder="官方服务可使用默认地址，中转服务通常填到 /v1" />
      </t-form-item>
      <t-form-item label="API Key">
        <t-input v-model="serviceForm.apiKey" type="password" placeholder="请输入 API Key" />
      </t-form-item>

      <div class="model-enable-panel">
        <div class="model-enable-head">
          <span>启用模型</span>
          <small>普通用户只需要勾选这个服务实际可用的模型</small>
        </div>
        <t-checkbox-group v-model="serviceForm.selectedModelNames" class="model-check-list">
          <div v-for="item in CAPABILITY_OPTIONS" :key="item.value" class="model-check-group">
            <strong v-if="getModelsByType(item.value).length">{{ item.label }}</strong>
            <t-checkbox v-for="model in getModelsByType(item.value)" :key="model.modelName" :value="model.modelName">
              <span>{{ model.displayName }}</span>
              <small>{{ model.modelName }}</small>
            </t-checkbox>
          </div>
        </t-checkbox-group>
      </div>

      <p class="settings-hint">普通用户不需要选择协议；系统按服务类型自动适配。协议、adapter、headers 等细节放在开发者模式。</p>
    </t-form>
  </t-dialog>

  <t-dialog v-model:visible="bindingDialogVisible" :header="activeSummary ? `更换${activeSummary.label}` : '更换模型'" width="560px" confirm-btn="保存" @confirm="saveBinding">
    <t-form layout="vertical">
      <t-form-item label="服务">
        <t-select v-model="bindingForm.connectionId" placeholder="选择模型服务">
          <t-option v-for="connection in availableConnections" :key="connection.id" :value="connection.id" :label="connection.name" />
        </t-select>
      </t-form-item>
      <t-form-item label="模型">
        <t-select v-model="bindingForm.modelName" placeholder="选择模型">
          <t-option
            v-for="model in availableModels"
            :key="model.modelName"
            :value="model.modelName"
            :label="selectedBindingConnection ? getModelOptionLabel(selectedBindingConnection, model.displayName, model.modelName) : model.displayName"
          />
        </t-select>
      </t-form-item>
      <t-form-item v-if="activeCapability === 'text'" label="测试 Prompt">
        <t-textarea v-model="testPrompt" :autosize="{ minRows: 3, maxRows: 6 }" />
      </t-form-item>
      <p v-if="availableConnections.length === 0" class="settings-hint">还没有支持当前能力的模型，请先新增模型服务并启用对应模型。</p>
    </t-form>
  </t-dialog>
</template>
