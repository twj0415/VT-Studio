<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useLanguageStore } from '@renderer/stores/language';

const languageStore = useLanguageStore();
const { tdesignGlobalConfig } = storeToRefs(languageStore);
</script>

<template>
  <t-config-provider :global-config="tdesignGlobalConfig">
    <RouterView />
  </t-config-provider>
</template>
