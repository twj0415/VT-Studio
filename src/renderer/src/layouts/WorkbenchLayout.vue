<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { storeToRefs } from 'pinia';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { BrowseIcon, FolderOpenIcon, SettingIcon, TaskIcon } from 'tdesign-icons-vue-next';
import { globalMenus, projectMenus } from '@renderer/router/menu';
import { useAppStore } from '@renderer/stores/app';
import type { MenuModule } from '@shared/types/app';

const route = useRoute();
const router = useRouter();
const appStore = useAppStore();
const { t } = useI18n();
const { appInfo, currentProject } = storeToRefs(appStore);

const activeRoute = computed(() => String(route.name ?? 'projects'));
const pageTitle = computed(() => t(`route.${activeRoute.value}`));

const allMenus = computed<MenuModule[]>(() => [...globalMenus, ...projectMenus]);
const activeMenu = computed(() => allMenus.value.find((item) => item.routeName === activeRoute.value));
const localizedGlobalMenus = computed(() => globalMenus.map((menu) => ({ ...menu, title: t(`route.${menu.routeName}`) })));
const localizedProjectMenus = computed(() => projectMenus.map((menu) => ({ ...menu, title: t(`route.${menu.routeName}`) })));
const currentProjectName = computed(() => (currentProject.value?.id === 'placeholder' ? t('common.noProject') : currentProject.value?.name ?? t('common.noProject')));

const iconMap = {
  projects: FolderOpenIcon,
  tasks: TaskIcon,
  settings: SettingIcon,
  default: BrowseIcon,
};

function resolveIcon(routeName: string) {
  return iconMap[routeName as keyof typeof iconMap] ?? iconMap.default;
}

function openMenu(menu: MenuModule): void {
  router.push({ name: menu.routeName });
}

onMounted(() => {
  appStore.loadAppInfo();
});
</script>

<template>
  <div class="app-shell min-h-screen">
    <aside class="sidebar">
      <div class="brand">
        <div class="brand-mark">VT</div>
        <div>
          <h1>VT Studio</h1>
          <p>{{ t('layout.brandTagline') }}</p>
        </div>
      </div>

      <nav class="nav-section">
        <div class="nav-caption">{{ t('common.global') }}</div>
        <button v-for="menu in localizedGlobalMenus" :key="menu.id" class="nav-item" :class="{ 'is-active': activeRoute === menu.routeName }" type="button" @click="openMenu(menu)">
          <component :is="resolveIcon(menu.routeName)" />
          <span>{{ menu.title }}</span>
        </button>
      </nav>

      <nav class="nav-section project-nav">
        <div class="nav-caption">{{ t('common.project') }}</div>
        <button v-for="menu in localizedProjectMenus" :key="menu.id" class="nav-item" :class="{ 'is-active': activeRoute === menu.routeName }" type="button" @click="openMenu(menu)">
          <component :is="resolveIcon(menu.routeName)" />
          <span>{{ menu.title }}</span>
        </button>
      </nav>

      <div class="sidebar-footer">
        <button class="nav-item" :class="{ 'is-active': activeRoute === 'settings' }" type="button" @click="router.push({ name: 'settings' })">
          <SettingIcon />
          <span>{{ t('common.settings') }}</span>
        </button>
      </div>
    </aside>

    <main class="workspace">
      <header class="topbar">
        <div>
          <p class="eyebrow">{{ activeMenu?.id ?? 'VT' }}</p>
          <h2>{{ pageTitle }}</h2>
        </div>
        <div class="project-context">
          <span class="project-state">{{ t('layout.previewState') }}</span>
          <strong>{{ currentProjectName }}</strong>
          <small>{{ appInfo?.name ?? 'VT Studio' }} · {{ appInfo?.version ?? '0.1.0' }}</small>
        </div>
      </header>

      <section class="content-frame">
        <RouterView />
      </section>
    </main>
  </div>
</template>
