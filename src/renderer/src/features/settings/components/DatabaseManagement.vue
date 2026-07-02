<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import { DataBaseIcon, DeleteIcon, DownloadIcon, RefreshIcon, UploadIcon } from 'tdesign-icons-vue-next';
import { DialogPlugin, MessagePlugin } from 'tdesign-vue-next';
import type { DatabaseBackupItem, DatabaseManagementInfo, DatabaseTableInfo } from '@shared/types/database-management';

const loading = ref(false);
const exporting = ref(false);
const importing = ref(false);
const clearingTable = ref(false);
const clearingAll = ref(false);
const checkingTasks = ref(false);
const info = ref<DatabaseManagementInfo | null>(null);
const backups = ref<DatabaseBackupItem[]>([]);
const tables = ref<DatabaseTableInfo[]>([]);

const form = reactive({
  selectedBackupName: '',
  importConfirmText: '',
  selectedTableName: '',
  tableConfirmText: '',
  clearAllConfirmText: '',
});

const selectedTable = computed(() => tables.value.find((table) => table.name === form.selectedTableName) ?? null);
const canImport = computed(() => Boolean(form.selectedBackupName) && form.importConfirmText === '导入数据库');
const canClearTable = computed(() => {
  const table = selectedTable.value;
  return Boolean(table) && !table?.protected && form.tableConfirmText === form.selectedTableName;
});
const canClearAll = computed(() => form.clearAllConfirmText === '清空全部数据');

function isOk(response: { code: number; msg: string }): boolean {
  return response.code === 200;
}

