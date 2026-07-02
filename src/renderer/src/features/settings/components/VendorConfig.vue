<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue';
import { AddIcon, CodeIcon, DeleteIcon, EditIcon, PlayCircleIcon, RefreshIcon, SaveIcon } from 'tdesign-icons-vue-next';
import { DialogPlugin, MessagePlugin } from 'tdesign-vue-next';
import type { VendorListItem, VendorModel, VendorModelType } from '@shared/types/vendor';

const MODEL_TYPE_OPTIONS: Array<{ label: string; value: VendorModelType }> = [
  { label: '文本', value: 'text' },
  { label: '图片', value: 'image' },
  { label: '视频', value: 'video' },
  { label: 'TTS', value: 'tts' },
];

const MODEL_TYPE_LABELS: Record<VendorModelType, string> = {
  text: '文本',
  image: '图片',
  video: '视频',
  tts: 'TTS',
};

const IMAGE_MODE_VALUES = ['text', 'singleImage', 'multiReference'] as const;
type ImageModeValue = (typeof IMAGE_MODE_VALUES)[number];

interface ModelForm {
  name: string;
  modelName: string;
  type: VendorModelType;
  think: boolean;
  imageModes: string[];
  videoModes: string;
  audio: 'optional' | 'true' | 'false';
  durationText: string;
  resolutionText: string;
  voicesText: string;
}

const vendors = ref<VendorListItem[]>([]);
const selectedVendorId = ref('');
const loading = ref(false);
const savingInputs = ref(false);
const inputDraft = reactive<Record<string, string>>({});
const advancedVisible = ref(false);

const modelDialogVisible = ref(false);
const modelDialogMode = ref<'create' | 'edit'>('create');
const editingModelName = ref('');
const modelForm = reactive<ModelForm>(createEmptyModelForm());

const testDialogVisible = ref(false);
const testPrompt = ref('请用一句话回复：模型配置测试成功。');
const testing = ref(false);
const testResult = ref('');
const testModel = ref<VendorModel | null>(null);

const codeDialogVisible = ref(false);
const codeDialogMode = ref<'add' | 'edit'>('add');
const codeDraft = ref('');
const codeVendorId = ref('');
const codeSaving = ref(false);

const selectedVendor = computed(() => vendors.value.find((vendor) => vendor.id === selectedVendorId.value) ?? null);
const selectedVendorBaseUrlHint = computed(() => {
  const vendor = selectedVendor.value;
  if (!vendor) {
    return '';
  }

  if (vendor.id === 'openai') {
    return '官方 OpenAI 可以留空；如果填中转地址，请填到 /v1，不要填完整的 /chat/completions 或 /responses。';
  }

  if (['atlascloud', 'volcengine', 'minimax'].includes(vendor.id)) {
    return '请填写协议根路径，不要填完整的 /chat/completions。';
  }

  return '';
});
const modelsByType = computed(() => {
  const groups: Record<VendorModelType, VendorModel[]> = {
    text: [],
    image: [],
    video: [],
    tts: [],
  };

  for (const model of selectedVendor.value?.models ?? []) {
    groups[model.type].push(model);
  }

  return groups;
});

watch(
  selectedVendor,
  (vendor) => {
    resetInputDraft(vendor);
    testResult.value = '';
  },
  { immediate: true },
);

function createEmptyModelForm(): ModelForm {
  return {
    name: '',
    modelName: '',
    type: 'text',
    think: false,
    imageModes: ['text'],
    videoModes: 'text',
    audio: 'optional',
    durationText: '5,10',
    resolutionText: '720p,1080p',
    voicesText: 'Alloy:alloy',
  };
}

function resetModelForm(model?: VendorModel): void {
  Object.assign(modelForm, createEmptyModelForm());

  if (!model) {
    return;
  }

  modelForm.name = model.name;
  modelForm.modelName = model.modelName;
  modelForm.type = model.type;

  if (model.type === 'text') {
    modelForm.think = model.think;
  }

  if (model.type === 'image') {
    modelForm.imageModes = [...model.mode];
  }

  if (model.type === 'video') {
    modelForm.videoModes = model.mode.map((mode) => (Array.isArray(mode) ? mode.join('|') : mode)).join(',');
    modelForm.audio = model.audio === 'optional' ? 'optional' : model.audio ? 'true' : 'false';
    const first = model.durationResolutionMap[0];
    modelForm.durationText = first?.duration.join(',') ?? '5,10';
    modelForm.resolutionText = first?.resolution.join(',') ?? '720p,1080p';
  }

  if (model.type === 'tts') {
    modelForm.voicesText = model.voices.map((voice) => `${voice.title}:${voice.voice}`).join('\n');
  }
}

