<template>
  <section class="view">
    <div class="ws">
      <header class="hdr">
        <div class="back" @click="router.push('/')">←</div>
        <div class="title">{{ projectTitle }}<span class="sub">draft</span></div>
        <div class="tkbadge">{{ t('script.badge') }}</div>
        <div class="sp"></div>
        <div class="hbtn ghost">⚙ {{ t('commonStatus.projectSettings') }}</div>
        <div class="hbtn ghost">⤓ {{ t('commonStatus.export') }}</div>
        <div class="hbtn primary" @click="handleApprove">▶ {{ t('script.enterStoryboard') }}</div>
      </header>

      <div class="ws-body no-list">
        <div class="stage-wrap">
          <div class="stagepane">
            <div class="stage-doc">
              <h2>{{ t('script.title', { count: sceneStore.narrations.length || 0 }) }}</h2>
              <div class="lead">{{ t('script.lead') }}</div>
              <div v-for="item in sceneStore.narrations" :key="item.index" class="narr-item" @click="selectedIndex = item.index">
                <div class="num">{{ item.index.toString().padStart(2, '0') }}</div>
                <div class="txt">{{ item.text }}</div>
                <div class="ed">✎</div>
              </div>
            </div>
          </div>
        </div>

        <aside class="side">
          <div class="side-tabs">
            <div class="side-tab active">⊟ {{ t('script.property') }}</div>
            <div class="side-tab">✦ {{ t('script.aiAssistant') }} <span class="dot"></span></div>
          </div>
          <div class="side-pane">
            <div class="prop">
              <div class="field">
                <label>{{ t('script.narration') }} <span class="lk">{{ selectedNarration?.locked ? `🔒 ${t('script.locked')}` : t('script.unlocked') }}</span></label>
                <n-input v-if="selectedNarration" v-model:value="selectedNarration.text" class="inp" type="textarea" :autosize="{ minRows: 5 }" />
              </div>
              <div class="prop-actions">
                <n-button class="btn btn-primary btn-block" @click="handleApprove">{{ t('script.enterStoryboard') }}</n-button>
                <n-button class="btn btn-ghost btn-block">↻ {{ t('script.regenerateUnlocked') }}</n-button>
                <n-button v-if="selectedNarration" class="btn btn-ghost btn-block" @click="selectedNarration.locked = !selectedNarration.locked">
                  {{ selectedNarration.locked ? t('script.unlockCurrent') : t('script.lockCurrent') }}
                </n-button>
              </div>
            </div>
          </div>
        </aside>
      </div>

      <footer class="status">
        <div class="live"><span class="d"></span>{{ t('script.badge') }}</div>
        <div class="pbar"><i class="w-[18%]"></i></div>
        <span class="frac">{{ t('script.advanced') }}</span>
        <div class="sp"></div>
        <div class="expand">{{ t('commonStatus.taskPanel') }} ▴</div>
      </footer>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { NButton, NInput } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import { useStoryboardStore } from '@/entities/storyboard/store'

const route = useRoute()
const router = useRouter()
const sceneStore = useStoryboardStore()
const { t } = useI18n()

const projectId = String(route.params.projectId)
const projectTitle = '为什么要早睡'
const selectedIndex = ref(1)
const selectedNarration = computed(() => sceneStore.narrations.find((item) => item.index === selectedIndex.value) ?? sceneStore.narrations[0])

onMounted(async () => {
  await sceneStore.loadNarrations(projectId)
  selectedIndex.value = sceneStore.narrations[0]?.index ?? 1
})

async function handleApprove() {
  await router.push(`/projects/${projectId}/workspace/storyboard`)
}
</script>
