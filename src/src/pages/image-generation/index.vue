<template>
  <section class="view h-full min-w-0 overflow-hidden bg-page text-primary">
    <div class="flex h-full min-h-0 flex-col overflow-hidden">
      <WorkspaceHeader :project-id="projectId" :project-title="projectTitle" current-step="image" :access="workspaceAccess" :back-to="`/projects/${projectId}/workspace/storyboard`" :badge-label="isMockImageFlow ? t('imageGeneration.mockBadge') : ''" right-width-class="w-[500px]" :usage="resourceUsage" @blocked="handleBlockedStep">
        <template #actions>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="dirtyItemIds.size === 0 || isSavingAll" @click="handleSaveAll">{{ t('imageGeneration.saveAll') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="selectedImageKind !== 'storyboard_image' || isBulkGenerating" @click="handleGenerateMissing">{{ t('imageGeneration.generateMissing') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm border border-border-strong px-vt-3 text-sm font-medium text-secondary transition hover:bg-card hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="selectedImageKind !== 'storyboard_image' || isBulkGenerating" @click="handleGenerateAll">{{ t('imageGeneration.generateAll') }}</button>
          <button type="button" class="inline-flex h-9 items-center justify-center rounded-vt-sm bg-accent px-vt-4 text-sm font-semibold text-accent-ink transition hover:brightness-110" @click="handleEnterVideo">{{ t('imageGeneration.enterVideo') }}</button>
        </template>
      </WorkspaceHeader>

      <main class="flex min-h-0 flex-1 flex-col overflow-hidden bg-page">
        <section class="flex min-h-0 flex-1 flex-col gap-vt-3 overflow-hidden p-vt-3">
          <div class="flex flex-none flex-wrap items-center gap-vt-2 rounded-vt-md border border-border bg-card px-vt-3 py-vt-2 text-xs">
            <span class="font-medium text-secondary">{{ t('imageGeneration.toolbarTitle') }}</span>
            <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-muted">{{ t('imageGeneration.rowCount', { count: storyboardItems.length }) }}</span>
            <span v-if="dirtyItemIds.size > 0" class="rounded-vt-sm border border-accent-line bg-accent-soft px-vt-2 py-1 text-accent">{{ t('imageGeneration.dirtyCount', { count: dirtyItemIds.size }) }}</span>
            <span v-if="imageResetCount > 0" class="rounded-vt-sm border border-status-retrying/50 bg-status-retrying/10 px-vt-2 py-1 text-status-retrying">{{ t('imageGeneration.downstreamReset.count', { count: imageResetCount }) }}</span>
            <span v-if="bulkImageLockedCount > 0" class="rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-muted">{{ t('imageGeneration.lockedSkipCount', { count: bulkImageLockedCount }) }}</span>
            <WorkspaceRowJump class="ml-auto" :count="storyboardItems.length" @jump="jumpToRow" />
            <span class="text-muted">{{ t('imageGeneration.boundaryHint') }}</span>
          </div>

          <div class="grid flex-none grid-cols-[220px_260px_220px_1fr] gap-vt-2 rounded-vt-md border border-border bg-card px-vt-3 py-vt-2 text-xs">
            <div class="grid gap-vt-1">
              <span class="font-medium text-secondary">{{ t('imageGeneration.imageKind.label') }}</span>
              <n-select v-model:value="selectedImageKind" size="small" :options="imageKindOptions" />
            </div>
            <div class="grid gap-vt-1">
              <span class="font-medium text-secondary">{{ t('imageGeneration.imageKind.owner') }}</span>
              <n-select v-model:value="selectedAssetOwnerId" size="small" :options="assetOwnerOptions" :disabled="selectedImageKind === 'storyboard_image' || assetOwnerOptions.length === 0" :placeholder="t('imageGeneration.imageKind.ownerPlaceholder')" />
            </div>
            <div class="grid gap-vt-1">
              <span class="font-medium text-secondary">{{ t('imageGeneration.imageKind.referenceRole') }}</span>
              <n-select v-model:value="selectedReferenceRole" size="small" :options="referenceRoleOptions" :disabled="selectedImageKind === 'storyboard_image' || referenceRoleOptions.length === 0" :placeholder="t('imageGeneration.imageKind.referenceRolePlaceholder')" />
            </div>
            <div class="grid gap-vt-1">
              <span class="font-medium text-secondary">{{ selectedImageKindLabel }}</span>
              <span class="text-muted">{{ selectedImageKindHint }}</span>
            </div>
          </div>

          <div class="min-h-0 flex-1 overflow-auto rounded-vt-md border border-border bg-card">
            <table class="w-full min-w-[2330px] table-fixed border-separate border-spacing-0 text-left text-sm">
              <thead class="sticky top-0 z-10 bg-panel text-xs text-muted">
                <tr>
                  <th class="w-[56px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.index') }}</th>
                  <th class="w-[240px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.source') }}</th>
                  <th class="w-[190px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.intent') }}</th>
                  <th class="w-[170px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.characters') }}</th>
                  <th class="w-[190px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.characterResources') }}</th>
                  <th class="w-[180px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.scene') }}</th>
                  <th class="w-[260px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.visual') }}</th>
                  <th class="w-[300px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.prompt') }}</th>
                  <th class="w-[220px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.negativePrompt') }}</th>
                  <th class="w-[260px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.candidates') }}</th>
                  <th class="w-[160px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.selected') }}</th>
                  <th class="w-[120px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.status') }}</th>
                  <th class="w-[170px] border-b border-border px-vt-2 py-vt-2 font-medium">{{ t('imageGeneration.columns.actions') }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="item in storyboardItems" :id="rowDomId(item.index)" :key="item.itemId" class="h-[132px] align-top transition hover:bg-card-hover/70" :class="dirtyItemIds.has(item.itemId) ? 'bg-accent-soft/40' : ''">
                  <td class="border-b border-border px-vt-2 py-vt-2 align-middle text-center">
                    <div class="mx-auto grid size-8 place-items-center rounded-vt-sm border border-border bg-page text-xs text-muted">#{{ item.index.toString().padStart(2, '0') }}</div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="h-[116px] overflow-y-auto rounded-vt-sm border border-border bg-page px-vt-2 py-vt-2 text-xs leading-5 text-secondary">
                      <div class="font-medium text-primary">{{ item.sourceText || t('imageGeneration.emptySource') }}</div>
                      <div v-if="item.narrationText && item.narrationText !== item.sourceText" class="mt-vt-2 border-t border-border pt-vt-2 text-muted">{{ item.narrationText }}</div>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="h-[116px] overflow-y-auto rounded-vt-sm border border-border bg-page px-vt-2 py-vt-2 text-xs leading-5 text-secondary">{{ item.visualGoal || t('imageGeneration.emptyIntent') }}</div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <button type="button" class="mb-vt-1 justify-self-end rounded-vt-sm border border-border bg-page px-vt-2 py-0.5 text-[11px] text-muted hover:border-border-strong hover:text-primary" @click="toggleItemLock(item, 'characters')">{{ lockButtonLabel(item, 'characters') }}</button>
                    <n-select :value="item.characterIds" multiple size="small" clearable :disabled="isItemLocked(item, 'characters')" :options="storyboardCharacterOptions" :placeholder="t('imageGeneration.placeholders.characterIds')" @update:value="updateCharacterIds(item, $event)" />
                    <div class="mt-vt-2 truncate text-[11px] text-muted">{{ formatCharacters(item) }}</div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid gap-vt-1 text-xs">
                      <div class="flex flex-wrap gap-vt-1">
                        <span class="rounded-vt-sm border border-status-failed/40 bg-status-failed/10 px-vt-2 py-0.5 text-status-failed">{{ t('imageGeneration.characterResources.required', { count: characterResourceMissingCount(item) }) }}</span>
                        <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-0.5 text-muted">{{ t('imageGeneration.characterResources.optional', { count: characterResourceOptionalCount(item) }) }}</span>
                      </div>
                      <div class="line-clamp-3 text-[11px] leading-4 text-muted">{{ characterResourceSummary(item) }}</div>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid gap-vt-2">
                      <button type="button" class="justify-self-end rounded-vt-sm border border-border bg-page px-vt-2 py-0.5 text-[11px] text-muted hover:border-border-strong hover:text-primary" @click="toggleItemLock(item, 'location')">{{ lockButtonLabel(item, 'location') }}</button>
                      <n-select :value="item.locationId" size="small" clearable :disabled="isItemLocked(item, 'location')" :options="storyboardLocationOptions" :placeholder="t('imageGeneration.placeholders.locationId')" @update:value="updateLocationId(item, $event)" />
                      <n-input v-if="!item.locationId" :value="item.sceneDescription" class="inp compact-inp" size="small" :disabled="isItemLocked(item, 'location')" :placeholder="t('imageGeneration.placeholders.scene')" @update:value="updateItem(item, { sceneDescription: $event }, 'location')" />
                      <div v-else class="truncate text-[11px] text-muted">{{ formatLocation(item) }}</div>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid gap-vt-1">
                      <button type="button" class="justify-self-end rounded-vt-sm border border-border bg-page px-vt-2 py-0.5 text-[11px] text-muted hover:border-border-strong hover:text-primary" @click="toggleItemLock(item, 'visualDescription')">{{ lockButtonLabel(item, 'visualDescription') }}</button>
                      <n-input :value="item.visualDescription" class="inp compact-inp storyboard-cell-textarea" size="small" type="textarea" :disabled="isItemLocked(item, 'visualDescription')" :placeholder="t('imageGeneration.placeholders.visual')" @update:value="updateItem(item, { visualDescription: $event }, 'visualDescription')" />
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid gap-vt-1">
                      <button type="button" class="justify-self-end rounded-vt-sm border border-border bg-page px-vt-2 py-0.5 text-[11px] text-muted hover:border-border-strong hover:text-primary" @click="toggleItemLock(item, 'imagePrompt')">{{ lockButtonLabel(item, 'imagePrompt') }}</button>
                      <n-input :value="item.imagePrompt" class="inp compact-inp storyboard-cell-textarea" size="small" type="textarea" :disabled="isItemLocked(item, 'imagePrompt')" :placeholder="t('imageGeneration.placeholders.prompt')" @update:value="updateItem(item, { imagePrompt: $event }, 'imagePrompt')" />
                      <n-button size="tiny" secondary :loading="promptPreviewLoadingItemId === item.itemId" @click="handleOpenPromptPreview(item)">{{ t('imageGeneration.previewPrompt') }}</n-button>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid gap-vt-1">
                      <button type="button" class="justify-self-end rounded-vt-sm border border-border bg-page px-vt-2 py-0.5 text-[11px] text-muted hover:border-border-strong hover:text-primary" @click="toggleItemLock(item, 'negativePrompt')">{{ lockButtonLabel(item, 'negativePrompt') }}</button>
                      <n-input :value="item.negativePrompt" class="inp compact-inp storyboard-cell-textarea" size="small" type="textarea" :disabled="isItemLocked(item, 'negativePrompt')" :placeholder="t('imageGeneration.placeholders.negativePrompt')" @update:value="updateItem(item, { negativePrompt: $event }, 'negativePrompt')" />
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div v-if="item.imageCandidates.length > 0" class="grid h-[116px] grid-cols-2 gap-vt-1 overflow-y-auto">
                      <div v-for="(candidate, candidateIndex) in sortedImageCandidates(item)" :key="candidate.imageId" class="grid min-h-[54px] grid-cols-[40px_1fr] items-center gap-vt-2 rounded-vt-sm border px-vt-2 py-vt-1 text-left text-[11px] leading-4 transition" :class="candidateButtonClass(item, candidate)">
                        <span class="grid h-9 w-10 place-items-center rounded-vt-sm border border-border text-[9px] font-semibold uppercase text-primary" :class="candidatePreviewClass(candidate, candidateIndex)">{{ t('imageGeneration.mockShort') }}</span>
                        <span class="min-w-0">
                          <span class="block truncate font-semibold">{{ t('imageGeneration.candidateVariant', { index: candidateVariantIndex(candidate, candidateIndex) }) }}</span>
                          <span class="block truncate text-muted">{{ t('imageGeneration.revisionLabel', { revision: candidateRevision(candidate) }) }} · {{ candidateFreshnessLabel(item, candidate) }}</span>
                          <span class="block truncate text-muted">{{ shortId(candidate.imageId) }}</span>
                          <span v-if="isCandidateSelected(item, candidate)" class="block text-accent">{{ t('imageGeneration.selectedMark') }}</span>
                          <span class="mt-1 flex gap-vt-1">
                            <button type="button" class="rounded-vt-sm border border-border bg-card px-vt-1.5 py-0.5 text-[10px] text-secondary hover:border-border-strong hover:text-primary disabled:cursor-not-allowed disabled:opacity-50" :disabled="isItemLocked(item, 'selectedImage')" @click.stop="handleSelectCandidate(item, candidate)">{{ t('imageGeneration.selectCandidate') }}</button>
                            <button type="button" class="rounded-vt-sm border border-border bg-card px-vt-1.5 py-0.5 text-[10px] text-secondary hover:border-border-strong hover:text-primary" @click.stop="handleOpenSnapshot(candidate)">{{ t('imageGeneration.snapshot.view') }}</button>
                          </span>
                        </span>
                      </div>
                    </div>
                    <div v-else class="grid h-[116px] place-items-center rounded-vt-sm border border-border bg-page px-vt-2 text-center text-xs text-muted">{{ t('imageGeneration.noCandidates') }}</div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="h-[116px] overflow-hidden rounded-vt-sm border border-border bg-page px-vt-2 py-vt-2 text-xs leading-5 text-secondary">
                      <template v-if="selectedCandidate(item)">
                        <button type="button" class="mb-vt-1 rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-[11px] text-muted hover:border-border-strong hover:text-primary" @click="toggleItemLock(item, 'selectedImage')">{{ lockButtonLabel(item, 'selectedImage') }}</button>
                        <div class="font-semibold text-primary">{{ shortId(selectedCandidate(item)?.imageId) }}</div>
                        <div class="mt-vt-1 line-clamp-4 text-muted">{{ selectedCandidate(item)?.imagePath }}</div>
                      </template>
                      <template v-else>
                        <button type="button" class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-[11px] text-muted hover:border-border-strong hover:text-primary" @click="toggleItemLock(item, 'selectedImage')">{{ lockButtonLabel(item, 'selectedImage') }}</button>
                        <div class="grid h-full place-items-center text-center text-muted">{{ t('imageGeneration.notSelected') }}</div>
                      </template>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid gap-vt-1 text-xs">
                      <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-vt-1 text-secondary">{{ t(`dict.sceneAssetStatus.${item.imageStatus}`) }}</span>
                      <span class="text-muted">{{ t('imageGeneration.candidateCount', { count: item.imageCandidates.length }) }}</span>
                      <span v-if="item.imageRetryCount > 0" class="text-muted">{{ t('imageGeneration.retryCount', { count: item.imageRetryCount }) }}</span>
                      <span v-if="item.imageLastErrorJson" class="line-clamp-2 rounded-vt-sm border border-status-failed/50 bg-status-failed/10 px-vt-2 py-vt-1 text-status-failed">{{ t('imageGeneration.lastError', { reason: imageErrorReason(item) }) }}</span>
                      <span v-if="imageResetRecord(item)" class="rounded-vt-sm border border-status-retrying/50 bg-status-retrying/10 px-vt-2 py-vt-1 text-status-retrying">{{ t('imageGeneration.downstreamReset.row', { reason: resetReasonLabel(imageResetRecord(item)?.triggerField) }) }}</span>
                    </div>
                  </td>
                  <td class="border-b border-border px-vt-2 py-vt-2">
                    <div class="grid gap-vt-1">
                      <n-button class="btn btn-ghost btn-block compact-action" size="small" :loading="generatingItemIds.has(item.itemId)" :disabled="selectedImageKind !== 'storyboard_image' && !canGenerateSelectedAssetKind(item)" @click="handleGenerateItem(item)">{{ generateButtonLabel(item) }}</n-button>
                      <n-button class="btn btn-ghost btn-block compact-action" size="small" :loading="savingItemId === item.itemId" :disabled="!dirtyItemIds.has(item.itemId)" @click="handleSaveItem(item)">{{ t('imageGeneration.saveRow') }}</n-button>
                      <n-popconfirm :positive-text="t('common.confirm')" :negative-text="t('common.cancel')" @positive-click="handleClearHistoricalCandidates(item)">
                        <template #trigger>
                          <n-button class="btn btn-ghost btn-block compact-action" size="small" :disabled="!hasClearableImageCandidates(item)">{{ t('imageGeneration.clearHistorical') }}</n-button>
                        </template>
                        {{ t('imageGeneration.clearHistoricalConfirm') }}
                      </n-popconfirm>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>

            <div v-if="storyboardItems.length === 0" class="grid h-full min-h-80 place-items-center text-sm text-muted">{{ t('imageGeneration.empty') }}</div>
          </div>
        </section>
      </main>
    </div>
    <n-modal v-model:show="isPromptPreviewOpen" preset="card" :title="t('imageGeneration.promptPreview.title')" class="max-w-4xl">
      <div v-if="promptPreview" class="grid gap-vt-4 text-sm">
        <section class="grid gap-vt-2">
          <div class="text-xs font-semibold text-muted">{{ t('imageGeneration.promptPreview.finalPrompt') }}</div>
          <pre class="max-h-56 overflow-auto whitespace-pre-wrap rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-secondary">{{ promptPreview.finalPrompt }}</pre>
        </section>
        <section class="grid gap-vt-2">
          <div class="flex items-center gap-vt-2 text-xs font-semibold text-muted">
            <span>{{ t('imageGeneration.promptPreview.finalNegativePrompt') }}</span>
            <span v-if="promptPreview.negativePromptTruncated" class="rounded-vt-sm border border-status-retrying/50 bg-status-retrying/10 px-vt-2 py-0.5 text-status-retrying">{{ t('imageGeneration.promptPreview.truncated', { max: promptPreview.negativePromptMaxLength }) }}</span>
          </div>
          <pre class="max-h-36 overflow-auto whitespace-pre-wrap rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-secondary">{{ promptPreview.finalNegativePrompt || '-' }}</pre>
        </section>
        <section class="grid gap-vt-2">
          <div class="text-xs font-semibold text-muted">{{ t('imageGeneration.promptPreview.sections') }}</div>
          <div class="grid gap-vt-2">
            <div v-for="section in promptPreview.sections" :key="section.key" class="rounded-vt-sm border border-border bg-page p-vt-3">
              <div class="text-xs font-semibold text-primary">{{ promptPreviewSectionLabel(section) }}</div>
              <div class="mt-vt-1 whitespace-pre-wrap text-xs leading-5 text-secondary">{{ section.content }}</div>
            </div>
          </div>
        </section>
        <section class="grid gap-vt-2">
          <div class="text-xs font-semibold text-muted">{{ t('imageGeneration.promptPreview.references') }}</div>
          <div v-if="promptPreview.referenceImages.length > 0" class="grid gap-vt-2">
            <div v-for="reference in promptPreview.referenceImages" :key="`${reference.role}:${reference.path}`" class="rounded-vt-sm border border-border bg-page p-vt-3 text-xs">
              <div class="font-semibold text-primary">{{ reference.role }}</div>
              <div class="mt-vt-1 break-all text-muted">{{ reference.path }}</div>
            </div>
          </div>
          <div v-else class="rounded-vt-sm border border-border bg-page p-vt-3 text-xs text-muted">{{ t('imageGeneration.promptPreview.noReferences') }}</div>
        </section>
      </div>
    </n-modal>
    <n-modal v-model:show="isSnapshotOpen" preset="card" :title="t('imageGeneration.snapshot.title')" class="max-w-4xl">
      <div v-if="snapshotCandidate" class="grid gap-vt-4 text-sm">
        <section class="grid grid-cols-2 gap-vt-2 text-xs md:grid-cols-4">
          <div class="rounded-vt-sm border border-border bg-page p-vt-3">
            <div class="text-muted">{{ t('imageGeneration.snapshot.imageId') }}</div>
            <div class="mt-vt-1 truncate font-semibold text-primary">{{ shortId(snapshotCandidate.imageId) }}</div>
          </div>
          <div class="rounded-vt-sm border border-border bg-page p-vt-3">
            <div class="text-muted">{{ t('imageGeneration.snapshot.revision') }}</div>
            <div class="mt-vt-1 font-semibold text-primary">{{ candidateRevision(snapshotCandidate) }}</div>
          </div>
          <div class="rounded-vt-sm border border-border bg-page p-vt-3">
            <div class="text-muted">{{ t('imageGeneration.snapshot.model') }}</div>
            <div class="mt-vt-1 truncate font-semibold text-primary">{{ snapshotText(snapshotCandidate.generationContextSnapshot.modelSnapshot, 'label') || snapshotCandidate.model }}</div>
          </div>
          <div class="rounded-vt-sm border border-border bg-page p-vt-3">
            <div class="text-muted">{{ t('imageGeneration.snapshot.referenceCount') }}</div>
            <div class="mt-vt-1 font-semibold text-primary">{{ snapshotArray(snapshotCandidate.generationContextSnapshot.promptSnapshot, 'referenceImages').length }}</div>
          </div>
        </section>
        <section class="grid gap-vt-2">
          <div class="text-xs font-semibold text-muted">{{ t('imageGeneration.snapshot.promptSections') }}</div>
          <div class="grid gap-vt-2">
            <div v-for="section in snapshotArray(snapshotCandidate.generationContextSnapshot.promptSnapshot, 'sections')" :key="String(section.key)" class="rounded-vt-sm border border-border bg-page p-vt-3 text-xs">
              <div class="font-semibold text-primary">{{ String(section.label ?? section.key ?? '-') }}</div>
              <div class="mt-vt-1 whitespace-pre-wrap leading-5 text-secondary">{{ String(section.content ?? '-') }}</div>
            </div>
          </div>
        </section>
        <section class="grid gap-vt-2">
          <div class="text-xs font-semibold text-muted">{{ t('imageGeneration.snapshot.bibles') }}</div>
          <div class="grid gap-vt-2 md:grid-cols-3">
            <pre class="max-h-44 overflow-auto whitespace-pre-wrap rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-secondary">{{ formatSnapshotJson(snapshotCandidate.generationContextSnapshot.styleBible) }}</pre>
            <pre class="max-h-44 overflow-auto whitespace-pre-wrap rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-secondary">{{ formatSnapshotJson(snapshotCandidate.generationContextSnapshot.characterBibles) }}</pre>
            <pre class="max-h-44 overflow-auto whitespace-pre-wrap rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-secondary">{{ formatSnapshotJson(snapshotCandidate.generationContextSnapshot.locationBible) }}</pre>
          </div>
        </section>
        <section class="grid gap-vt-2">
          <div class="text-xs font-semibold text-muted">{{ t('imageGeneration.snapshot.rawJson') }}</div>
          <pre class="max-h-80 overflow-auto whitespace-pre-wrap rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-secondary">{{ formatSnapshotJson(snapshotCandidate.generationContextSnapshot) }}</pre>
        </section>
      </div>
    </n-modal>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { NButton, NInput, NModal, NPopconfirm, NSelect, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import { useProjectStore } from '@/entities/project/store'
import { buildImagePromptPreview, listProjectCharacterBibles, listProjectLocationBibles } from '@/entities/config/api'
import type { CharacterBibleDto, ImagePromptPreviewDto, LocationBibleDto, PromptSectionDto } from '@/entities/config/types'
import { buildCharacterResourcePlan } from '@/entities/scene/api'
import { useStoryboardStore } from '@/entities/storyboard/store'
import type { CharacterResourcePlanDto, GeneratedImageKind, ImageCandidateDto, StartImageAssetGenerationRequest, StoryboardItemDto, StoryboardItemLockField } from '@/entities/storyboard/types'
import { validateStoryboardItemsForVideoGeneration, type StoryboardVideoEntryField } from '@/entities/storyboard/validation'
import { getLatestStoryboardResetRecord, isResetRelevantToImage, isStoryboardItemLocked, isStoryboardItemLockedForBulkImageGeneration, lockedFieldsForImageGeneration, setStoryboardItemLock } from '@/entities/storyboard/reset'
import { getRequiredWorkspaceStep, getWorkspaceStepAccess, getWorkspaceStepPath, type WorkspaceStepKey } from '@/features/workspace/steps'
import WorkspaceHeader from '@/features/workspace/WorkspaceHeader.vue'
import WorkspaceRowJump from '@/features/workspace/WorkspaceRowJump.vue'

const route = useRoute()
const router = useRouter()
const message = useMessage()
const { t } = useI18n()
const projectStore = useProjectStore()
const storyboardStore = useStoryboardStore()
const projectId = String(route.params.projectId)

const dirtyItemIds = ref<Set<string>>(new Set())
const savingItemId = ref<string | null>(null)
const generatingItemIds = ref<Set<string>>(new Set())
const isSavingAll = ref(false)
const isBulkGenerating = ref(false)
const isPromptPreviewOpen = ref(false)
const isSnapshotOpen = ref(false)
const promptPreview = ref<ImagePromptPreviewDto | null>(null)
const promptPreviewLoadingItemId = ref<string | null>(null)
const snapshotCandidate = ref<ImageCandidateDto | null>(null)
const characterBibles = ref<CharacterBibleDto[]>([])
const locationBibles = ref<LocationBibleDto[]>([])
const characterResourcePlans = ref<Record<string, CharacterResourcePlanDto>>({})
type ImageGenerationKind = 'storyboard_image' | GeneratedImageKind
const selectedImageKind = ref<ImageGenerationKind>('storyboard_image')
const selectedAssetOwnerId = ref<string | null>(null)
const selectedReferenceRole = ref<string | null>(null)

const storyboardItems = computed(() => storyboardStore.storyboard?.items ?? [])
const storyboardCharacterOptions = computed(() => characterBibles.value.map((character) => ({ label: `${character.name} · ${character.characterId}`, value: character.characterId })))
const storyboardLocationOptions = computed(() => locationBibles.value.map((location) => ({ label: `${location.name} · ${location.locationId}`, value: location.locationId })))
const workspaceAccess = computed(() => getWorkspaceStepAccess(storyboardItems.value, storyboardStore.storyboard?.reviewStatus))
const projectTitle = computed(() => (projectStore.currentProject?.project.projectId === projectId ? projectStore.currentProject.project.title : projectId))
const isMockImageFlow = computed(() => storyboardItems.value.some((item) => item.imagePrompt.startsWith('MOCK') || item.imageCandidates.some((candidate) => candidate.providerModelId.startsWith('mock'))))
const imageResetCount = computed(() => storyboardItems.value.filter((item) => imageResetRecord(item)).length)
const bulkImageLockedCount = computed(() => storyboardItems.value.filter(isStoryboardItemLockedForBulkImageGeneration).length)
const resourceUsage = computed(() => ({
  images: storyboardItems.value.reduce((total, item) => total + item.imageCandidates.length, 0),
  videos: storyboardItems.value.reduce((total, item) => total + item.videoSegments.length, 0),
  llm: null,
}))
const imageKindOptions = computed(() =>
  IMAGE_KIND_OPTIONS.map((option) => ({
    label: t(`imageGeneration.imageKind.options.${option.value}`),
    value: option.value,
  }))
)
const assetOwnerOptions = computed(() => {
  const detail = projectStore.currentProject
  if (!detail) return []
  if (selectedImageKind.value === 'character_reference') {
    return characterBibles.value.map((item) => ({ label: item.name, value: item.characterId }))
  }
  if (selectedImageKind.value === 'scene_reference') {
    return locationBibles.value.map((item) => ({ label: item.name, value: item.locationId }))
  }
  if (selectedImageKind.value === 'style_reference') {
    const bible = detail.styleBible
    return bible ? [{ label: bible.name, value: bible.id }] : []
  }
  if (selectedImageKind.value === 'prop_reference' || selectedImageKind.value === 'cover_image') {
    return [{ label: projectTitle.value, value: projectId }]
  }
  return []
})
const referenceRoleOptions = computed(() =>
  referenceRolesForImageKind(selectedImageKind.value).map((role) => ({
    label: referenceRoleLabel(role),
    value: role,
  }))
)
const selectedImageKindLabel = computed(() => t(`imageGeneration.imageKind.options.${selectedImageKind.value}`))
const selectedImageKindHint = computed(() => t(`imageGeneration.imageKind.hints.${selectedImageKind.value}`))

watch([selectedImageKind, assetOwnerOptions], syncSelectedAssetOwner)
watch([selectedImageKind, referenceRoleOptions], syncSelectedReferenceRole)

onMounted(async () => {
  await Promise.all([projectStore.loadProject(projectId), storyboardStore.loadStoryboard(projectId)])
  const [characters, locations] = await Promise.all([listProjectCharacterBibles(projectId), listProjectLocationBibles(projectId)])
  characterBibles.value = characters
  locationBibles.value = locations
  await loadCharacterResourcePlans()
  syncSelectedAssetOwner()
  syncSelectedReferenceRole()

  if (!workspaceAccess.value.canEnterImage) {
    message.warning(t('workspaceStepBar.blocked.image'))
    await router.replace(getWorkspaceStepPath(projectId, getRequiredWorkspaceStep('image', workspaceAccess.value)))
  }
})

const IMAGE_KIND_OPTIONS: Array<{ value: ImageGenerationKind }> = [
  { value: 'storyboard_image' },
  { value: 'character_reference' },
  { value: 'scene_reference' },
  { value: 'style_reference' },
  { value: 'prop_reference' },
  { value: 'end_frame' },
  { value: 'control_image' },
  { value: 'cover_image' },
]
const CHARACTER_REFERENCE_ROLE_OPTIONS = [
  'character_front_view',
  'character_side_view',
  'character_back_view',
  'character_full_body',
  'character_face_closeup',
  'character_expression_sheet',
  'character_outfit',
  'character_pose',
  'character_mood',
]
const SCENE_REFERENCE_ROLE_OPTIONS = [
  'scene_wide_view',
  'scene_layout_view',
  'scene_detail_view',
  'scene_day_variant',
  'scene_night_variant',
]

function updateItem(item: StoryboardItemDto, patch: Partial<StoryboardItemDto>, lockedField?: StoryboardItemLockField) {
  if (lockedField && isItemLocked(item, lockedField)) {
    showLockedWarning(lockedField)
    return
  }
  Object.assign(item, patch)
  markItemDirty(item.itemId)
}

function updateCharacterIds(item: StoryboardItemDto, value: Array<string | number>) {
  updateItem(item, { characterIds: value.map(String).filter(Boolean) }, 'characters')
}

function updateLocationId(item: StoryboardItemDto, value: string | null) {
  if (isItemLocked(item, 'location')) {
    showLockedWarning('location')
    return
  }
  const locationId = value?.trim() || null
  const location = locationId ? locationBibles.value.find((entry) => entry.locationId === locationId) : null
  updateItem(item, {
    locationId,
    sceneDescription: location ? location.name : item.sceneDescription,
  }, 'location')
}

function isItemLocked(item: StoryboardItemDto, field: StoryboardItemLockField) {
  return isStoryboardItemLocked(item, field)
}

function toggleItemLock(item: StoryboardItemDto, field: StoryboardItemLockField) {
  Object.assign(item, setStoryboardItemLock(item, field, !isItemLocked(item, field)))
  markItemDirty(item.itemId)
}

function lockButtonLabel(item: StoryboardItemDto, field: StoryboardItemLockField) {
  return isItemLocked(item, field) ? t('common.unlock') : t('common.lock')
}

function lockFieldLabel(field: StoryboardItemLockField) {
  return t(`storyboard.lockFields.${field}`)
}

function showLockedWarning(field: StoryboardItemLockField) {
  message.warning(t('imageGeneration.lockedActionBlocked', { field: lockFieldLabel(field) }))
}

async function loadCharacterResourcePlans() {
  const entries = await Promise.all(
    storyboardItems.value.map(async (item) => {
      try {
        const plan = await buildCharacterResourcePlan({ projectId, itemId: item.itemId })
        return [item.itemId, plan] as const
      } catch {
        return [item.itemId, null] as const
      }
    })
  )
  characterResourcePlans.value = Object.fromEntries(entries.filter((entry): entry is readonly [string, CharacterResourcePlanDto] => Boolean(entry[1])))
}

function markItemDirty(itemId: string) {
  dirtyItemIds.value = new Set(dirtyItemIds.value).add(itemId)
}

function markItemClean(itemId: string) {
  const next = new Set(dirtyItemIds.value)
  next.delete(itemId)
  dirtyItemIds.value = next
}

async function handleSaveItem(item: StoryboardItemDto) {
  savingItemId.value = item.itemId
  try {
    await storyboardStore.saveScene(normalizeImageItem(item))
    markItemClean(item.itemId)
    await refreshCharacterResourcePlan(item.itemId)
    message.success(t('imageGeneration.saveSuccess'))
  } finally {
    savingItemId.value = null
  }
}

async function handleSaveAll() {
  if (dirtyItemIds.value.size === 0) return

  isSavingAll.value = true
  try {
    const dirtyIds = [...dirtyItemIds.value]
    for (const itemId of dirtyIds) {
      const item = storyboardItems.value.find((entry) => entry.itemId === itemId)
      if (item) await storyboardStore.saveScene(normalizeImageItem(item))
    }
    dirtyItemIds.value = new Set()
    await loadCharacterResourcePlans()
    message.success(t('imageGeneration.saveAllSuccess'))
  } finally {
    isSavingAll.value = false
  }
}

async function handleGenerateItem(item: StoryboardItemDto) {
  const lockedFields = lockedFieldsForImageGeneration(item)
  if (selectedImageKind.value === 'storyboard_image' && lockedFields.length > 0) {
    showLockedWarning(lockedFields[0])
    return
  }
  await saveDirtyItemIfNeeded(item)
  await refreshCharacterResourcePlan(item.itemId)
  setGenerating(item.itemId, true)
  try {
    if (selectedImageKind.value === 'storyboard_image') {
      await storyboardStore.generateImages(projectId, item.itemId)
      message.success(t('imageGeneration.generateSuccess', { index: item.index }))
    } else {
      const owner = resolveAssetOwner(item)
      if (!owner) {
        message.error(t('imageGeneration.imageKind.ownerRequired'))
        return
      }
      if (!selectedReferenceRole.value) {
        message.error(t('imageGeneration.imageKind.referenceRoleRequired'))
        return
      }
      const generated = await storyboardStore.generateImageAsset(createAssetGenerationRequest(item, owner))
      await refreshBibleContextAfterAssetGeneration()
      await refreshCharacterResourcePlan(item.itemId)
      message.success(t('imageGeneration.assetGenerateSuccess', { count: generated.length, kind: selectedImageKindLabel.value }))
    }
  } catch (error) {
    message.error(t('imageGeneration.generateFailed', { index: item.index, reason: errorMessage(error) }))
    throw error
  } finally {
    setGenerating(item.itemId, false)
  }
}

async function handleOpenPromptPreview(item: StoryboardItemDto) {
  await saveDirtyItemIfNeeded(item)
  promptPreviewLoadingItemId.value = item.itemId
  try {
    promptPreview.value = await buildImagePromptPreview({ projectId, itemId: item.itemId })
    isPromptPreviewOpen.value = true
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    promptPreviewLoadingItemId.value = null
  }
}

async function handleGenerateAll() {
  await generateItems(storyboardItems.value.filter((item) => !isStoryboardItemLockedForBulkImageGeneration(item)))
}

async function handleGenerateMissing() {
  await generateItems(storyboardItems.value.filter((item) => item.imageCandidates.length === 0 && !isStoryboardItemLockedForBulkImageGeneration(item)))
}

async function generateItems(items: StoryboardItemDto[]) {
  if (items.length === 0) {
    message.info(t(bulkImageLockedCount.value > 0 ? 'imageGeneration.noRowsAfterLockSkip' : 'imageGeneration.noMissingRows'))
    return
  }

  isBulkGenerating.value = true
  let successCount = 0
  let failedCount = 0
  try {
    await handleSaveAll()
    for (const item of items) {
      setGenerating(item.itemId, true)
      try {
        await storyboardStore.generateImages(projectId, item.itemId)
        successCount += 1
      } catch (error) {
        failedCount += 1
        message.error(t('imageGeneration.generateFailed', { index: item.index, reason: errorMessage(error) }))
      } finally {
        setGenerating(item.itemId, false)
      }
    }
    if (failedCount > 0) {
      message.warning(t('imageGeneration.bulkGeneratePartial', { success: successCount, failed: failedCount }))
    } else {
      message.success(t('imageGeneration.bulkGenerateSuccess', { count: successCount }))
    }
  } finally {
    generatingItemIds.value = new Set()
    isBulkGenerating.value = false
  }
}

async function handleSelectCandidate(item: StoryboardItemDto, candidate: ImageCandidateDto) {
  if (isItemLocked(item, 'selectedImage')) {
    showLockedWarning('selectedImage')
    return
  }
  await storyboardStore.selectImage(item.itemId, candidate.imageId)
  message.success(t('imageGeneration.selectSuccess', { index: item.index }))
}

function handleOpenSnapshot(candidate: ImageCandidateDto) {
  snapshotCandidate.value = candidate
  isSnapshotOpen.value = true
}

async function handleClearHistoricalCandidates(item: StoryboardItemDto) {
  await storyboardStore.clearOldImageCandidates(item.itemId)
  message.success(t('imageGeneration.clearHistoricalSuccess', { index: item.index }))
}

async function handleEnterVideo() {
  if (dirtyItemIds.value.size > 0) await handleSaveAll()

  const issues = validateStoryboardItemsForVideoGeneration(storyboardItems.value)
  if (issues.length > 0) {
    showFirstVideoIssue(issues[0])
    return
  }

  await router.push(`/projects/${projectId}/workspace/video`)
}

async function saveDirtyItemIfNeeded(item: StoryboardItemDto) {
  if (dirtyItemIds.value.has(item.itemId)) await handleSaveItem(item)
}

async function refreshCharacterResourcePlan(itemId: string) {
  try {
    const plan = await buildCharacterResourcePlan({ projectId, itemId })
    characterResourcePlans.value = { ...characterResourcePlans.value, [itemId]: plan }
  } catch {
    const next = { ...characterResourcePlans.value }
    delete next[itemId]
    characterResourcePlans.value = next
  }
}

function setGenerating(itemId: string, loading: boolean) {
  const next = new Set(generatingItemIds.value)
  if (loading) next.add(itemId)
  else next.delete(itemId)
  generatingItemIds.value = next
}

function selectedCandidate(item: StoryboardItemDto) {
  return item.imageCandidates.find((candidate) => candidate.imageId === item.selectedImageId || candidate.selected) ?? null
}

function sortedImageCandidates(item: StoryboardItemDto) {
  return [...item.imageCandidates].sort((left, right) => {
    const revisionDelta = candidateRevision(right) - candidateRevision(left)
    if (revisionDelta !== 0) return revisionDelta
    return candidateVariantIndex(left, 0) - candidateVariantIndex(right, 0)
  })
}

function isCandidateSelected(item: StoryboardItemDto, candidate: ImageCandidateDto) {
  return item.selectedImageId === candidate.imageId || candidate.selected
}

function hasClearableImageCandidates(item: StoryboardItemDto) {
  const latestRevision = latestCandidateRevision(item)
  return item.imageCandidates.some((candidate) => candidateRevision(candidate) < latestRevision && !isCandidateSelected(item, candidate))
}

function canGenerateSelectedAssetKind(item: StoryboardItemDto) {
  return Boolean(resolveAssetOwner(item))
}

function generateButtonLabel(item: StoryboardItemDto) {
  if (selectedImageKind.value === 'storyboard_image') return item.imageCandidates.length > 0 ? t('imageGeneration.regenerateRow') : t('imageGeneration.generateRow')
  return t('imageGeneration.generateAssetRow')
}

function resolveAssetOwner(item: StoryboardItemDto): { ownerKind: string; ownerId: string } | null {
  if (selectedImageKind.value === 'storyboard_image') return null
  if (selectedImageKind.value === 'end_frame' || selectedImageKind.value === 'control_image') return { ownerKind: 'storyboard_item', ownerId: item.itemId }
  const ownerId = selectedAssetOwnerId.value
  if (!ownerId) return null
  if (selectedImageKind.value === 'character_reference') return { ownerKind: 'character_bible', ownerId }
  if (selectedImageKind.value === 'scene_reference') return { ownerKind: 'location_bible', ownerId }
  if (selectedImageKind.value === 'style_reference') return { ownerKind: 'style_bible', ownerId }
  if (selectedImageKind.value === 'prop_reference' || selectedImageKind.value === 'cover_image') return { ownerKind: 'project', ownerId }
  return null
}

function createAssetGenerationRequest(item: StoryboardItemDto, owner: { ownerKind: string; ownerId: string }): StartImageAssetGenerationRequest {
  const imageKind = selectedImageKind.value as GeneratedImageKind
  return {
    projectId,
    imageKind,
    assetKind: defaultAssetKindForImageKind(imageKind),
    ownerKind: owner.ownerKind,
    ownerId: owner.ownerId,
    referenceRole: selectedReferenceRole.value ?? defaultReferenceRoleForImageKind(imageKind),
    itemId: item.itemId,
    prompt: item.imagePrompt,
    negativePrompt: item.negativePrompt,
    count: 1,
  }
}

function defaultAssetKindForImageKind(imageKind: GeneratedImageKind) {
  if (imageKind === 'character_reference') return 'character_reference_image'
  if (imageKind === 'scene_reference') return 'scene_reference_image'
  if (imageKind === 'style_reference') return 'style_reference_image'
  if (imageKind === 'cover_image') return 'cover_source'
  return 'generated_output'
}

function syncSelectedAssetOwner() {
  if (selectedImageKind.value === 'storyboard_image' || selectedImageKind.value === 'end_frame' || selectedImageKind.value === 'control_image') {
    selectedAssetOwnerId.value = null
    return
  }
  if (!assetOwnerOptions.value.some((option) => option.value === selectedAssetOwnerId.value)) {
    selectedAssetOwnerId.value = assetOwnerOptions.value[0]?.value ?? null
  }
}

function syncSelectedReferenceRole() {
  if (selectedImageKind.value === 'storyboard_image') {
    selectedReferenceRole.value = null
    return
  }
  if (!referenceRoleOptions.value.some((option) => option.value === selectedReferenceRole.value)) {
    selectedReferenceRole.value = referenceRoleOptions.value[0]?.value ?? null
  }
}

async function refreshBibleContextAfterAssetGeneration() {
  if (selectedImageKind.value === 'character_reference') {
    characterBibles.value = await listProjectCharacterBibles(projectId)
  } else if (selectedImageKind.value === 'scene_reference') {
    locationBibles.value = await listProjectLocationBibles(projectId)
  } else if (selectedImageKind.value === 'style_reference') {
    await projectStore.loadProject(projectId)
  }
}

function referenceRolesForImageKind(imageKind: ImageGenerationKind) {
  if (imageKind === 'character_reference') return CHARACTER_REFERENCE_ROLE_OPTIONS
  if (imageKind === 'scene_reference') return SCENE_REFERENCE_ROLE_OPTIONS
  if (imageKind === 'style_reference') return ['style_reference']
  if (imageKind === 'prop_reference') return ['prop_reference']
  if (imageKind === 'cover_image') return ['cover_image']
  if (imageKind === 'end_frame') return ['end_frame']
  if (imageKind === 'control_image') return ['control_image', 'pose_reference', 'depth_reference', 'mask_reference']
  return []
}

function defaultReferenceRoleForImageKind(imageKind: GeneratedImageKind) {
  return referenceRolesForImageKind(imageKind)[0] ?? imageKind
}

function referenceRoleLabel(role: string) {
  const key = `imageGeneration.imageKind.referenceRoles.${role}`
  const translated = t(key)
  return translated === key ? role : translated
}

function candidateFreshnessLabel(item: StoryboardItemDto, candidate: ImageCandidateDto) {
  return candidateRevision(candidate) === latestCandidateRevision(item) ? t('imageGeneration.latestCandidate') : t('imageGeneration.historicalCandidate')
}

function latestCandidateRevision(item: StoryboardItemDto) {
  return Math.max(0, ...item.imageCandidates.map(candidateRevision))
}

function candidateRevision(candidate: ImageCandidateDto) {
  const revision = candidate.generationContextSnapshot.revision
  return typeof revision === 'number' && Number.isFinite(revision) ? revision : 1
}

function candidateVariantIndex(candidate: ImageCandidateDto, fallbackIndex: number) {
  const variantIndex = candidate.generationContextSnapshot.variantIndex
  return typeof variantIndex === 'number' && Number.isFinite(variantIndex) ? variantIndex : fallbackIndex + 1
}

function candidateButtonClass(item: StoryboardItemDto, candidate: ImageCandidateDto) {
  return isCandidateSelected(item, candidate) ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-page text-secondary hover:border-border-strong hover:text-primary'
}

function imageResetRecord(item: StoryboardItemDto) {
  const record = getLatestStoryboardResetRecord(item)
  return isResetRelevantToImage(record) ? record : null
}

function resetReasonLabel(triggerField?: string) {
  return triggerField ? t(`imageGeneration.downstreamReset.reasons.${triggerField}`) : t('imageGeneration.downstreamReset.reasons.unknown')
}

function candidatePreviewClass(candidate: ImageCandidateDto, fallbackIndex: number) {
  const toneClass = candidate.generationContextSnapshot.visualToneClass
  if (typeof toneClass === 'string' && /^scene-preview-tone-[0-4]$/.test(toneClass)) return toneClass
  return `scene-preview-tone-${fallbackIndex % 5}`
}

function imageErrorReason(item: StoryboardItemDto) {
  const error = item.imageLastErrorJson
  if (!error) return '-'
  const code = error.code ?? error.errorCode
  const message = error.message
  if (typeof code === 'string' && typeof message === 'string') return `${code}: ${message}`
  if (typeof message === 'string') return message
  if (typeof code === 'string') return code
  return '-'
}

function errorMessage(error: unknown) {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  return String(error)
}

function snapshotArray(source: unknown, key: string): Record<string, unknown>[] {
  if (!source || typeof source !== 'object') return []
  const value = (source as Record<string, unknown>)[key]
  return Array.isArray(value) ? (value.filter((entry) => entry && typeof entry === 'object') as Record<string, unknown>[]) : []
}

function snapshotText(source: unknown, key: string) {
  if (!source || typeof source !== 'object') return ''
  const value = (source as Record<string, unknown>)[key]
  return typeof value === 'string' ? value : ''
}

function formatSnapshotJson(value: unknown) {
  return JSON.stringify(value ?? null, null, 2)
}

function promptPreviewSectionLabel(section: PromptSectionDto) {
  const key = `imageGeneration.promptPreview.sectionLabels.${section.key}`
  const translated = t(key)
  return translated === key ? section.label : translated
}

function normalizeImageItem(item: StoryboardItemDto): StoryboardItemDto {
  return {
    ...item,
    characterIds: normalizeStringList(item.characterIds),
    sceneDescription: item.sceneDescription.trim(),
    visualDescription: item.visualDescription.trim(),
    imagePrompt: item.imagePrompt.trim(),
    negativePrompt: item.negativePrompt.trim(),
  }
}

function formatCharacters(item: StoryboardItemDto) {
  if (item.characterIds.length === 0) return t('imageGeneration.placeholders.characterIds')
  return item.characterIds
    .map((characterId) => characterBibles.value.find((character) => character.characterId === characterId)?.name ?? characterId)
    .join('、')
}

function formatLocation(item: StoryboardItemDto) {
  if (!item.locationId) return item.sceneDescription || t('imageGeneration.placeholders.scene')
  const location = locationBibles.value.find((entry) => entry.locationId === item.locationId)
  return location ? `${location.name} · ${location.locationId}` : item.locationId
}

function characterResourceMissingCount(item: StoryboardItemDto) {
  return characterResourcePlans.value[item.itemId]?.missingRequiredCount ?? 0
}

function characterResourceOptionalCount(item: StoryboardItemDto) {
  const plan = characterResourcePlans.value[item.itemId]
  if (!plan) return 0
  return plan.items.filter((entry) => entry.requirement === 'optional' && !entry.available).length
}

function characterResourceSummary(item: StoryboardItemDto) {
  const plan = characterResourcePlans.value[item.itemId]
  if (!plan) return item.characterIds.length > 0 ? t('imageGeneration.characterResources.loading') : t('imageGeneration.characterResources.noCharacters')
  const missing = plan.items.filter((entry) => entry.requirement === 'required' && !entry.available)
  if (missing.length > 0) return missing.map((entry) => `${entry.characterName}:${characterResourceRoleLabel(entry.role)}`).join(' / ')
  const optional = plan.items.filter((entry) => entry.requirement === 'optional' && !entry.available)
  if (optional.length > 0) return optional.slice(0, 3).map((entry) => `${entry.characterName}:${characterResourceRoleLabel(entry.role)}`).join(' / ')
  return item.characterIds.length > 0 ? t('imageGeneration.characterResources.ready') : t('imageGeneration.characterResources.noCharacters')
}

function characterResourceRoleLabel(role: string) {
  const key = `imageGeneration.characterResources.roles.${role}`
  const translated = t(key)
  return translated === key ? role : translated
}

function normalizeStringList(values: string[]) {
  return values.map((value) => value.trim()).filter(Boolean)
}

function shortId(value?: string) {
  if (!value) return '-'
  return value.length > 12 ? value.slice(-12) : value
}

function showFirstVideoIssue(issue: { index: number; fields: StoryboardVideoEntryField[] }) {
  const fields = issue.fields.map((field) => t(`imageGeneration.validation.fields.${field}`)).join('、')
  message.error(t('imageGeneration.validation.enterVideoBlocked', { index: issue.index, fields }))
}

function handleBlockedStep(step: WorkspaceStepKey) {
  message.warning(t(`workspaceStepBar.blocked.${step}`))
}

function rowDomId(index: number) {
  return `image-row-${index}`
}

function jumpToRow(index: number) {
  document.getElementById(rowDomId(index))?.scrollIntoView({ block: 'center', behavior: 'smooth' })
}
</script>