function resetInputDraft(vendor: VendorListItem | null): void {
  for (const key of Object.keys(inputDraft)) {
    delete inputDraft[key];
  }

  if (!vendor) {
    return;
  }

  for (const input of vendor.inputs) {
    inputDraft[input.key] = vendor.inputValues[input.key] ?? '';
  }
}

function getResponseOk(response: { code: number; msg: string }): boolean {
  return response.code === 200;
}

async function loadVendors(): Promise<void> {
  loading.value = true;
  try {
    const response = await window.vtStudio.settings.vendor.list();
    if (!getResponseOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    vendors.value = response.data.vendors;
    if (!selectedVendorId.value || !vendors.value.some((vendor) => vendor.id === selectedVendorId.value)) {
      selectedVendorId.value = vendors.value[0]?.id ?? '';
    }
  } finally {
    loading.value = false;
  }
}

async function saveInputs(): Promise<void> {
  const vendor = selectedVendor.value;
  if (!vendor) {
    return;
  }

  const missing = vendor.inputs.find((input) => input.required && !inputDraft[input.key]?.trim());
  if (missing) {
    MessagePlugin.warning(`${missing.label}不能为空`);
    return;
  }

  savingInputs.value = true;
  try {
    const response = await window.vtStudio.settings.vendor.updateInputs({
      vendorId: vendor.id,
      inputValues: { ...inputDraft },
    });

    if (!getResponseOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    MessagePlugin.success('供应商参数已保存');
    await loadVendors();
  } finally {
    savingInputs.value = false;
  }
}

async function saveInputsForTest(vendor: VendorListItem): Promise<boolean> {
  const missing = vendor.inputs.find((input) => input.required && !inputDraft[input.key]?.trim());
  if (missing) {
    const message = `${missing.label}不能为空`;
    testResult.value = message;
    MessagePlugin.warning(message);
    return false;
  }

  const response = await window.vtStudio.settings.vendor.updateInputs({
    vendorId: vendor.id,
    inputValues: { ...inputDraft },
  });

  if (!getResponseOk(response)) {
    testResult.value = response.msg;
    MessagePlugin.error(response.msg);
    return false;
  }

  return true;
}

async function setEnabled(vendor: VendorListItem, enabled: boolean): Promise<void> {
  const previous = vendor.enabled;
  vendor.enabled = enabled;

  const response = await window.vtStudio.settings.vendor.setEnabled({ vendorId: vendor.id, enabled });
  if (!getResponseOk(response)) {
    vendor.enabled = previous;
    MessagePlugin.error(response.msg);
    return;
  }

  MessagePlugin.success(enabled ? '供应商已启用' : '供应商已禁用');
}

function openCreateModelDialog(type: VendorModelType = 'text'): void {
  modelDialogMode.value = 'create';
  editingModelName.value = '';
  resetModelForm();
  modelForm.type = type;
  modelDialogVisible.value = true;
}

function openEditModelDialog(model: VendorModel): void {
  modelDialogMode.value = 'edit';
  editingModelName.value = model.modelName;
  resetModelForm(model);
  modelDialogVisible.value = true;
}

function parseCsv(text: string): string[] {
  return text
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean);
}

function isImageMode(value: string): value is ImageModeValue {
  return IMAGE_MODE_VALUES.includes(value as ImageModeValue);
}

function buildModelFromForm(): VendorModel | null {
  const base = {
    name: modelForm.name.trim(),
    modelName: modelForm.modelName.trim(),
  };

  if (!base.name || !base.modelName) {
    MessagePlugin.warning('模型名称和模型 ID 不能为空');
    return null;
  }

  if (modelForm.type === 'text') {
    return { ...base, type: 'text', think: modelForm.think };
  }

  if (modelForm.type === 'image') {
    const mode = modelForm.imageModes.filter(isImageMode);
    if (mode.length === 0) {
      MessagePlugin.warning('图片模型需要至少一个生成模式');
      return null;
    }

    return { ...base, type: 'image', mode };
  }

  if (modelForm.type === 'video') {
    const duration = parseCsv(modelForm.durationText).map(Number).filter((value) => Number.isFinite(value) && value > 0);
    const resolution = parseCsv(modelForm.resolutionText);
    if (duration.length === 0 || resolution.length === 0) {
      MessagePlugin.warning('视频模型需要至少一个时长和分辨率');
      return null;
    }

    return {
      ...base,
      type: 'video',
      mode: parseCsv(modelForm.videoModes),
      audio: modelForm.audio === 'optional' ? 'optional' : modelForm.audio === 'true',
      durationResolutionMap: [{ duration, resolution }],
    };
  }

  const voices = modelForm.voicesText
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const [title, voice] = line.split(/:(.+)/);
      return { title: title?.trim() ?? '', voice: voice?.trim() ?? '' };
    })
    .filter((voice) => voice.title && voice.voice);

  if (voices.length === 0) {
    MessagePlugin.warning('TTS 模型需要至少一个音色，格式：名称:voiceId');
    return null;
  }

  return { ...base, type: 'tts', voices };
}

