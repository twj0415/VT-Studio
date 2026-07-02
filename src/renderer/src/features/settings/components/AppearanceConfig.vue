<script setup lang="ts">
import { computed } from 'vue';
import { storeToRefs } from 'pinia';
import { RollbackIcon } from 'tdesign-icons-vue-next';
import { useI18n } from 'vue-i18n';
import { useAppearanceStore } from '@renderer/stores/appearance';
import {
  APPEARANCE_PRESETS,
  type AppearanceFontSize,
  type AppearanceMode,
} from '../appearance/theme';

const appearanceStore = useAppearanceStore();
const { t } = useI18n();
const { mode, themePresetId, fontSize } = storeToRefs(appearanceStore);

const modeOptions: Array<{ value: AppearanceMode; label: string }> = [
  { value: 'auto', label: 'appearance.modeAuto' },
  { value: 'light', label: 'appearance.modeLight' },
  { value: 'dark', label: 'appearance.modeDark' },
];

const fontSizeOptions: AppearanceFontSize[] = [12, 13, 14, 16, 18, 20, 22];
const resolvedModeText = computed(() => (appearanceStore.resolvedMode === 'dark' ? t('appearance.modeResolvedDark') : t('appearance.modeResolvedLight')));
const activePreset = computed(() => APPEARANCE_PRESETS.find((preset) => preset.id === themePresetId.value) ?? APPEARANCE_PRESETS[0]);

function updateMode(nextMode: AppearanceMode): void {
  appearanceStore.setMode(nextMode);
}

function updatePreset(nextPresetId: (typeof APPEARANCE_PRESETS)[number]['id']): void {
  appearanceStore.setThemePresetId(nextPresetId);
}

function updateFontSize(nextFontSize: AppearanceFontSize): void {
  appearanceStore.setFontSize(nextFontSize);
}
</script>

<template>
  <section class="appearance-section">
    <div class="appearance-head">
      <div>
        <strong>{{ t('appearance.title') }}</strong>
        <p>{{ t('appearance.hint') }}</p>
      </div>
      <div class="settings-actions">
        <t-button variant="outline" theme="warning" @click="appearanceStore.restoreDefault()">
          <template #icon><RollbackIcon /></template>
          {{ t('appearance.restore') }}
        </t-button>
      </div>
    </div>

    <div class="appearance-status-row">
      <div>
        <span>{{ t('appearance.currentTheme') }}</span>
        <b>{{ t(`appearance.presets.${activePreset.id}.name`) }}</b>
      </div>
      <t-tag variant="light">{{ resolvedModeText }}</t-tag>
    </div>

    <div class="appearance-panel">
      <div class="appearance-block">
        <span class="appearance-label">{{ t('appearance.modeLabel') }}</span>
        <t-radio-group :model-value="mode" variant="default-filled" @update:model-value="updateMode">
          <t-radio-button v-for="option in modeOptions" :key="option.value" :value="option.value">{{ t(option.label) }}</t-radio-button>
        </t-radio-group>
      </div>

      <div class="appearance-block">
        <span class="appearance-label">{{ t('appearance.presetLabel') }}</span>
        <div class="appearance-theme-grid">
          <button
            v-for="preset in APPEARANCE_PRESETS"
            :key="preset.id"
            class="appearance-theme-card"
            :class="{ 'is-active': themePresetId === preset.id }"
            type="button"
            @click="updatePreset(preset.id)"
          >
            <div class="appearance-theme-card-head">
              <div>
                <strong>{{ t(`appearance.presets.${preset.id}.name`) }}</strong>
                <small>{{ t(`appearance.presets.${preset.id}.description`) }}</small>
              </div>
              <t-tag v-if="themePresetId === preset.id" theme="success" variant="light">{{ t('appearance.currentTag') }}</t-tag>
            </div>
            <div class="appearance-theme-swatches">
              <span v-for="color in preset.preview" :key="color" :style="{ background: color }" />
            </div>
          </button>
        </div>
      </div>

      <div class="appearance-block">
        <span class="appearance-label">{{ t('appearance.fontSizeLabel') }}</span>
        <t-radio-group :model-value="fontSize" variant="default-filled" @update:model-value="updateFontSize">
          <t-radio-button v-for="size in fontSizeOptions" :key="size" :value="size">{{ size }}</t-radio-button>
        </t-radio-group>
      </div>
    </div>
  </section>
</template>
