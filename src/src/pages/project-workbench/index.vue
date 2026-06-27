<template>
  <section class="view h-full w-full min-w-0 overflow-hidden bg-page text-primary">
    <header class="flex h-16 w-full flex-none items-center gap-vt-4 border-b border-border bg-panel px-vt-6">
      <button type="button" class="flex items-center gap-vt-2 text-sm text-secondary transition hover:text-primary" @click="router.push('/')">
        <span aria-hidden="true">←</span>
        <span>{{ t('common.back') }}</span>
      </button>
      <div class="min-w-0">
        <h2 class="truncate text-base font-semibold">{{ projectTitle }}</h2>
        <div class="truncate text-xs text-muted">{{ t('workbench.subtitle') }}</div>
      </div>
      <span class="ml-auto rounded-vt-sm border border-accent-line bg-accent-soft px-vt-3 py-vt-1 text-xs font-medium text-accent">{{ t('workbench.projectContextBadge') }}</span>
    </header>

    <main class="min-h-0 flex-1 overflow-y-auto">
      <div class="grid gap-vt-5 p-vt-5 lg:p-vt-6 xl:grid-cols-[minmax(0,1.35fr)_360px]">
        <section class="flex min-w-0 flex-col gap-vt-5">
          <article class="rounded-vt-md border border-border bg-card p-vt-5 shadow-vt-md">
            <div class="flex flex-wrap items-start justify-between gap-vt-4">
              <div class="min-w-0">
                <div class="text-xs font-medium uppercase tracking-[0.12em] text-accent">{{ t('workbench.contentCreationEyebrow') }}</div>
                <h1 class="mt-vt-2 truncate text-2xl font-semibold text-primary">{{ t('workbench.contentCreationTitle') }}</h1>
                <p class="mt-vt-2 max-w-3xl text-sm leading-6 text-secondary">{{ t('workbench.contentCreationDesc') }}</p>
              </div>
              <n-button class="btn btn-primary" @click="router.push(`/projects/${projectId}/workspace/storyboard`)">{{ t('workbench.continueStoryboard') }} →</n-button>
            </div>

            <div class="mt-vt-5 grid gap-vt-3 md:grid-cols-2 xl:grid-cols-4">
              <div v-for="item in contentSummary" :key="item.key" class="rounded-vt-sm border border-border bg-page p-vt-3">
                <div class="text-[11px] text-muted">{{ t(`workbench.summary.${item.key}`) }}</div>
                <div class="mt-vt-1 truncate text-sm font-medium text-primary">{{ item.value }}</div>
              </div>
            </div>
          </article>

          <section class="grid gap-vt-4 md:grid-cols-2">
            <button v-for="entry in workspaceEntries" :key="entry.key" type="button" class="group rounded-vt-md border border-border bg-card p-vt-5 text-left shadow-vt-md transition hover:-translate-y-0.5 hover:border-border-strong hover:bg-card-hover" @click="router.push(entry.path)">
              <div class="flex items-center justify-between gap-vt-3">
                <span class="grid size-9 place-items-center rounded-vt-sm border border-border bg-page text-sm font-semibold text-accent">{{ entry.step }}</span>
                <span class="text-xs text-muted">{{ t(`workbench.entryStatus.${entry.status}`) }}</span>
              </div>
              <h2 class="mt-vt-4 text-base font-semibold text-primary">{{ t(`workbench.entries.${entry.key}.title`) }}</h2>
              <p class="mt-vt-2 text-sm leading-6 text-secondary">{{ t(`workbench.entries.${entry.key}.desc`) }}</p>
              <div class="mt-vt-4 text-sm font-medium text-accent">{{ t('commonStatus.enter') }} →</div>
            </button>
          </section>
        </section>

        <aside class="flex min-w-0 flex-col gap-vt-5">
          <section class="rounded-vt-md border border-border bg-card p-vt-5 shadow-vt-md">
            <div class="flex items-center justify-between gap-vt-3">
              <h2 class="text-sm font-semibold text-primary">{{ t('workbench.taskSummaryTitle') }}</h2>
              <n-button size="small" secondary :disabled="!canCancelLatestTask || isCancellingTask" :loading="isCancellingTask" @click="handleCancelLatestTask">{{ t('workbench.cancelTask') }}</n-button>
            </div>
            <div class="mt-vt-4 rounded-vt-sm border border-border bg-page p-vt-4">
              <div class="flex items-center gap-vt-2 text-sm text-secondary">
                <span class="lifecycle" :class="lifecycleClass"><span class="d"></span>{{ lifecycleText }}</span>
                <span class="text-muted">·</span>
                <span>{{ latestTaskStatus }}</span>
              </div>
              <p class="mt-vt-3 text-sm leading-6 text-secondary">{{ latestTaskSummary }}</p>
            </div>
          </section>

          <section class="rounded-vt-md border border-border bg-card p-vt-5 shadow-vt-md">
            <h2 class="text-sm font-semibold text-primary">{{ t('workbench.resourceUsage.title') }}</h2>
            <div class="mt-vt-4 grid gap-vt-2 text-sm">
              <div v-for="item in resourceUsageSummary" :key="item.key" class="flex items-center justify-between border-b border-border py-vt-2 last:border-b-0">
                <span class="text-secondary">{{ t(`workbench.resourceUsage.${item.key}`) }}</span>
                <span class="font-medium text-primary">{{ item.value }}</span>
              </div>
            </div>
            <p class="mt-vt-3 text-xs leading-5 text-muted">{{ t('workbench.resourceUsage.sourceHint') }}</p>
          </section>

          <section class="rounded-vt-md border border-border bg-card p-vt-5 shadow-vt-md">
            <div class="flex items-center justify-between gap-vt-3">
              <h2 class="text-sm font-semibold text-primary">{{ t('workbench.projectBibleTitle') }}</h2>
              <n-button size="small" secondary :loading="isSavingStyleBible || isSavingCharacterBible || isSavingLocationBible" @click="handleSaveStyleBible">{{ t('common.save') }}</n-button>
            </div>
            <div class="mt-vt-4 grid gap-vt-2 text-sm">
              <div class="flex items-center justify-between border-b border-border py-vt-2">
                <span class="text-secondary">{{ t('workbench.bible.style') }}</span>
                <span class="text-muted">{{ projectDetail?.styleBible?.name ?? t('workbench.bible.pending') }}</span>
              </div>
              <div class="flex items-center justify-between border-b border-border py-vt-2">
                <span class="text-secondary">{{ t('workbench.bible.characters') }}</span>
                <span class="text-muted">{{ t('workbench.count', { count: projectDetail?.characterBibles.length ?? 0 }) }}</span>
              </div>
              <div class="flex items-center justify-between py-vt-2">
                <span class="text-secondary">{{ t('workbench.bible.locations') }}</span>
                <span class="text-muted">{{ t('workbench.count', { count: projectDetail?.locationBibles.length ?? 0 }) }}</span>
              </div>
            </div>
            <div class="mt-vt-4 grid gap-vt-3">
              <label class="grid gap-vt-1 text-xs text-muted">
                <span>{{ t('workbench.styleBible.preset') }}</span>
                <n-select v-model:value="selectedStylePresetId" size="small" :options="stylePresetOptions" clearable :placeholder="t('workbench.styleBible.presetPlaceholder')" @update:value="handleApplyStylePreset" />
              </label>
              <label class="grid gap-vt-1 text-xs text-muted">
                <span>{{ t('workbench.styleBible.name') }}</span>
                <n-input v-model:value="styleForm.name" size="small" />
              </label>
              <label class="grid gap-vt-1 text-xs text-muted">
                <span>{{ t('workbench.styleBible.stylePrompt') }}</span>
                <n-input v-model:value="styleForm.stylePrompt" size="small" type="textarea" :autosize="{ minRows: 3, maxRows: 5 }" />
              </label>
              <label class="grid gap-vt-1 text-xs text-muted">
                <span>{{ t('workbench.styleBible.colorPalette') }}</span>
                <n-input v-model:value="styleForm.colorPaletteText" size="small" :placeholder="t('workbench.styleBible.colorPalettePlaceholder')" />
              </label>
              <label class="grid gap-vt-1 text-xs text-muted">
                <span>{{ t('workbench.styleBible.lighting') }}</span>
                <n-input v-model:value="styleForm.lighting" size="small" />
              </label>
              <label class="grid gap-vt-1 text-xs text-muted">
                <span>{{ t('workbench.styleBible.composition') }}</span>
                <n-input v-model:value="styleForm.composition" size="small" />
              </label>
              <label class="grid gap-vt-1 text-xs text-muted">
                <span>{{ t('workbench.styleBible.negativePrompt') }}</span>
                <n-input v-model:value="styleForm.negativePrompt" size="small" type="textarea" :autosize="{ minRows: 2, maxRows: 4 }" />
              </label>
              <div class="rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-muted">
                <div class="font-medium text-secondary">{{ t('workbench.styleBible.referenceImage') }}</div>
                <div class="mt-vt-1 break-all">{{ styleForm.referenceImagePath || t('workbench.styleBible.noReferenceImage') }}</div>
                <div class="mt-vt-3 grid gap-vt-2">
                  <input ref="styleReferenceFileInput" class="hidden" type="file" accept="image/*" @change="handleStyleReferenceFilePicked" />
                  <n-input v-model:value="styleReferenceSourcePath" size="small" :placeholder="t('workbench.styleBible.referenceSourcePlaceholder')" />
                  <div class="flex flex-wrap gap-vt-2">
                    <n-button size="small" @click="styleReferenceFileInput?.click()">{{ t('workbench.styleBible.chooseReference') }}</n-button>
                    <n-button size="small" secondary :loading="isImportingStyleReference" :disabled="!styleReferenceSourcePath.trim()" @click="handleImportStyleReference">{{ t('workbench.styleBible.importReference') }}</n-button>
                    <n-button size="small" secondary :loading="isAnalyzingStyleReference" :disabled="!styleForm.referenceImagePath" @click="handleAnalyzeStyleReference">{{ t('workbench.styleBible.analyzeReference') }}</n-button>
                  </div>
                </div>
              </div>
              <div v-if="styleReferenceAnalysis" class="rounded-vt-sm border border-accent-line bg-accent-soft p-vt-3 text-xs leading-5">
                <div class="flex items-start justify-between gap-vt-3">
                  <div class="min-w-0">
                    <div class="font-medium text-primary">{{ t('workbench.styleBible.analysisTitle') }}</div>
                    <div class="mt-vt-1 break-all text-muted">{{ styleReferenceAnalysis.referenceImagePath }}</div>
                  </div>
                  <n-button size="tiny" secondary @click="handleApplyStyleAnalysis">{{ t('workbench.styleBible.applyAnalysis') }}</n-button>
                </div>
                <div class="mt-vt-3 grid gap-vt-2 text-secondary">
                  <div>
                    <span class="font-medium text-primary">{{ t('workbench.styleBible.stylePrompt') }}：</span>
                    <span>{{ styleReferenceAnalysis.stylePrompt }}</span>
                  </div>
                  <div>
                    <span class="font-medium text-primary">{{ t('workbench.styleBible.colorPalette') }}：</span>
                    <span>{{ styleReferenceAnalysis.colorPalette.join('、') }}</span>
                  </div>
                  <div>
                    <span class="font-medium text-primary">{{ t('workbench.styleBible.lighting') }}：</span>
                    <span>{{ styleReferenceAnalysis.lighting }}</span>
                  </div>
                  <div>
                    <span class="font-medium text-primary">{{ t('workbench.styleBible.composition') }}：</span>
                    <span>{{ styleReferenceAnalysis.composition }}</span>
                  </div>
                  <div>
                    <span class="font-medium text-primary">{{ t('workbench.styleBible.negativePrompt') }}：</span>
                    <span>{{ styleReferenceAnalysis.negativePromptSuggestion }}</span>
                  </div>
                </div>
                <div v-if="styleReferenceAnalysis.warnings.length" class="mt-vt-3 border-t border-accent-line pt-vt-2 text-muted">
                  {{ t('workbench.styleBible.analysisWarnings') }}：{{ styleReferenceAnalysis.warnings.map(styleAnalysisWarningLabel).join('、') }}
                </div>
              </div>
              <label class="flex items-center gap-vt-2 text-xs text-muted">
                <n-switch v-model:value="styleForm.saveAsPreset" size="small" />
                <span>{{ t('workbench.styleBible.saveAsPreset') }}</span>
              </label>
            </div>

            <div class="mt-vt-5 border-t border-border pt-vt-4">
              <div class="flex items-center justify-between gap-vt-3">
                <div>
                  <div class="text-xs font-semibold text-primary">{{ t('workbench.characterBible.title') }}</div>
                  <div class="mt-vt-1 text-xs text-muted">{{ t('workbench.characterBible.desc') }}</div>
                </div>
                <n-button size="tiny" secondary @click="handleNewCharacterBible">{{ t('workbench.characterBible.newCharacter') }}</n-button>
              </div>
              <div class="mt-vt-3 flex flex-wrap gap-vt-2">
                <button v-for="character in characterBibles" :key="character.characterId" type="button" class="rounded-vt-sm border px-vt-2 py-vt-1 text-xs transition" :class="character.characterId === selectedCharacterId ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-page text-secondary hover:border-border-strong hover:text-primary'" @click="selectCharacterBible(character.characterId)">
                  {{ character.name }}
                </button>
                <span v-if="characterBibles.length === 0" class="text-xs text-muted">{{ t('workbench.characterBible.empty') }}</span>
              </div>
              <div class="mt-vt-3 grid gap-vt-3">
                <label class="grid gap-vt-1 text-xs text-muted">
                  <span>{{ t('workbench.characterBible.characterId') }}</span>
                  <n-input v-model:value="characterForm.characterId" size="small" :disabled="Boolean(selectedCharacterId)" :placeholder="t('workbench.characterBible.characterIdPlaceholder')" />
                </label>
                <label class="grid gap-vt-1 text-xs text-muted">
                  <span>{{ t('workbench.characterBible.name') }}</span>
                  <n-input v-model:value="characterForm.name" size="small" />
                </label>
                <label class="grid gap-vt-1 text-xs text-muted">
                  <span>{{ t('workbench.characterBible.alias') }}</span>
                  <n-input v-model:value="characterForm.aliasText" size="small" :placeholder="t('workbench.characterBible.aliasPlaceholder')" />
                </label>
                <div class="grid gap-vt-2 md:grid-cols-2">
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('workbench.characterBible.age') }}</span>
                    <n-input v-model:value="characterForm.age" size="small" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('workbench.characterBible.gender') }}</span>
                    <n-input v-model:value="characterForm.gender" size="small" />
                  </label>
                </div>
                <label class="grid gap-vt-1 text-xs text-muted">
                  <span>{{ t('workbench.characterBible.appearance') }}</span>
                  <n-input v-model:value="characterForm.appearance" size="small" type="textarea" :autosize="{ minRows: 2, maxRows: 4 }" />
                </label>
                <label class="grid gap-vt-1 text-xs text-muted">
                  <span>{{ t('workbench.characterBible.clothing') }}</span>
                  <n-input v-model:value="characterForm.clothing" size="small" type="textarea" :autosize="{ minRows: 2, maxRows: 4 }" />
                </label>
                <label class="grid gap-vt-1 text-xs text-muted">
                  <span>{{ t('workbench.characterBible.personality') }}</span>
                  <n-input v-model:value="characterForm.personality" size="small" />
                </label>
                <label class="grid gap-vt-1 text-xs text-muted">
                  <span>{{ t('workbench.characterBible.visualPrompt') }}</span>
                  <n-input v-model:value="characterForm.visualPrompt" size="small" type="textarea" :autosize="{ minRows: 2, maxRows: 4 }" />
                </label>
                <label class="grid gap-vt-1 text-xs text-muted">
                  <span>{{ t('workbench.characterBible.negativePrompt') }}</span>
                  <n-input v-model:value="characterForm.negativePrompt" size="small" type="textarea" :autosize="{ minRows: 2, maxRows: 4 }" />
                </label>
                <div class="rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-muted">
                  <div class="font-medium text-secondary">{{ t('workbench.characterBible.referenceImage') }}</div>
                  <div class="mt-vt-1 break-all">{{ characterForm.referenceImagePath || t('workbench.characterBible.noReferenceImage') }}</div>
                  <div class="mt-vt-3 grid gap-vt-2">
                    <input ref="characterReferenceFileInput" class="hidden" type="file" accept="image/*" @change="handleCharacterReferenceFilePicked" />
                    <n-input v-model:value="characterReferenceSourcePath" size="small" :placeholder="t('workbench.characterBible.referenceSourcePlaceholder')" />
                    <div class="flex flex-wrap gap-vt-2">
                      <n-button size="small" @click="characterReferenceFileInput?.click()">{{ t('workbench.characterBible.chooseReference') }}</n-button>
                      <n-button size="small" secondary :loading="isImportingCharacterReference" :disabled="!selectedCharacterId || !characterReferenceSourcePath.trim()" @click="handleImportCharacterReference">{{ t('workbench.characterBible.importReference') }}</n-button>
                    </div>
                  </div>
                </div>
                <div class="flex flex-wrap justify-end gap-vt-2">
                  <n-button v-if="selectedCharacterId" size="small" tertiary type="error" :loading="isDeletingCharacterBible" @click="handleDeleteCharacterBible">{{ t('workbench.characterBible.deleteCharacter') }}</n-button>
                  <n-button size="small" secondary :loading="isSavingCharacterBible" @click="handleSaveCharacterBible">{{ t('workbench.characterBible.saveCharacter') }}</n-button>
                </div>
              </div>
            </div>

            <div class="mt-vt-5 border-t border-border pt-vt-4">
              <div class="flex items-center justify-between gap-vt-3">
                <div>
                  <div class="text-xs font-semibold text-primary">{{ t('workbench.locationBible.title') }}</div>
                  <div class="mt-vt-1 text-xs text-muted">{{ t('workbench.locationBible.desc') }}</div>
                </div>
                <n-button size="tiny" secondary @click="handleNewLocationBible">{{ t('workbench.locationBible.newLocation') }}</n-button>
              </div>
              <div class="mt-vt-3 flex flex-wrap gap-vt-2">
                <button v-for="location in locationBibles" :key="location.locationId" type="button" class="rounded-vt-sm border px-vt-2 py-vt-1 text-xs transition" :class="location.locationId === selectedLocationId ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-page text-secondary hover:border-border-strong hover:text-primary'" @click="selectLocationBible(location.locationId)">
                  {{ location.name }}
                </button>
                <span v-if="locationBibles.length === 0" class="text-xs text-muted">{{ t('workbench.locationBible.empty') }}</span>
              </div>
              <div class="mt-vt-3 grid gap-vt-3">
                <label class="grid gap-vt-1 text-xs text-muted">
                  <span>{{ t('workbench.locationBible.locationId') }}</span>
                  <n-input v-model:value="locationForm.locationId" size="small" :disabled="Boolean(selectedLocationId)" :placeholder="t('workbench.locationBible.locationIdPlaceholder')" />
                </label>
                <label class="grid gap-vt-1 text-xs text-muted">
                  <span>{{ t('workbench.locationBible.name') }}</span>
                  <n-input v-model:value="locationForm.name" size="small" />
                </label>
                <label class="grid gap-vt-1 text-xs text-muted">
                  <span>{{ t('workbench.locationBible.spaceDescription') }}</span>
                  <n-input v-model:value="locationForm.spaceDescription" size="small" type="textarea" :autosize="{ minRows: 2, maxRows: 4 }" />
                </label>
                <div class="grid gap-vt-2 md:grid-cols-2">
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('workbench.locationBible.lighting') }}</span>
                    <n-input v-model:value="locationForm.lighting" size="small" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('workbench.locationBible.timeOfDay') }}</span>
                    <n-input v-model:value="locationForm.timeOfDay" size="small" />
                  </label>
                </div>
                <label class="grid gap-vt-1 text-xs text-muted">
                  <span>{{ t('workbench.locationBible.props') }}</span>
                  <n-input v-model:value="locationForm.propsText" size="small" :placeholder="t('workbench.locationBible.propsPlaceholder')" />
                </label>
                <label class="grid gap-vt-1 text-xs text-muted">
                  <span>{{ t('workbench.locationBible.visualPrompt') }}</span>
                  <n-input v-model:value="locationForm.visualPrompt" size="small" type="textarea" :autosize="{ minRows: 2, maxRows: 4 }" />
                </label>
                <label class="grid gap-vt-1 text-xs text-muted">
                  <span>{{ t('workbench.locationBible.negativePrompt') }}</span>
                  <n-input v-model:value="locationForm.negativePrompt" size="small" type="textarea" :autosize="{ minRows: 2, maxRows: 4 }" />
                </label>
                <div class="rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-muted">
                  <div class="font-medium text-secondary">{{ t('workbench.locationBible.referenceImage') }}</div>
                  <div class="mt-vt-1 break-all">{{ locationForm.referenceImagePath || t('workbench.locationBible.noReferenceImage') }}</div>
                  <div class="mt-vt-3 grid gap-vt-2">
                    <input ref="locationReferenceFileInput" class="hidden" type="file" accept="image/*" @change="handleLocationReferenceFilePicked" />
                    <n-input v-model:value="locationReferenceSourcePath" size="small" :placeholder="t('workbench.locationBible.referenceSourcePlaceholder')" />
                    <div class="flex flex-wrap gap-vt-2">
                      <n-button size="small" @click="locationReferenceFileInput?.click()">{{ t('workbench.locationBible.chooseReference') }}</n-button>
                      <n-button size="small" secondary :loading="isImportingLocationReference" :disabled="!selectedLocationId || !locationReferenceSourcePath.trim()" @click="handleImportLocationReference">{{ t('workbench.locationBible.importReference') }}</n-button>
                    </div>
                  </div>
                </div>
                <div class="flex flex-wrap justify-end gap-vt-2">
                  <n-button v-if="selectedLocationId" size="small" tertiary type="error" :loading="isDeletingLocationBible" @click="handleDeleteLocationBible">{{ t('workbench.locationBible.deleteLocation') }}</n-button>
                  <n-button size="small" secondary :loading="isSavingLocationBible" @click="handleSaveLocationBible">{{ t('workbench.locationBible.saveLocation') }}</n-button>
                </div>
              </div>
            </div>
          </section>

          <section class="rounded-vt-md border border-border bg-card p-vt-5 shadow-vt-md">
            <h2 class="text-sm font-semibold text-primary">{{ t('workbench.cover.title') }}</h2>
            <p class="mt-vt-3 text-sm leading-6 text-secondary">{{ t('workbench.cover.desc') }}</p>
            <div class="mt-vt-4 grid gap-vt-3">
              <div class="rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-muted">
                <div class="font-medium text-secondary">{{ project?.coverPath || t('workbench.cover.noCover') }}</div>
                <div class="mt-vt-1">{{ project?.coverTitle || t('workbench.cover.noTitle') }}</div>
                <div class="mt-vt-1">{{ project?.coverTemplateId || t('workbench.cover.defaultTemplate') }}</div>
              </div>
              <label class="grid gap-vt-1 text-xs text-muted">
                <span>{{ t('workbench.cover.coverTitle') }}</span>
                <n-input v-model:value="coverForm.title" size="small" :maxlength="15" :placeholder="t('workbench.cover.titlePlaceholder')" />
              </label>
              <label class="grid gap-vt-1 text-xs text-muted">
                <span>{{ t('workbench.cover.sourceItem') }}</span>
                <n-select v-model:value="coverForm.sourceItemId" size="small" clearable :options="coverSourceItemOptions" :placeholder="t('workbench.cover.sourceItemPlaceholder')" />
              </label>
              <label class="grid gap-vt-1 text-xs text-muted">
                <span>{{ t('workbench.cover.uploadImage') }}</span>
                <input ref="coverFileInput" class="hidden" type="file" accept="image/*" @change="handleCoverFilePicked" />
                <n-input v-model:value="coverUploadSourcePath" size="small" :placeholder="t('workbench.cover.uploadPlaceholder')" />
              </label>
              <div class="grid gap-vt-2">
                <n-button size="small" @click="coverFileInput?.click()">{{ t('workbench.cover.chooseImage') }}</n-button>
                <n-button size="small" secondary :loading="isUploadingCover" :disabled="!coverUploadSourcePath.trim()" @click="handleUploadCoverImage">{{ t('workbench.cover.uploadAndGenerate') }}</n-button>
                <n-button size="small" secondary :loading="isGeneratingCover" @click="handleGenerateCover">{{ t('workbench.cover.generate') }}</n-button>
              </div>
            </div>
          </section>

          <section class="rounded-vt-md border border-border bg-card p-vt-5 shadow-vt-md">
            <h2 class="text-sm font-semibold text-primary">{{ t('workbench.exportTitle') }}</h2>
            <p class="mt-vt-3 text-sm leading-6 text-secondary">{{ t('workbench.exportDesc') }}</p>
            <button type="button" class="mt-vt-4 w-full rounded-vt-sm border border-border-strong px-vt-3 py-vt-2 text-sm text-secondary transition hover:bg-card-hover hover:text-primary">{{ t('workbench.exportPlaceholder') }}</button>
          </section>
        </aside>
      </div>
    </main>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { NButton, NInput, NSelect, NSwitch, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import { analyzeStyleReferenceImage, applyStylePreset, bindCharacterReferenceAsset, bindLocationReferenceAsset, bindStyleReferenceAsset, deleteProjectCharacterBible, deleteProjectLocationBible, getProjectStyleBible, importAsset, listProjectCharacterBibles, listProjectLocationBibles, listStylePresets, upsertProjectCharacterBible, upsertProjectLocationBible, upsertProjectStyleBible } from '@/entities/config/api'
import type { CharacterBibleDto, LocationBibleDto, StyleBibleDto, StylePresetDto, StyleReferenceAnalysisDto } from '@/entities/config/types'
import { useProjectStore } from '@/entities/project/store'
import type { ProjectDetailDto } from '@/entities/project/types'
import { useStoryboardStore } from '@/entities/storyboard/store'
import { useTaskStore } from '@/entities/task/store'
import { useDictOptions } from '@/shared/dict/useDictOptions'
import type { ProjectLifecycle, TaskStatus } from '@/shared/enums/generated'
import { getStatusToneClass } from '@/shared/theme'

type WorkspaceEntryKey = 'storyboard' | 'image' | 'video' | 'composition'

interface WorkspaceEntry {
  key: WorkspaceEntryKey
  step: string
  status: 'ready' | 'locked' | 'optional'
  path: string
}

const route = useRoute()
const router = useRouter()
const projectStore = useProjectStore()
const storyboardStore = useStoryboardStore()
const taskStore = useTaskStore()
const { t } = useI18n()
const message = useMessage()
const lifecycleOptions = useDictOptions('projectLifecycle')
const taskStatusOptions = useDictOptions('taskStatus')
const isCancellingTask = ref(false)
const isSavingStyleBible = ref(false)
const isSavingCharacterBible = ref(false)
const isSavingLocationBible = ref(false)
const isDeletingCharacterBible = ref(false)
const isDeletingLocationBible = ref(false)
const isImportingStyleReference = ref(false)
const isImportingCharacterReference = ref(false)
const isImportingLocationReference = ref(false)
const isAnalyzingStyleReference = ref(false)
const isGeneratingCover = ref(false)
const isUploadingCover = ref(false)
const styleBible = ref<StyleBibleDto | null>(null)
const characterBibles = ref<CharacterBibleDto[]>([])
const locationBibles = ref<LocationBibleDto[]>([])
const stylePresets = ref<StylePresetDto[]>([])
const styleReferenceAnalysis = ref<StyleReferenceAnalysisDto | null>(null)
const selectedStylePresetId = ref<string | null>(null)
const selectedCharacterId = ref<string | null>(null)
const selectedLocationId = ref<string | null>(null)
const styleReferenceFileInput = ref<HTMLInputElement | null>(null)
const characterReferenceFileInput = ref<HTMLInputElement | null>(null)
const locationReferenceFileInput = ref<HTMLInputElement | null>(null)
const coverFileInput = ref<HTMLInputElement | null>(null)
const styleReferenceSourcePath = ref('')
const characterReferenceSourcePath = ref('')
const locationReferenceSourcePath = ref('')
const coverUploadSourcePath = ref('')

const styleForm = reactive({
  name: '',
  stylePrompt: '',
  colorPaletteText: '',
  lighting: '',
  composition: '',
  negativePrompt: '',
  referenceImagePath: '',
  saveAsPreset: false,
})

const characterForm = reactive({
  characterId: '',
  name: '',
  aliasText: '',
  age: '',
  gender: '',
  appearance: '',
  clothing: '',
  personality: '',
  visualPrompt: '',
  negativePrompt: '',
  referenceImagePath: '',
})

const locationForm = reactive({
  locationId: '',
  name: '',
  spaceDescription: '',
  lighting: '',
  timeOfDay: '',
  propsText: '',
  visualPrompt: '',
  negativePrompt: '',
  referenceImagePath: '',
})

const coverForm = reactive({
  title: '',
  sourceItemId: null as string | null,
})

const projectId = computed(() => String(route.params.projectId))
const projectDetail = computed<ProjectDetailDto | null>(() => projectStore.currentProject)
const project = computed(() => projectDetail.value?.project)
const projectTitle = computed(() => project.value?.title || t('workbench.loadingTitle'))

const workspaceEntries = computed<WorkspaceEntry[]>(() => [
  { key: 'storyboard', step: '01', status: 'ready', path: `/projects/${projectId.value}/workspace/storyboard` },
  { key: 'image', step: '02', status: 'locked', path: `/projects/${projectId.value}/workspace/image` },
  { key: 'video', step: '03', status: 'locked', path: `/projects/${projectId.value}/workspace/video` },
  { key: 'composition', step: '04', status: 'locked', path: `/projects/${projectId.value}/workspace/compose` },
])

const contentSummary = computed(() => [
  { key: 'input', value: project.value ? t(`dict.inputType.${normalizeI18nKey(project.value.inputType)}`) : t('workbench.loadingValue') },
  { key: 'videoPack', value: project.value?.activePackId ?? t('workbench.noVideoPack') },
  { key: 'aspectRatio', value: project.value?.aspectRatio ?? t('workbench.loadingValue') },
  { key: 'sceneCount', value: project.value ? t('workbench.count', { count: project.value.targetSceneCount }) : t('workbench.loadingValue') },
  { key: 'duration', value: project.value ? t('workbench.seconds', { seconds: project.value.segmentDurationSeconds }) : t('workbench.loadingValue') },
  { key: 'ruleRefs', value: project.value ? t('workbench.count', { count: Object.keys(project.value.ruleRefs ?? {}).length }) : t('workbench.loadingValue') },
  { key: 'executableRefs', value: project.value ? t('workbench.count', { count: Object.keys(project.value.executableRefs ?? {}).length }) : t('workbench.loadingValue') },
])
const resourceUsageSummary = computed(() => {
  const items = storyboardStore.storyboard?.items ?? []
  const imageCount = items.reduce((total, item) => total + item.imageCandidates.length, 0)
  const videoCount = items.reduce((total, item) => total + item.videoSegments.length, 0)
  return [
    { key: 'images', value: t('workbench.resourceUsage.count', { count: imageCount }) },
    { key: 'videos', value: t('workbench.resourceUsage.count', { count: videoCount }) },
    { key: 'llm', value: t('workbench.resourceUsage.notTracked') },
  ]
})
const coverSourceItemOptions = computed(() =>
  (storyboardStore.storyboard?.items ?? [])
    .filter((item) => item.selectedImageId)
    .map((item) => ({
      label: `#${item.index.toString().padStart(2, '0')} ${item.narrationText || item.sourceText}`,
      value: item.itemId,
    }))
)
const stylePresetOptions = computed(() =>
  stylePresets.value.map((preset) => ({
    label: `${preset.name} · ${preset.sourceType}`,
    value: preset.presetId,
  }))
)

const lifecycleText = computed(() => labelFromOptions(lifecycleOptions.value, project.value?.lifecycle, t('workbench.loadingValue')))
const lifecycleClass = computed(() => getStatusToneClass(lifecycleOptions.value.find((option) => option.value === project.value?.lifecycle)?.colorToken))
const latestTaskStatus = computed(() => labelFromOptions(taskStatusOptions.value, project.value?.latestTask?.taskStatus, t('workbench.noTask')))
const latestTaskSummary = computed(() => project.value?.latestTask?.summary ?? t('workbench.noTaskSummary'))
const canCancelLatestTask = computed(() => project.value?.latestTask?.taskStatus === 'running' || project.value?.latestTask?.taskStatus === 'pending')

onMounted(async () => {
  await Promise.all([projectStore.loadProject(projectId.value), storyboardStore.loadStoryboard(projectId.value)])
  assignCoverForm()
  await loadStyleBibleContext()
  await loadCharacterBibleContext()
  await loadLocationBibleContext()
})

function assignCoverForm() {
  coverForm.title = project.value?.coverTitle || project.value?.title || ''
  coverForm.sourceItemId = project.value?.coverSourceItemId || null
}

async function handleCancelLatestTask() {
  if (!canCancelLatestTask.value) return
  isCancellingTask.value = true
  try {
    await taskStore.cancelTask({ projectId: projectId.value })
    await projectStore.loadProject(projectId.value)
    message.success(t('workbench.cancelTaskSuccess'))
  } catch (error) {
    message.error(error instanceof Error ? error.message : t('workbench.cancelTaskFailed'))
  } finally {
    isCancellingTask.value = false
  }
}

async function handleGenerateCover() {
  isGeneratingCover.value = true
  try {
    await projectStore.generateCover({
      projectId: projectId.value,
      coverTitle: coverForm.title,
      coverTemplateId: project.value?.coverTemplateId || 'knowledge_bold',
      coverSourceItemId: coverForm.sourceItemId || undefined,
    })
    assignCoverForm()
    message.success(t('workbench.cover.generateSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isGeneratingCover.value = false
  }
}

function handleCoverFilePicked(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  coverUploadSourcePath.value = readFilePath(file) || coverUploadSourcePath.value
  input.value = ''
}

async function handleUploadCoverImage() {
  const sourcePath = coverUploadSourcePath.value.trim()
  if (!sourcePath) return
  isUploadingCover.value = true
  try {
    await projectStore.replaceCoverImage({
      projectId: projectId.value,
      sourcePath,
      coverTitle: coverForm.title,
      coverTemplateId: project.value?.coverTemplateId || 'knowledge_bold',
    })
    coverUploadSourcePath.value = ''
    assignCoverForm()
    message.success(t('workbench.cover.uploadSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isUploadingCover.value = false
  }
}

async function loadStyleBibleContext() {
  try {
    const [style, presets] = await Promise.all([getProjectStyleBible(projectId.value), listStylePresets()])
    styleBible.value = style
    stylePresets.value = presets
    assignStyleForm(style)
    styleReferenceAnalysis.value = null
  } catch (error) {
    message.error(errorMessage(error))
  }
}

async function loadCharacterBibleContext() {
  try {
    const characters = await listProjectCharacterBibles(projectId.value)
    characterBibles.value = characters
    if (selectedCharacterId.value && !characters.some((item) => item.characterId === selectedCharacterId.value)) {
      selectedCharacterId.value = null
    }
    if (selectedCharacterId.value) {
      const current = characters.find((item) => item.characterId === selectedCharacterId.value)
      if (current) assignCharacterForm(current)
    } else if (characters[0]) {
      selectCharacterBible(characters[0].characterId)
    } else {
      assignEmptyCharacterForm()
    }
  } catch (error) {
    message.error(errorMessage(error))
  }
}

async function loadLocationBibleContext() {
  try {
    const locations = await listProjectLocationBibles(projectId.value)
    locationBibles.value = locations
    if (selectedLocationId.value && !locations.some((item) => item.locationId === selectedLocationId.value)) {
      selectedLocationId.value = null
    }
    if (selectedLocationId.value) {
      const current = locations.find((item) => item.locationId === selectedLocationId.value)
      if (current) assignLocationForm(current)
    } else if (locations[0]) {
      selectLocationBible(locations[0].locationId)
    } else {
      assignEmptyLocationForm()
    }
  } catch (error) {
    message.error(errorMessage(error))
  }
}

async function handleSaveStyleBible() {
  isSavingStyleBible.value = true
  const shouldRefreshPresets = styleForm.saveAsPreset
  try {
    const saved = await upsertProjectStyleBible({
      projectId: projectId.value,
      styleBibleId: styleBible.value?.styleBibleId,
      name: styleForm.name,
      stylePrompt: styleForm.stylePrompt,
      colorPalette: parseStyleList(styleForm.colorPaletteText),
      lighting: styleForm.lighting,
      composition: styleForm.composition,
      negativePrompt: styleForm.negativePrompt,
      referenceImagePath: styleForm.referenceImagePath || undefined,
      referenceImages: styleBible.value?.referenceImages,
      saveAsPreset: styleForm.saveAsPreset,
      presetName: styleForm.name,
    })
    styleBible.value = saved
    assignStyleForm(saved)
    await projectStore.loadProject(projectId.value)
    if (shouldRefreshPresets) stylePresets.value = await listStylePresets()
    message.success(t('workbench.styleBible.saveSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isSavingStyleBible.value = false
  }
}

async function handleApplyStylePreset(value: string | null) {
  if (!value) return
  try {
    const style = await applyStylePreset({ projectId: projectId.value, presetId: value })
    styleBible.value = style
    assignStyleForm(style)
    await projectStore.loadProject(projectId.value)
    message.success(t('workbench.styleBible.applySuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  }
}

function handleStyleReferenceFilePicked(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  styleReferenceSourcePath.value = readFilePath(file) || styleReferenceSourcePath.value
  input.value = ''
}

async function handleImportStyleReference() {
  const sourcePath = styleReferenceSourcePath.value.trim()
  if (!sourcePath) return
  isImportingStyleReference.value = true
  try {
    const fileName = fileNameFromPath(sourcePath)
    const asset = await importAsset({
      sourcePath,
      kind: 'style_reference_image',
      displayName: trimExtension(fileName) || 'style_reference',
      mimeType: guessImageMimeType(fileName),
      metadata: {
        displayName: trimExtension(fileName) || 'style_reference',
        mediaKind: 'image',
        sourceType: 'style_bible_reference',
      },
    })
    const bound = await bindStyleReferenceAsset({
      projectId: projectId.value,
      styleBibleId: styleBible.value?.styleBibleId,
      assetId: asset.assetId,
    })
    styleBible.value = bound.styleBible
    assignStyleForm(bound.styleBible)
    styleReferenceAnalysis.value = null
    await projectStore.loadProject(projectId.value)
    styleReferenceSourcePath.value = ''
    message.success(t('workbench.styleBible.referenceImportSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isImportingStyleReference.value = false
  }
}

async function handleAnalyzeStyleReference() {
  if (!styleForm.referenceImagePath) return
  isAnalyzingStyleReference.value = true
  try {
    styleReferenceAnalysis.value = await analyzeStyleReferenceImage({
      projectId: projectId.value,
      styleBibleId: styleBible.value?.styleBibleId,
    })
    message.success(t('workbench.styleBible.analysisSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isAnalyzingStyleReference.value = false
  }
}

function handleApplyStyleAnalysis() {
  const analysis = styleReferenceAnalysis.value
  if (!analysis) return
  styleForm.stylePrompt = analysis.stylePrompt
  styleForm.colorPaletteText = analysis.colorPalette.join('、')
  styleForm.lighting = analysis.lighting
  styleForm.composition = analysis.composition
  styleForm.negativePrompt = analysis.negativePromptSuggestion
  message.success(t('workbench.styleBible.analysisApplied'))
}

function handleNewCharacterBible() {
  selectedCharacterId.value = null
  assignEmptyCharacterForm()
}

function selectCharacterBible(characterId: string) {
  const character = characterBibles.value.find((item) => item.characterId === characterId)
  if (!character) return
  selectedCharacterId.value = character.characterId
  assignCharacterForm(character)
}

async function handleSaveCharacterBible() {
  isSavingCharacterBible.value = true
  try {
    const saved = await upsertProjectCharacterBible({
      projectId: projectId.value,
      characterId: selectedCharacterId.value || characterForm.characterId.trim() || undefined,
      name: characterForm.name,
      alias: parseStyleList(characterForm.aliasText),
      age: characterForm.age,
      gender: characterForm.gender,
      appearance: characterForm.appearance,
      clothing: characterForm.clothing,
      personality: characterForm.personality,
      visualPrompt: characterForm.visualPrompt,
      negativePrompt: characterForm.negativePrompt,
      referenceImagePath: characterForm.referenceImagePath || undefined,
      referenceImages: selectedCharacterId.value ? characterBibles.value.find((item) => item.characterId === selectedCharacterId.value)?.referenceImages : undefined,
    })
    selectedCharacterId.value = saved.characterId
    await loadCharacterBibleContext()
    await projectStore.loadProject(projectId.value)
    message.success(t('workbench.characterBible.saveSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isSavingCharacterBible.value = false
  }
}

async function handleDeleteCharacterBible() {
  if (!selectedCharacterId.value) return
  isDeletingCharacterBible.value = true
  try {
    await deleteProjectCharacterBible({ characterId: selectedCharacterId.value })
    selectedCharacterId.value = null
    await loadCharacterBibleContext()
    await projectStore.loadProject(projectId.value)
    message.success(t('workbench.characterBible.deleteSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isDeletingCharacterBible.value = false
  }
}

function handleCharacterReferenceFilePicked(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  characterReferenceSourcePath.value = readFilePath(file) || characterReferenceSourcePath.value
  input.value = ''
}

async function handleImportCharacterReference() {
  if (!selectedCharacterId.value) return
  const sourcePath = characterReferenceSourcePath.value.trim()
  if (!sourcePath) return
  isImportingCharacterReference.value = true
  try {
    const fileName = fileNameFromPath(sourcePath)
    const asset = await importAsset({
      sourcePath,
      kind: 'character_reference_image',
      displayName: trimExtension(fileName) || selectedCharacterId.value,
      mimeType: guessImageMimeType(fileName),
      metadata: {
        displayName: trimExtension(fileName) || selectedCharacterId.value,
        mediaKind: 'image',
        sourceType: 'character_bible_reference',
      },
    })
    const bound = await bindCharacterReferenceAsset({
      projectId: projectId.value,
      characterId: selectedCharacterId.value,
      assetId: asset.assetId,
      referenceRole: 'character_front_view',
    })
    selectedCharacterId.value = bound.characterBible.characterId
    await loadCharacterBibleContext()
    characterReferenceSourcePath.value = ''
    message.success(t('workbench.characterBible.referenceImportSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isImportingCharacterReference.value = false
  }
}

function handleNewLocationBible() {
  selectedLocationId.value = null
  assignEmptyLocationForm()
}

function selectLocationBible(locationId: string) {
  const location = locationBibles.value.find((item) => item.locationId === locationId)
  if (!location) return
  selectedLocationId.value = location.locationId
  assignLocationForm(location)
}

async function handleSaveLocationBible() {
  isSavingLocationBible.value = true
  try {
    const saved = await upsertProjectLocationBible({
      projectId: projectId.value,
      locationId: selectedLocationId.value || locationForm.locationId.trim() || undefined,
      name: locationForm.name,
      spaceDescription: locationForm.spaceDescription,
      lighting: locationForm.lighting,
      timeOfDay: locationForm.timeOfDay,
      props: parseStyleList(locationForm.propsText),
      visualPrompt: locationForm.visualPrompt,
      negativePrompt: locationForm.negativePrompt,
      referenceImagePath: locationForm.referenceImagePath || undefined,
      referenceImages: selectedLocationId.value ? locationBibles.value.find((item) => item.locationId === selectedLocationId.value)?.referenceImages : undefined,
      variants: selectedLocationId.value ? locationBibles.value.find((item) => item.locationId === selectedLocationId.value)?.variants : undefined,
    })
    selectedLocationId.value = saved.locationId
    await loadLocationBibleContext()
    await projectStore.loadProject(projectId.value)
    message.success(t('workbench.locationBible.saveSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isSavingLocationBible.value = false
  }
}

async function handleDeleteLocationBible() {
  if (!selectedLocationId.value) return
  isDeletingLocationBible.value = true
  try {
    await deleteProjectLocationBible({ locationId: selectedLocationId.value })
    selectedLocationId.value = null
    await loadLocationBibleContext()
    await projectStore.loadProject(projectId.value)
    message.success(t('workbench.locationBible.deleteSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isDeletingLocationBible.value = false
  }
}

function handleLocationReferenceFilePicked(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  locationReferenceSourcePath.value = readFilePath(file) || locationReferenceSourcePath.value
  input.value = ''
}

async function handleImportLocationReference() {
  if (!selectedLocationId.value) return
  const sourcePath = locationReferenceSourcePath.value.trim()
  if (!sourcePath) return
  isImportingLocationReference.value = true
  try {
    const fileName = fileNameFromPath(sourcePath)
    const asset = await importAsset({
      sourcePath,
      kind: 'scene_reference_image',
      displayName: trimExtension(fileName) || selectedLocationId.value,
      mimeType: guessImageMimeType(fileName),
      metadata: {
        displayName: trimExtension(fileName) || selectedLocationId.value,
        mediaKind: 'image',
        sourceType: 'location_bible_reference',
      },
    })
    const bound = await bindLocationReferenceAsset({
      projectId: projectId.value,
      locationId: selectedLocationId.value,
      assetId: asset.assetId,
      referenceRole: 'scene_wide_view',
    })
    selectedLocationId.value = bound.locationBible.locationId
    await loadLocationBibleContext()
    locationReferenceSourcePath.value = ''
    message.success(t('workbench.locationBible.referenceImportSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isImportingLocationReference.value = false
  }
}

function assignStyleForm(style: StyleBibleDto) {
  styleForm.name = style.name
  styleForm.stylePrompt = style.stylePrompt
  styleForm.colorPaletteText = style.colorPalette.join('、')
  styleForm.lighting = style.lighting
  styleForm.composition = style.composition
  styleForm.negativePrompt = style.negativePrompt
  styleForm.referenceImagePath = style.referenceImagePath ?? ''
  styleForm.saveAsPreset = false
}

function assignCharacterForm(character: CharacterBibleDto) {
  characterForm.characterId = character.characterId
  characterForm.name = character.name
  characterForm.aliasText = character.alias.join('、')
  characterForm.age = character.age
  characterForm.gender = character.gender
  characterForm.appearance = character.appearance
  characterForm.clothing = character.clothing
  characterForm.personality = character.personality
  characterForm.visualPrompt = character.visualPrompt
  characterForm.negativePrompt = character.negativePrompt
  characterForm.referenceImagePath = character.referenceImagePath ?? ''
}

function assignEmptyCharacterForm() {
  characterForm.characterId = ''
  characterForm.name = ''
  characterForm.aliasText = ''
  characterForm.age = ''
  characterForm.gender = ''
  characterForm.appearance = ''
  characterForm.clothing = ''
  characterForm.personality = ''
  characterForm.visualPrompt = ''
  characterForm.negativePrompt = ''
  characterForm.referenceImagePath = ''
  characterReferenceSourcePath.value = ''
}

function assignLocationForm(location: LocationBibleDto) {
  locationForm.locationId = location.locationId
  locationForm.name = location.name
  locationForm.spaceDescription = location.spaceDescription
  locationForm.lighting = location.lighting
  locationForm.timeOfDay = location.timeOfDay
  locationForm.propsText = location.props.join('、')
  locationForm.visualPrompt = location.visualPrompt
  locationForm.negativePrompt = location.negativePrompt
  locationForm.referenceImagePath = location.referenceImagePath ?? ''
}

function assignEmptyLocationForm() {
  locationForm.locationId = ''
  locationForm.name = ''
  locationForm.spaceDescription = ''
  locationForm.lighting = ''
  locationForm.timeOfDay = ''
  locationForm.propsText = ''
  locationForm.visualPrompt = ''
  locationForm.negativePrompt = ''
  locationForm.referenceImagePath = ''
  locationReferenceSourcePath.value = ''
}

function styleAnalysisWarningLabel(warning: string) {
  const key = `workbench.styleBible.warningLabels.${normalizeI18nKey(warning)}`
  const translated = t(key)
  return translated === key ? warning : translated
}

function parseStyleList(value: string) {
  return value.split(/[、,，\n]/).map((item) => item.trim()).filter(Boolean)
}

function readFilePath(file: File) {
  const record = file as File & { path?: string }
  return typeof record.path === 'string' ? record.path : ''
}

function fileNameFromPath(path: string) {
  return path.split(/[\\/]/).filter(Boolean).at(-1) ?? ''
}

function trimExtension(fileName: string) {
  const index = fileName.lastIndexOf('.')
  return index > 0 ? fileName.slice(0, index) : fileName
}

function guessImageMimeType(fileName: string) {
  const extension = fileName.split('.').at(-1)?.toLowerCase()
  if (extension === 'jpg' || extension === 'jpeg') return 'image/jpeg'
  if (extension === 'webp') return 'image/webp'
  if (extension === 'gif') return 'image/gif'
  if (extension === 'bmp') return 'image/bmp'
  return 'image/png'
}

function normalizeI18nKey(value: string) {
  return value.replace(/[-.]/g, '_')
}

function labelFromOptions<T extends string>(options: Array<{ value: T; label: string }>, value: T | undefined, fallback: string) {
  return options.find((option) => option.value === value)?.label ?? value ?? fallback
}

function errorMessage(error: unknown) {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  return String(error)
}
</script>
