<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { EditIcon, RefreshIcon, SaveIcon } from 'tdesign-icons-vue-next';
import { DialogPlugin, MessagePlugin } from 'tdesign-vue-next';
import type { SkillManagementItem, SkillManagementValidationWarning } from '@shared/types/skill-management';

const loading = ref(false);
const loadingContent = ref(false);
const saving = ref(false);
const keyword = ref('');
const skills = ref<SkillManagementItem[]>([]);
const activeSkill = ref<SkillManagementItem | null>(null);
const activeContent = ref('');
const editorVisible = ref(false);
const editorText = ref('');

const activeContentLength = computed(() => activeContent.value.trim().length);
const editorContentLength = computed(() => editorText.value.trim().length);

function isOk(response: { code: number; msg: string }): boolean {
  return response.code === 200;
}

function formatUpdatedAt(value: number): string {
  if (!value) {
    return '未记录';
  }

  return new Date(value).toLocaleString('zh-CN', { hour12: false });
}

function getFileTheme(status: SkillManagementItem['fileStatus']): 'success' | 'danger' {
  return status === 'ready' ? 'success' : 'danger';
}

function getEmbeddingTheme(status: SkillManagementItem['embeddingStatus']): 'success' | 'warning' | 'default' {
  if (status === 'ready') {
    return 'success';
  }

  if (status === 'expired') {
    return 'warning';
  }

  return 'default';
}

function getEmbeddingText(status: SkillManagementItem['embeddingStatus']): string {
  if (status === 'ready') {
    return '向量可用';
  }

  if (status === 'expired') {
    return '需重建向量';
  }

  return '主 Skill';
}

function formatWarnings(warnings: SkillManagementValidationWarning[]): string {
  return warnings.map((item) => `- ${item.message}`).join('\n');
}

function syncSkill(nextSkill: SkillManagementItem): void {
  activeSkill.value = nextSkill;
  skills.value = skills.value.map((item) => (item.id === nextSkill.id ? nextSkill : item));
}

async function loadSkills(): Promise<void> {
  loading.value = true;
  try {
    const response = await window.vtStudio.settings.skill.list({ keyword: keyword.value });
    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    skills.value = response.data.skills;
    if (activeSkill.value && !skills.value.some((item) => item.id === activeSkill.value?.id)) {
      activeSkill.value = null;
      activeContent.value = '';
    }
  } finally {
    loading.value = false;
  }
}

async function selectSkill(skill: SkillManagementItem): Promise<void> {
  activeSkill.value = skill;
  activeContent.value = '';

  if (skill.fileStatus !== 'ready') {
    MessagePlugin.warning('Skill 文件缺失，无法查看内容');
    return;
  }

  loadingContent.value = true;
  try {
    const response = await window.vtStudio.settings.skill.getContent({ id: skill.id });
    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    syncSkill(response.data.skill);
    activeContent.value = response.data.content;
  } finally {
    loadingContent.value = false;
  }
}

function openEditor(): void {
  if (!activeSkill.value) {
    return;
  }

  if (activeSkill.value.fileStatus !== 'ready') {
    MessagePlugin.warning('Skill 文件缺失，不能编辑');
    return;
  }

  editorText.value = activeContent.value;
  editorVisible.value = true;
}

async function saveSkill(force = false): Promise<void> {
  if (!activeSkill.value) {
    return;
  }

  if (!editorText.value.trim()) {
    MessagePlugin.warning('Skill 内容不能为空');
    return;
  }

  saving.value = true;
  try {
    const response = await window.vtStudio.settings.skill.saveContent({
      id: activeSkill.value.id,
      content: editorText.value,
      force,
    });

    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    if (!response.data.saved) {
      const dialog = DialogPlugin.confirm({
        header: 'Skill 结构风险',
        body: `检测到这些风险，继续保存可能影响 Agent 行为：\n\n${formatWarnings(response.data.warnings)}`,
        confirmBtn: '仍然保存',
        cancelBtn: '返回修改',
        theme: 'warning',
        async onConfirm() {
          dialog.destroy();
          await saveSkill(true);
        },
      });
      return;
    }

    if (response.data.skill) {
      syncSkill(response.data.skill);
    }
    activeContent.value = response.data.content ?? editorText.value;
    editorVisible.value = false;
    MessagePlugin.success('Skill 已保存');
    await loadSkills();
  } finally {
    saving.value = false;
  }
}