function formatBytes(value: number): string {
  if (value < 1024) {
    return `${value} B`;
  }

  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KB`;
  }

  return `${(value / 1024 / 1024).toFixed(1)} MB`;
}

function formatDate(value: number): string {
  return new Date(value).toLocaleString('zh-CN', { hour12: false });
}

async function loadDatabaseState(): Promise<void> {
  loading.value = true;
  try {
    const [infoResponse, backupsResponse, tablesResponse] = await Promise.all([window.vtStudio.settings.database.info(), window.vtStudio.settings.database.listBackups(), window.vtStudio.settings.database.listTables()]);

    if (!isOk(infoResponse)) {
      MessagePlugin.error(infoResponse.msg);
      return;
    }
    if (!isOk(backupsResponse)) {
      MessagePlugin.error(backupsResponse.msg);
      return;
    }
    if (!isOk(tablesResponse)) {
      MessagePlugin.error(tablesResponse.msg);
      return;
    }

    info.value = infoResponse.data.info;
    backups.value = backupsResponse.data.backups;
    tables.value = tablesResponse.data.tables;

    if (!form.selectedBackupName && backups.value[0]) {
      form.selectedBackupName = backups.value[0].name;
    }
    if (!form.selectedTableName) {
      form.selectedTableName = tables.value.find((table) => !table.protected)?.name ?? '';
    }
  } finally {
    loading.value = false;
  }
}

async function exportBackup(): Promise<void> {
  exporting.value = true;
  try {
    const response = await window.vtStudio.settings.database.export();
    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    MessagePlugin.success(`备份已生成：${response.data.backup.name}`);
    await loadDatabaseState();
  } finally {
    exporting.value = false;
  }
}

async function checkRunningTasks(): Promise<void> {
  checkingTasks.value = true;
  try {
    const response = await window.vtStudio.settings.database.checkRunningTasks();
    if (!isOk(response)) {
      MessagePlugin.error(response.msg);
      return;
    }

    MessagePlugin.info(response.data.runningTaskCount > 0 ? `当前有 ${response.data.runningTaskCount} 个运行中任务` : '当前没有运行中任务');
    await loadDatabaseState();
  } finally {
    checkingTasks.value = false;
  }
}

async function importBackup(): Promise<void> {
  if (!canImport.value) {
    MessagePlugin.warning('请选择备份并输入确认短语：导入数据库');
    return;
  }

  const dialog = DialogPlugin.confirm({
    header: '导入数据库备份',
    body: `将从受控备份 ${form.selectedBackupName} 恢复数据库。当前数据库会先自动备份，运行中任务存在时会被阻止。`,
    confirmBtn: '确认导入',
    cancelBtn: '取消',
    theme: 'danger',
    async onConfirm() {
      importing.value = true;
      try {
        const response = await window.vtStudio.settings.database.import({
          backupName: form.selectedBackupName,
          confirmText: form.importConfirmText,
        });
        if (!isOk(response)) {
          MessagePlugin.error(response.msg);
          return;
        }

        MessagePlugin.success(`导入成功，自动备份：${response.data.autoBackupName}`);
        form.importConfirmText = '';
        dialog.destroy();
        await loadDatabaseState();
      } finally {
        importing.value = false;
      }
    },
  });
}

async function clearSelectedTable(): Promise<void> {
  if (!selectedTable.value) {
    MessagePlugin.warning('请选择要清空的数据表');
    return;
  }
  if (selectedTable.value.protected) {
    MessagePlugin.warning('受保护表不能清空');
    return;
  }
  if (!canClearTable.value) {
    MessagePlugin.warning(`请输入表名确认：${form.selectedTableName}`);
    return;
  }

  const tableName = form.selectedTableName;
  const dialog = DialogPlugin.confirm({
    header: '清空指定表',
    body: `将清空 ${tableName} 表，影响模块：${selectedTable.value.module}。执行前会自动备份，运行中任务存在时会被阻止。`,
    confirmBtn: '确认清空',
    cancelBtn: '取消',
    theme: 'danger',
    async onConfirm() {
      clearingTable.value = true;
      try {
        const response = await window.vtStudio.settings.database.clearTable({
          tableName,
          confirmText: form.tableConfirmText,
        });
        if (!isOk(response)) {
          MessagePlugin.error(response.msg);
          return;
        }

        tables.value = response.data.tables;
        form.tableConfirmText = '';
        MessagePlugin.success(`已清空 ${response.data.tableName}，删除 ${response.data.deleted} 行`);
        dialog.destroy();
        await loadDatabaseState();
      } finally {
        clearingTable.value = false;
      }
    },
  });
}

async function clearAllData(): Promise<void> {
  if (!canClearAll.value) {
    MessagePlugin.warning('请输入确认短语：清空全部数据');
    return;
  }

  const dialog = DialogPlugin.confirm({
    header: '清空全部数据库业务数据',
    body: '该操作会清空业务表并重新初始化默认数据，不删除素材文件或项目目录。执行前会自动备份，运行中任务存在时会被阻止。',
    confirmBtn: '确认清空全部',
    cancelBtn: '取消',
    theme: 'danger',
    async onConfirm() {
      clearingAll.value = true;
      try {
        const response = await window.vtStudio.settings.database.clearAll({
          confirmText: form.clearAllConfirmText,
        });
        if (!isOk(response)) {
          MessagePlugin.error(response.msg);
          return;
        }

        info.value = response.data.info;
        tables.value = response.data.tables;
        form.clearAllConfirmText = '';
        MessagePlugin.success(`已清空并重新初始化，自动备份：${response.data.autoBackupName}`);
        dialog.destroy();
        await loadDatabaseState();
      } finally {
        clearingAll.value = false;
      }
    },
  });
}

defineExpose({ loadDatabaseState });
onMounted(loadDatabaseState);
</script>

<template>
  <section class="database-management-section">
    <div class="database-management-head">
      <div>
        <strong>数据库管理</strong>
        <p>查看 SQLite 状态，导出受控备份，从备份恢复，并在确认后清理数据。</p>
      </div>
      <div class="settings-actions">
        <t-button variant="outline" :loading="checkingTasks" @click="checkRunningTasks">
          <template #icon><DataBaseIcon /></template>
          检查任务
        </t-button>
        <t-button variant="outline" :loading="loading" @click="loadDatabaseState">
          <template #icon><RefreshIcon /></template>
          刷新
        </t-button>
        <t-button theme="primary" :loading="exporting" @click="exportBackup">
          <template #icon><DownloadIcon /></template>
          导出备份
        </t-button>
      </div>
    </div>

    <div class="database-warning">
      数据库导入、清表、清空全部都属于高风险操作；页面不能输入任意文件路径，所有恢复只能从受控备份列表选择。
    </div>

    <div class="database-info-grid">
      <div>
        <span>数据库大小</span>
        <b>{{ info ? formatBytes(info.sizeBytes) : '-' }}</b>
      </div>
      <div>
        <span>表数量</span>
        <b>{{ info?.tableCount ?? '-' }}</b>
      </div>
      <div>
        <span>迁移记录</span>
        <b>{{ info?.migrationCount ?? '-' }}</b>
      </div>
      <div>
        <span>运行中任务</span>
        <b>{{ info?.runningTaskCount ?? '-' }}</b>
      </div>
    </div>

    <div class="database-path-panel">
      <span>数据库路径</span>
      <b>{{ info?.filePath ?? '-' }}</b>
    </div>

    <div class="database-panels">
      <div class="database-panel">
        <div class="database-panel-head">
          <div>
            <strong>备份恢复</strong>
            <p>只显示受控目录内的 `.sqlite` 备份。</p>
          </div>
          <t-tag variant="light">共 {{ backups.length }} 个</t-tag>
        </div>
        <t-select v-model="form.selectedBackupName" placeholder="选择备份">
          <t-option v-for="backup in backups" :key="backup.name" :value="backup.name" :label="backup.name" />
        </t-select>
        <div class="database-backup-list">
          <div v-for="backup in backups" :key="backup.name">
            <span>{{ backup.name }}</span>
            <small>{{ formatBytes(backup.sizeBytes) }} / {{ formatDate(backup.createdAt) }}</small>
          </div>
          <t-empty v-if="backups.length === 0" description="暂无备份" />
        </div>
        <div class="database-danger-row">
          <t-input v-model="form.importConfirmText" placeholder="输入：导入数据库" />
          <t-button theme="danger" variant="outline" :disabled="!canImport" :loading="importing" @click="importBackup">
            <template #icon><UploadIcon /></template>
            导入
          </t-button>
        </div>
      </div>

      <div class="database-panel">
        <div class="database-panel-head">
          <div>
            <strong>表清理</strong>
            <p>受保护表不可清空，高风险表必须输入表名确认。</p>
          </div>
          <t-tag variant="light">共 {{ tables.length }} 张</t-tag>
        </div>
        <t-select v-model="form.selectedTableName" placeholder="选择数据表">
          <t-option v-for="table in tables" :key="table.name" :value="table.name" :label="`${table.name} (${table.rowCount})`" :disabled="table.protected" />
        </t-select>
        <div v-if="selectedTable" class="database-table-detail">
          <div>
            <span>影响模块</span>
            <b>{{ selectedTable.module }}</b>
          </div>
          <div>
            <span>行数</span>
            <b>{{ selectedTable.rowCount }}</b>
          </div>
          <t-tag :theme="selectedTable.protected ? 'danger' : 'warning'" variant="light">
            {{ selectedTable.protected ? '受保护' : '可清空' }}
          </t-tag>
        </div>
        <div class="database-danger-row">
          <t-input v-model="form.tableConfirmText" :placeholder="form.selectedTableName ? `输入：${form.selectedTableName}` : '输入表名确认'" />
          <t-button theme="danger" variant="outline" :disabled="!canClearTable" :loading="clearingTable" @click="clearSelectedTable">
            <template #icon><DeleteIcon /></template>
            清空表
          </t-button>
        </div>
      </div>
    </div>

    <div class="database-clear-all-panel">
      <div>
        <strong>清空全部数据</strong>
        <p>清空业务数据并重新初始化默认数据；不删除数据库文件、素材文件和项目目录。</p>
      </div>
      <div class="database-danger-row">
        <t-input v-model="form.clearAllConfirmText" placeholder="输入：清空全部数据" />
        <t-button theme="danger" :disabled="!canClearAll" :loading="clearingAll" @click="clearAllData">
          <template #icon><DeleteIcon /></template>
          清空全部
        </t-button>
      </div>
    </div>
  </section>
</template>