async function saveModel(): Promise<void> {
  const vendor = selectedVendor.value;
  const model = buildModelFromForm();
  if (!vendor || !model) {
    return;
  }

  const response = await window.vtStudio.settings.vendor.saveModel({
    vendorId: vendor.id,
    model,
    originalModelName: modelDialogMode.value === 'edit' ? editingModelName.value : undefined,
  });

  if (!getResponseOk(response)) {
    MessagePlugin.error(response.msg);
    return;
  }

  MessagePlugin.success(modelDialogMode.value === 'edit' ? '模型已保存' : '模型已添加');
  modelDialogVisible.value = false;
  await loadVendors();
}

function confirmDeleteModel(model: VendorModel): void {
  const vendor = selectedVendor.value;
  if (!vendor) {
    return;
  }

  const dialog = DialogPlugin.confirm({
    header: '删除模型',
    body: `将删除 ${model.name}。如果 Agent 或项目仍在引用，系统会阻止删除。`,
    confirmBtn: '删除',
    cancelBtn: '取消',
    theme: 'warning',
    async onConfirm() {
      const response = await window.vtStudio.settings.vendor.deleteModel({ vendorId: vendor.id, modelName: model.modelName });
      if (!getResponseOk(response)) {
        MessagePlugin.error(response.msg);
        return;
      }

      dialog.destroy();
      MessagePlugin.success('模型已删除');
      await loadVendors();
    },
  });
}

function openTestDialog(model: VendorModel): void {
  testModel.value = model;
  testResult.value = '';
  testPrompt.value = '请用一句话回复：模型配置测试成功。';
  testDialogVisible.value = true;
}

async function runModelTest(): Promise<void> {
  const vendor = selectedVendor.value;
  const model = testModel.value;
  if (!vendor || !model) {
    return;
  }

  testing.value = true;
  testResult.value = '';
  try {
    const saved = await saveInputsForTest(vendor);
    if (!saved) {
      return;
    }

    if (model.type === 'text') {
      const response = await window.vtStudio.settings.vendor.testText({
        vendorId: vendor.id,
        modelName: model.modelName,
        prompt: testPrompt.value,
      });
      if (!getResponseOk(response)) {
        testResult.value = response.msg;
        MessagePlugin.error(response.msg);
        return;
      }

      testResult.value = `${response.data.content}\n\n耗时：${response.data.durationMs}ms`;
      return;
    }

    if (model.type === 'image') {
      const response = await window.vtStudio.settings.vendor.testImage({
        vendorId: vendor.id,
        modelName: model.modelName,
        prompt: testPrompt.value,
      });
      testResult.value = getResponseOk(response) ? `图片测试成功：${response.data.filePath}` : response.msg;
      if (!getResponseOk(response)) {
        MessagePlugin.error(response.msg);
      }
      return;
    }

    if (model.type === 'video') {
      const response = await window.vtStudio.settings.vendor.testVideo({
        vendorId: vendor.id,
        modelName: model.modelName,
        mode: Array.isArray(model.mode[0]) ? model.mode[0].join(',') : String(model.mode[0] ?? 'text'),
        prompt: testPrompt.value,
      });
      testResult.value = getResponseOk(response) ? `视频测试成功：${response.data.filePath}` : response.msg;
      if (!getResponseOk(response)) {
        MessagePlugin.error(response.msg);
      }
      return;
    }

    testResult.value = 'TTS 测试入口将在音频配置任务中接入';
  } finally {
    testing.value = false;
  }
}