defineExpose({ loadSkills });
onMounted(loadSkills);
</script>

<template>
  <section class="skill-management-section">
    <div class="skill-management-head">
      <div>
        <strong>Skill 管理</strong>
        <p>编辑本地 Skill Markdown；保存后 references Skill 会标记为需要重建向量。</p>
      </div>
      <div class="settings-actions">
        <t-input v-model="keyword" class="skill-search" clearable placeholder="搜索 Skill" @enter="loadSkills" />
        <t-button variant="outline" :loading="loading" @click="loadSkills">
          <template #icon><RefreshIcon /></template>
          刷新
        </t-button>
      </div>
    </div>

    <div class="skill-management-layout">
      <div class="skill-list">
        <button
          v-for="skill in skills"
          :key="skill.id"
          type="button"
          class="skill-list-item"
          :class="{ active: activeSkill?.id === skill.id }"
          @click="selectSkill(skill)"
        >
          <span>{{ skill.name }}</span>
          <small>{{ skill.path }}</small>
          <div>
            <t-tag size="small" :theme="skill.type === 'main' ? 'primary' : 'default'" variant="light">{{ skill.type }}</t-tag>
            <t-tag size="small" :theme="getFileTheme(skill.fileStatus)" variant="light">{{ skill.fileStatus === 'ready' ? '文件正常' : '文件缺失' }}</t-tag>
          </div>
        </button>
        <p v-if="skills.length === 0" class="model-empty">{{ loading ? '正在读取 Skill...' : '没有匹配的 Skill' }}</p>
      </div>

      <div class="skill-detail">
        <template v-if="activeSkill">
          <div class="skill-detail-head">
            <div>
              <strong>{{ activeSkill.name }}</strong>
              <small>{{ activeSkill.path }}</small>
            </div>
            <t-button variant="outline" :disabled="activeSkill.fileStatus !== 'ready'" @click="openEditor">
              <template #icon><EditIcon /></template>
              编辑
            </t-button>
          </div>

          <div class="skill-meta-grid">
            <div>
              <span>类型</span>
              <b>{{ activeSkill.type }}</b>
            </div>
            <div>
              <span>文件</span>
              <b>{{ activeSkill.fileStatus === 'ready' ? '正常' : '缺失' }}</b>
            </div>
            <div>
              <span>向量</span>
              <t-tag :theme="getEmbeddingTheme(activeSkill.embeddingStatus)" variant="light">{{ getEmbeddingText(activeSkill.embeddingStatus) }}</t-tag>
            </div>
            <div>
              <span>字符数</span>
              <b>{{ activeContentLength }}</b>
            </div>
          </div>

          <p class="skill-description">{{ activeSkill.description || '无描述' }}</p>
          <p v-if="activeSkill.attributions.length > 0" class="skill-attributions">归属：{{ activeSkill.attributions.join('、') }}</p>
          <p class="skill-updated">更新：{{ formatUpdatedAt(activeSkill.updatedAt) }}</p>

          <pre v-if="activeContent" class="skill-content-preview">{{ activeContent }}</pre>
          <p v-else class="model-empty">{{ loadingContent ? '正在读取内容...' : '请选择文件正常的 Skill 查看内容。' }}</p>
        </template>
        <p v-else class="model-empty">请选择一个 Skill。</p>
      </div>
    </div>

    <t-dialog v-model:visible="editorVisible" :header="activeSkill ? `编辑 ${activeSkill.name}` : '编辑 Skill'" width="920px" confirm-btn="保存" :confirm-loading="saving" @confirm="() => saveSkill(false)">
      <div class="skill-editor">
        <div class="skill-editor-meta">
          <div>
            <span>路径</span>
            <b>{{ activeSkill?.path }}</b>
          </div>
          <div>
            <span>字符数</span>
            <b>{{ editorContentLength }}</b>
          </div>
        </div>
        <t-textarea v-model="editorText" class="code-editor skill-textarea" placeholder="请输入 Skill Markdown" :autosize="{ minRows: 22, maxRows: 34 }" />
      </div>

      <template #footer>
        <div class="prompt-dialog-footer">
          <t-button variant="outline" @click="editorVisible = false">取消</t-button>
          <t-button theme="primary" :loading="saving" @click="saveSkill(false)">
            <template #icon><SaveIcon /></template>
            保存
          </t-button>
        </div>
      </template>
    </t-dialog>
  </section>
</template>
