<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { FolderOpenIcon, RefreshIcon } from 'tdesign-icons-vue-next';
import { useI18n } from 'vue-i18n';
import { MessagePlugin } from 'tdesign-vue-next';
import type { FileManagementDirectoryGroup, FileManagementDirectoryItem } from '@shared/types/file-management';

interface DirectoryGroupMeta {
  key: FileManagementDirectoryGroup;
  title: string;
  description: string;
}

const GROUPS: DirectoryGroupMeta[] = [
  { key: 'common', title: 'files.groups.common.title', description: 'files.groups.common.description' },
  { key: 'diagnostic', title: 'files.groups.diagnostic.title', description: 'files.groups.diagnostic.description' },
  { key: 'advanced', title: 'files.groups.advanced.title', description: 'files.groups.advanced.description' },
];

const loading = ref(false);
const openingKey = ref('');
const directories = ref<FileManagementDirectoryItem[]>([]);
const { t } = useI18n();

function isOk(response: { code: number; msg: string }): boolean {
  return response.code === 200;
}

const groupedDirectories = computed(() =>
  GROUPS.map((group) => ({
    ...group,
    items: directories.value.filter((item) => item.group === group.key),
  })),
);

async function loadDirectories(): Promise<void> {
  loading.value = true;
  try {
    const response = await window.vtStudio.settings.files.listOpenableDirs();
    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    directories.value = response.data.directories;
  } finally {
    loading.value = false;
  }
}

async function openDirectory(directory: FileManagementDirectoryItem): Promise<void> {
  openingKey.value = directory.key;
  try {
    const response = await window.vtStudio.settings.files.openDir({ key: directory.key });
    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    MessagePlugin.success(response.data.created ? t('files.createdAndOpened', { name: response.data.directory.name }) : t('files.opened', { name: response.data.directory.name }));
    await loadDirectories();
  } finally {
    openingKey.value = '';
  }
}

defineExpose({ loadDirectories });
onMounted(loadDirectories);
</script>

<template>
  <section class="file-management-section">
    <div class="file-management-head">
      <div>
        <strong>{{ t('files.title') }}</strong>
        <p>{{ t('files.hint') }}</p>
      </div>
      <div class="settings-actions">
        <t-button variant="outline" :loading="loading" @click="loadDirectories">
          <template #icon><RefreshIcon /></template>
          {{ t('files.refresh') }}
        </t-button>
      </div>
    </div>

    <div class="file-management-warning">
      {{ t('files.warning') }}
    </div>

    <div class="file-management-groups">
      <div v-for="group in groupedDirectories" :key="group.key" class="file-management-group">
        <div class="file-management-group-head">
          <div>
            <strong>{{ t(group.title) }}</strong>
            <p>{{ t(group.description) }}</p>
          </div>
          <t-tag variant="light">{{ t('files.total', { count: group.items.length }) }}</t-tag>
        </div>

        <div class="file-management-grid">
          <div v-for="directory in group.items" :key="directory.key" class="file-management-card">
            <div class="file-management-card-head">
              <div>
                <strong>{{ directory.name }}</strong>
                <small>{{ directory.description }}</small>
              </div>
              <t-tag :theme="directory.exists ? 'success' : 'warning'" variant="light">
                {{ directory.exists ? t('files.exists') : t('files.pending') }}
              </t-tag>
            </div>
            <div class="file-management-path">{{ directory.path }}</div>
            <div class="file-management-card-foot">
              <span>{{ directory.autoCreate ? t('files.autoCreate') : t('files.openOnly') }}</span>
              <t-button theme="primary" variant="outline" :loading="openingKey === directory.key" @click="openDirectory(directory)">
                <template #icon><FolderOpenIcon /></template>
                {{ t('files.open') }}
              </t-button>
            </div>
          </div>
          <t-empty v-if="group.items.length === 0" :description="t('files.empty')" />
        </div>
      </div>
    </div>
  </section>
</template>