async function openCodeDialog(mode: 'add' | 'edit'): Promise<void> {
  codeDialogMode.value = mode;
  codeDraft.value = '';
  codeVendorId.value = selectedVendor.value?.id ?? '';

  if (mode === 'edit') {
    const vendor = selectedVendor.value;
    if (!vendor) {
      return;
    }

    const response = await window.vtStudio.settings.vendor.getCode({ vendorId: vendor.id });
    if (!getResponseOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    codeDraft.value = response.data.code;
    codeVendorId.value = response.data.vendorId;
  }

  codeDialogVisible.value = true;
}

async function saveCode(): Promise<void> {
  if (!codeDraft.value.trim()) {
    MessagePlugin.warning('adapter 代码不能为空');
    return;
  }

  codeSaving.value = true;
  try {
    const response =
      codeDialogMode.value === 'add'
        ? await window.vtStudio.settings.vendor.addCode({ code: codeDraft.value })
        : await window.vtStudio.settings.vendor.updateCode({ vendorId: codeVendorId.value, code: codeDraft.value });

    if (!getResponseOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    MessagePlugin.success(codeDialogMode.value === 'add' ? '自定义供应商已添加' : 'adapter 已保存');
    codeDialogVisible.value = false;
    await loadVendors();
    selectedVendorId.value = response.data.vendorId;
  } finally {
    codeSaving.value = false;
  }
}

function confirmDeleteVendor(): void {
  const vendor = selectedVendor.value;
  if (!vendor) {
    return;
  }

  const dialog = DialogPlugin.confirm({
    header: '删除供应商',
    body: `将删除 ${vendor.name} 和对应 adapter 文件。内置供应商不能删除；如果有引用，系统会阻止删除。`,
    confirmBtn: '删除',
    cancelBtn: '取消',
    theme: 'danger',
    async onConfirm() {
      const response = await window.vtStudio.settings.vendor.delete({ vendorId: vendor.id });
      if (!getResponseOk(response)) {
        MessagePlugin.error(response.msg);
        return;
      }

      dialog.destroy();
      MessagePlugin.success('供应商已删除');
      selectedVendorId.value = '';
      await loadVendors();
    },
  });
}

function getCapabilityLabel(capability: string): string {
  const labels: Record<string, string> = {
    text: '文本',
    image: '图片',
    video: '视频',
    tts: 'TTS',
    workflow: '工作流',
  };
  return labels[capability] ?? capability;
}

onMounted(loadVendors);
</script>

<template>
  <section class="settings-section vendor-section">
    <div class="vendor-section-head">
      <div>
        <p class="eyebrow">F-002-003</p>
        <h3>模型服务配置</h3>
      </div>
      <div class="vendor-head-actions">
        <t-button variant="outline" :loading="loading" @click="loadVendors">
          <template #icon><RefreshIcon /></template>
          刷新
        </t-button>
        <t-button variant="outline" @click="advancedVisible = !advancedVisible">
          <template #icon><CodeIcon /></template>
          高级 adapter
        </t-button>
      </div>
    </div>

    <div v-if="advancedVisible" class="vendor-warning">
      自定义 adapter 属于高风险能力，只在主进程受控执行。导入、编辑、删除前必须二次确认；普通供应商只需要填 API Key、Base URL 和模型。
    </div>

    <div class="vendor-layout">
      <aside class="vendor-list" :class="{ 'is-loading': loading }">
        <button
          v-for="vendor in vendors"
          :key="vendor.id"
          type="button"
          class="vendor-list-item"
          :class="{ 'is-active': vendor.id === selectedVendorId }"
          @click="selectedVendorId = vendor.id"
        >
          <span class="vendor-logo">{{ vendor.name.slice(0, 2).toUpperCase() }}</span>
          <span class="vendor-list-main">
            <strong>{{ vendor.name }}</strong>
            <small>{{ vendor.enabled ? '已启用' : '未启用' }} · {{ vendor.builtin ? '内置' : '自定义' }}</small>
          </span>
          <t-tag size="small" :theme="vendor.status === 'ready' ? 'success' : 'warning'" variant="light">{{ vendor.status === 'ready' ? '可用' : '待处理' }}</t-tag>
        </button>
      </aside>

      <main v-if="selectedVendor" class="vendor-detail">
        <div class="vendor-title-row">
          <div>
            <h4>{{ selectedVendor.name }}</h4>
            <p>{{ selectedVendor.description || '暂无说明' }}</p>
          </div>
          <t-switch :model-value="selectedVendor.enabled" size="large" @change="(value) => setEnabled(selectedVendor!, Boolean(value))" />
        </div>

        <div class="vendor-meta-row">
          <t-tag v-for="capability in selectedVendor.capabilities" :key="capability" variant="light">{{ getCapabilityLabel(capability) }}</t-tag>
          <t-tag :theme="selectedVendor.codeReady ? 'success' : 'default'" variant="light">{{ selectedVendor.codeReady ? '自定义 adapter' : '内置 adapter' }}</t-tag>
          <t-tag v-if="selectedVendor.status !== 'ready'" theme="warning" variant="light">{{ selectedVendor.statusText }}</t-tag>
        </div>

        <section class="vendor-panel">
          <div class="vendor-panel-head">
            <strong>连接参数</strong>
            <t-button size="small" theme="primary" :loading="savingInputs" @click="saveInputs">
              <template #icon><SaveIcon /></template>
              保存参数
            </t-button>
          </div>

          <div v-if="selectedVendor.inputs.length" class="vendor-input-grid">
            <label v-for="input in selectedVendor.inputs" :key="input.key" class="vendor-field">
              <span>{{ input.label }}<b v-if="input.required">*</b></span>
              <t-input v-model="inputDraft[input.key]" :type="input.type === 'password' ? 'password' : 'text'" :placeholder="input.placeholder" />
            </label>
          </div>
          <p v-if="selectedVendorBaseUrlHint" class="vendor-input-hint">{{ selectedVendorBaseUrlHint }}</p>
          <t-empty v-else size="small" description="该供应商没有动态输入项" />
        </section>

        <section class="vendor-panel">
          <div class="vendor-panel-head">
            <strong>模型列表</strong>
            <t-button size="small" theme="primary" @click="openCreateModelDialog()">
              <template #icon><AddIcon /></template>
              添加模型
            </t-button>
          </div>

          <div class="model-groups">
            <div v-for="typeOption in MODEL_TYPE_OPTIONS" :key="typeOption.value" class="model-group">
              <div class="model-group-title">
                <span>{{ typeOption.label }}</span>
                <t-button size="small" variant="text" @click="openCreateModelDialog(typeOption.value)">添加</t-button>
              </div>
              <div v-if="modelsByType[typeOption.value].length" class="model-card-grid">
                <article v-for="model in modelsByType[typeOption.value]" :key="model.modelName" class="model-card">
                  <div>
                    <strong>{{ model.name }}</strong>
                    <small>{{ model.modelName }}</small>
                  </div>
                  <div class="model-card-actions">
                    <t-button shape="square" size="small" variant="text" title="测试模型" @click="openTestDialog(model)">
                      <PlayCircleIcon />
                    </t-button>
                    <t-button shape="square" size="small" variant="text" title="编辑模型" @click="openEditModelDialog(model)">
                      <EditIcon />
                    </t-button>
                    <t-button shape="square" size="small" variant="text" theme="danger" title="删除模型" @click="confirmDeleteModel(model)">
                      <DeleteIcon />
                    </t-button>
                  </div>
                </article>
              </div>
              <p v-else class="model-empty">暂无{{ typeOption.label }}模型</p>
            </div>
          </div>
        </section>

        <section v-if="advancedVisible" class="vendor-panel">
          <div class="vendor-panel-head">
            <strong>高级 adapter</strong>
            <div class="vendor-head-actions">
              <t-button size="small" variant="outline" @click="openCodeDialog('add')">
                <template #icon><AddIcon /></template>
                新增自定义供应商
              </t-button>
              <t-button size="small" variant="outline" :disabled="!selectedVendor.codeEditable" @click="openCodeDialog('edit')">
                <template #icon><CodeIcon /></template>
                编辑 adapter
              </t-button>
              <t-button size="small" theme="danger" variant="outline" :disabled="selectedVendor.builtin" @click="confirmDeleteVendor">
                <template #icon><DeleteIcon /></template>
                删除供应商
              </t-button>
            </div>
          </div>
        </section>
      </main>

      <main v-else class="vendor-detail empty-detail">
        <t-empty description="暂无供应商配置" />
      </main>
    </div>
  </section>

  <t-dialog v-model:visible="modelDialogVisible" :header="modelDialogMode === 'edit' ? '编辑模型' : '添加模型'" width="620px" confirm-btn="保存" @confirm="saveModel">
    <t-form :data="modelForm" layout="vertical">
      <div class="model-form-grid">
        <t-form-item label="模型名称">
          <t-input v-model="modelForm.name" placeholder="例如 GPT-4.1 mini" />
        </t-form-item>
        <t-form-item label="模型 ID">
          <t-input v-model="modelForm.modelName" placeholder="例如 gpt-4.1-mini" />
        </t-form-item>
      </div>
      <t-form-item label="模型类型">
        <t-radio-group v-model="modelForm.type">
          <t-radio-button v-for="item in MODEL_TYPE_OPTIONS" :key="item.value" :value="item.value">{{ item.label }}</t-radio-button>
        </t-radio-group>
      </t-form-item>
      <t-form-item v-if="modelForm.type === 'text'" label="思考能力">
        <t-switch v-model="modelForm.think" />
      </t-form-item>
      <t-form-item v-if="modelForm.type === 'image'" label="图片模式">
        <t-checkbox-group v-model="modelForm.imageModes">
          <t-checkbox value="text">文生图</t-checkbox>
          <t-checkbox value="singleImage">单图参考</t-checkbox>
          <t-checkbox value="multiReference">多图参考</t-checkbox>
        </t-checkbox-group>
      </t-form-item>
      <template v-if="modelForm.type === 'video'">
        <t-form-item label="视频模式">
          <t-input v-model="modelForm.videoModes" placeholder="text,singleImage,startEndRequired" />
        </t-form-item>
        <div class="model-form-grid">
          <t-form-item label="时长">
            <t-input v-model="modelForm.durationText" placeholder="5,10" />
          </t-form-item>
          <t-form-item label="分辨率">
            <t-input v-model="modelForm.resolutionText" placeholder="720p,1080p" />
          </t-form-item>
        </div>
        <t-form-item label="输出音频">
          <t-radio-group v-model="modelForm.audio">
            <t-radio-button value="optional">可选</t-radio-button>
            <t-radio-button value="true">固定开启</t-radio-button>
            <t-radio-button value="false">关闭</t-radio-button>
          </t-radio-group>
        </t-form-item>
      </template>
      <t-form-item v-if="modelForm.type === 'tts'" label="音色">
        <t-textarea v-model="modelForm.voicesText" placeholder="Alloy:alloy" :autosize="{ minRows: 3, maxRows: 8 }" />
      </t-form-item>
    </t-form>
  </t-dialog>

  <t-dialog v-model:visible="testDialogVisible" :header="testModel ? `测试 ${MODEL_TYPE_LABELS[testModel.type]}模型` : '测试模型'" width="640px" confirm-btn="开始测试" :confirm-loading="testing" @confirm="runModelTest">
    <t-form layout="vertical">
      <t-form-item label="Prompt">
        <t-textarea v-model="testPrompt" :autosize="{ minRows: 4, maxRows: 8 }" />
      </t-form-item>
      <t-form-item v-if="testResult" label="结果">
        <pre class="test-result">{{ testResult }}</pre>
      </t-form-item>
    </t-form>
  </t-dialog>

  <t-dialog v-model:visible="codeDialogVisible" :header="codeDialogMode === 'add' ? '新增自定义供应商' : '编辑 adapter'" width="860px" confirm-btn="保存 adapter" :confirm-loading="codeSaving" @confirm="saveCode">
    <div class="vendor-warning compact">
      保存前会校验导出结构。失败不会覆盖旧 adapter。请只粘贴可信来源代码。
    </div>
    <t-textarea v-model="codeDraft" class="code-editor" placeholder="粘贴供应商 adapter TypeScript 代码" :autosize="{ minRows: 18, maxRows: 28 }" />
  </t-dialog>
</template>

<style scoped>
.vendor-input-hint {
  margin: 12px 0 0;
  color: var(--td-text-color-secondary);
  font-size: 12px;
  line-height: 1.6;
}
</style>
