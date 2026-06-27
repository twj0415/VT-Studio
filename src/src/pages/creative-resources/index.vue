<template>
  <section class="view">
    <div class="gpage">
      <div class="wrap flex min-h-0 flex-1 flex-col">
        <div class="phead">
          <div>
            <h1>{{ t('creativeResources.title') }}</h1>
            <div class="desc">{{ t('creativeResources.desc') }}</div>
          </div>
          <span class="rounded-vt-sm border border-accent-line bg-accent-soft px-vt-3 py-vt-2 text-xs font-medium text-accent">{{ t('creativeResources.stageBadge') }}</span>
        </div>

        <div class="grid min-h-0 gap-vt-4 xl:grid-cols-[260px_minmax(0,1fr)]">
          <aside class="rounded-vt-md border border-border bg-card p-vt-3 shadow-vt-md">
            <button v-for="entry in resourceEntries" :key="entry.key" type="button" class="mb-vt-2 flex w-full gap-vt-3 rounded-vt-sm border px-vt-3 py-vt-3 text-left transition" :class="activeEntry === entry.key ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-page text-secondary hover:border-border-strong hover:text-primary'" @click="activeEntry = entry.key">
              <span class="grid size-8 flex-none place-items-center rounded-vt-sm border border-current text-sm">{{ entry.icon }}</span>
              <span class="min-w-0">
                <span class="block font-medium">{{ t(`creativeResources.entries.${entry.key}.title`) }}</span>
                <span class="mt-vt-1 block text-xs leading-5 text-muted">{{ t(`creativeResources.entries.${entry.key}.desc`) }}</span>
              </span>
            </button>
          </aside>

          <main class="min-w-0">
            <section v-if="activeEntry === 'packs'" class="grid gap-vt-4 2xl:grid-cols-[300px_minmax(0,1fr)_420px]">
              <aside class="rounded-vt-md border border-border bg-card p-vt-4 shadow-vt-md">
                <div class="grid gap-vt-2">
                  <n-select v-model:value="packFilterSource" size="small" clearable :placeholder="t('creativeResources.packs.sourceAll')" :options="packSourceOptions" />
                  <n-button size="small" @click="toggleDisabledPacks">
                    {{ includeDisabledPacks ? t('creativeResources.packs.hideDisabled') : t('creativeResources.packs.showDisabled') }}
                  </n-button>
                </div>
                <div class="mt-vt-3 flex items-center gap-vt-2">
                  <h2 class="text-base font-semibold text-primary">{{ t('creativeResources.packs.listTitle') }}</h2>
                  <span class="ml-auto rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-xs text-muted">{{ t('creativeResources.packs.packCount', { count: filteredPacks.length }) }}</span>
                </div>
                <div class="mt-vt-3 grid gap-vt-2">
                  <button v-for="pack in filteredPacks" :key="pack.packId" type="button" class="rounded-vt-sm border p-vt-3 text-left text-xs transition" :class="selectedPackId === pack.packId ? 'border-accent-line bg-accent-soft' : 'border-border bg-page hover:border-border-strong'" @click="selectPack(pack)">
                    <div class="flex min-w-0 items-center gap-vt-2">
                      <span class="min-w-0 truncate font-semibold text-primary">{{ pack.name }}</span>
                      <span class="ml-auto rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ t(`creativeResources.packs.${pack.sourceType}`) }}</span>
                    </div>
                    <p class="mt-vt-1 line-clamp-2 text-muted">{{ pack.description }}</p>
                    <div class="mt-vt-2 flex flex-wrap gap-vt-1 text-muted">
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ pack.defaultAspectRatio }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ t('createProject.packSceneCount', { count: pack.defaultSceneCount }) }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ pack.isEnabled ? t('creativeResources.packs.enabled') : t('creativeResources.packs.disabled') }}</span>
                    </div>
                  </button>
                  <div v-if="filteredPacks.length === 0" class="rounded-vt-sm border border-border bg-page p-vt-4 text-center text-xs text-muted">{{ t('creativeResources.packs.empty') }}</div>
                </div>
              </aside>

              <div class="rounded-vt-md border border-border bg-card p-vt-4 shadow-vt-md">
                <div class="flex flex-wrap items-start gap-vt-3">
                  <div class="min-w-0">
                    <h2 class="text-base font-semibold text-primary">{{ t('creativeResources.packs.title') }}</h2>
                    <p class="mt-vt-1 text-sm leading-6 text-secondary">{{ t('creativeResources.packs.desc') }}</p>
                  </div>
                  <n-button class="ml-auto" size="small" :loading="isLoadingPacks" @click="loadVideoPacks">{{ t('creativeResources.packs.refresh') }}</n-button>
                </div>

                <div v-if="selectedPack" class="mt-vt-4 grid gap-vt-3">
                  <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                    <div class="flex flex-wrap items-center gap-vt-2 text-xs">
                      <span class="font-semibold text-primary">{{ selectedPack.name }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ t(`creativeResources.packs.${selectedPack.sourceType}`) }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ selectedPack.isEnabled ? t('creativeResources.packs.enabled') : t('creativeResources.packs.disabled') }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ selectedPack.contentCategory || '-' }}</span>
                    </div>
                    <div class="mt-vt-2 break-all font-mono text-[11px] text-muted">{{ selectedPack.packId }}</div>
                  </div>

                  <div class="grid gap-vt-3 lg:grid-cols-2">
                    <label class="grid gap-vt-1 text-xs text-muted">
                      <span>{{ t('creativeResources.packs.fields.name') }}</span>
                      <n-input v-model:value="packForm.name" size="small" :disabled="!canEditSelectedPack" />
                    </label>
                    <label class="grid gap-vt-1 text-xs text-muted">
                      <span>{{ t('creativeResources.packs.fields.contentCategory') }}</span>
                      <n-input v-model:value="packForm.contentCategory" size="small" :disabled="!canEditSelectedPack" />
                    </label>
                    <label class="grid gap-vt-1 text-xs text-muted">
                      <span>{{ t('creativeResources.packs.fields.aspectRatio') }}</span>
                      <n-select v-model:value="packForm.defaultAspectRatio" size="small" :options="packAspectRatioOptions" :disabled="!canEditSelectedPack" />
                    </label>
                    <label class="grid gap-vt-1 text-xs text-muted">
                      <span>{{ t('creativeResources.packs.fields.sceneCount') }}</span>
                      <n-input-number v-model:value="packForm.defaultSceneCount" size="small" :min="1" :max="60" :precision="0" :disabled="!canEditSelectedPack" />
                    </label>
                    <label class="grid gap-vt-1 text-xs text-muted">
                      <span>{{ t('creativeResources.packs.fields.duration') }}</span>
                      <n-input-number v-model:value="packForm.defaultDurationSeconds" size="small" :min="1" :max="600" :precision="0" :disabled="!canEditSelectedPack" />
                    </label>
                    <label class="flex items-center gap-vt-2 pt-vt-5 text-xs text-secondary">
                      <n-switch v-model:value="packForm.isEnabled" size="small" :disabled="!canEditSelectedPack" />
                      <span>{{ t('creativeResources.packs.fields.enabled') }}</span>
                    </label>
                    <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                      <span>{{ t('creativeResources.packs.fields.inputTypes') }}</span>
                      <n-select v-model:value="packForm.applicableInputTypes" size="small" multiple :options="packInputTypeOptions" :disabled="!canEditSelectedPack" />
                    </label>
                    <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                      <span>{{ t('creativeResources.packs.fields.tone') }}</span>
                      <n-input v-model:value="packForm.defaultTone" size="small" :disabled="!canEditSelectedPack" />
                    </label>
                    <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                      <span>{{ t('creativeResources.packs.fields.description') }}</span>
                      <n-input v-model:value="packForm.description" size="small" type="textarea" :disabled="!canEditSelectedPack" :autosize="{ minRows: 3, maxRows: 6 }" />
                    </label>
                  </div>

                  <div class="flex flex-wrap items-center gap-vt-2">
                    <n-button v-if="selectedPack.sourceType === 'builtin'" size="small" type="primary" :loading="isCloningPack" @click="handleClonePack">{{ t('creativeResources.packs.clone') }}</n-button>
                    <n-button v-else size="small" type="primary" :loading="isSavingPack" @click="handleSavePack">{{ t('creativeResources.packs.save') }}</n-button>
                    <n-button v-if="selectedPack.sourceType === 'user'" size="small" :loading="isTogglingPack" @click="handleTogglePack">{{ selectedPack.isEnabled ? t('creativeResources.packs.disable') : t('creativeResources.packs.enable') }}</n-button>
                    <n-popconfirm v-if="selectedPack.sourceType === 'user'" :positive-text="t('common.confirm')" :negative-text="t('common.cancel')" @positive-click="handleDeletePack">
                      <template #trigger>
                        <n-button size="small" :loading="isDeletingPack" :disabled="selectedPack.projectReferenceCount > 0">{{ t('creativeResources.packs.delete') }}</n-button>
                      </template>
                      {{ t('creativeResources.packs.deleteConfirm') }}
                    </n-popconfirm>
                  </div>

                  <div class="rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-muted">
                    {{ selectedPack.sourceType === 'builtin' ? t('creativeResources.packs.builtinBoundary') : t('creativeResources.packs.userBoundary') }}
                  </div>
                </div>

                <div v-else class="mt-vt-4 rounded-vt-sm border border-border bg-page p-vt-4 text-center text-xs text-muted">
                  {{ t('creativeResources.packs.emptySelect') }}
                </div>
              </div>

              <aside class="rounded-vt-md border border-border bg-card p-vt-4 shadow-vt-md">
                <template v-if="selectedPack">
                  <div class="flex items-center gap-vt-2">
                    <h2 class="text-base font-semibold text-primary">{{ t('creativeResources.packs.referencesTitle') }}</h2>
                    <span class="ml-auto rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-xs text-muted">{{ selectedPack.projectReferenceCount > 0 ? t('creativeResources.packs.projectRefs', { count: selectedPack.projectReferenceCount }) : t('creativeResources.packs.noProjectRefs') }}</span>
                  </div>
                  <div class="mt-vt-4 grid gap-vt-3">
                    <label class="grid gap-vt-1 text-xs text-muted">
                      <span>{{ t('creativeResources.packs.fields.ruleRefs') }}</span>
                      <n-input v-model:value="ruleRefsJson" size="small" type="textarea" :disabled="!canEditSelectedPack" :autosize="{ minRows: 6, maxRows: 12 }" />
                    </label>
                    <label class="grid gap-vt-1 text-xs text-muted">
                      <span>{{ t('creativeResources.packs.fields.executableRefs') }}</span>
                      <n-input v-model:value="executableRefsJson" size="small" type="textarea" :disabled="!canEditSelectedPack" :autosize="{ minRows: 6, maxRows: 12 }" />
                    </label>
                    <label class="grid gap-vt-1 text-xs text-muted">
                      <span>{{ t('creativeResources.packs.fields.assetRefs') }}</span>
                      <n-input v-model:value="assetRefsJson" size="small" type="textarea" :disabled="!canEditSelectedPack" :autosize="{ minRows: 4, maxRows: 10 }" />
                    </label>
                  </div>
                </template>
                <div v-else class="rounded-vt-sm border border-border bg-page p-vt-5 text-center text-xs text-muted">{{ t('creativeResources.packs.emptySelect') }}</div>
              </aside>
            </section>

            <section v-else-if="activeEntry === 'assets'" class="grid gap-vt-4 2xl:grid-cols-[minmax(0,1fr)_420px]">
              <div class="rounded-vt-md border border-border bg-card p-vt-4 shadow-vt-md">
                <div class="flex flex-wrap items-start gap-vt-3">
                  <div class="min-w-0">
                    <h2 class="text-base font-semibold text-primary">{{ t('creativeResources.assets.title') }}</h2>
                    <p class="mt-vt-1 text-sm leading-6 text-secondary">{{ t('creativeResources.assets.desc') }}</p>
                  </div>
                  <div class="ml-auto flex flex-wrap items-center gap-vt-2">
                    <n-button size="small" type="primary" @click="openImportAssetModal">{{ t('creativeResources.assets.importAsset') }}</n-button>
                    <n-button size="small" :loading="isLoadingAssets" @click="loadAssets">{{ t('creativeResources.assets.refresh') }}</n-button>
                  </div>
                </div>

                <div class="mt-vt-4 grid gap-vt-3 lg:grid-cols-[220px_1fr_auto]">
                  <n-select v-model:value="assetFilterGroup" size="small" clearable :placeholder="t('creativeResources.assets.filterGroup')" :options="assetGroupOptions" />
                  <div class="rounded-vt-sm border border-border bg-page px-vt-3 py-vt-2 text-xs text-muted">
                    {{ t('creativeResources.assets.assetCount', { count: filteredAssets.length }) }}
                  </div>
                  <n-button size="small" @click="toggleDeletedAssets">
                    {{ includeDeletedAssets ? t('creativeResources.assets.hideDeleted') : t('creativeResources.assets.showDeleted') }}
                  </n-button>
                </div>

                <div class="mt-vt-4 overflow-hidden rounded-vt-sm border border-border">
                  <button v-for="asset in filteredAssets" :key="asset.assetId" type="button" class="grid w-full gap-vt-2 border-b border-border bg-page p-vt-3 text-left text-xs transition last:border-b-0 hover:bg-card" :class="selectedAssetId === asset.assetId ? 'bg-accent-soft' : ''" @click="selectAsset(asset)">
                    <div class="flex min-w-0 items-center gap-vt-2">
                      <span class="min-w-0 truncate font-semibold text-primary">{{ assetDisplayName(asset) }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ assetGroupLabel(asset) }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ asset.lifecycle }}</span>
                    </div>
                    <div class="truncate font-mono text-[11px] text-muted">{{ asset.relativePath }}</div>
                    <div class="flex flex-wrap gap-vt-1 text-muted">
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ asset.kind }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ asset.sourceKind }}</span>
                      <span v-if="asset.mimeType" class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ asset.mimeType }}</span>
                      <span v-if="asset.sizeBytes" class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ formatBytes(asset.sizeBytes) }}</span>
                    </div>
                  </button>
                  <div v-if="filteredAssets.length === 0" class="rounded-vt-sm bg-page p-vt-5 text-center text-xs text-muted">{{ t('creativeResources.assets.empty') }}</div>
                </div>
              </div>

              <aside class="rounded-vt-md border border-border bg-card p-vt-4 shadow-vt-md">
                <template v-if="selectedAsset">
                  <div class="flex flex-wrap items-start gap-vt-3">
                    <div class="min-w-0">
                      <h2 class="break-all text-base font-semibold text-primary">{{ assetDisplayName(selectedAsset) }}</h2>
                      <p class="mt-vt-1 break-all font-mono text-[11px] leading-5 text-muted">{{ selectedAsset.assetId }}</p>
                    </div>
                    <span class="ml-auto rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-xs text-muted">{{ selectedAsset.lifecycle }}</span>
                  </div>

                  <div class="mt-vt-4 grid gap-vt-2 text-xs">
                    <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                      <div class="font-semibold text-primary">{{ t('creativeResources.assets.preview') }}</div>
                      <div class="mt-vt-2 overflow-hidden rounded-vt-sm border border-border bg-card">
                        <img v-if="assetPreviewUrl" :src="assetPreviewUrl" :alt="assetDisplayName(selectedAsset)" class="max-h-64 w-full object-contain" />
                        <div v-else class="grid min-h-32 place-items-center p-vt-4 text-center text-muted">
                          <span v-if="isLoadingAssetPreview">{{ t('creativeResources.assets.previewLoading') }}</span>
                          <span v-else-if="assetPreviewError" class="break-all text-status-warning">{{ assetPreviewError }}</span>
                          <span v-else>{{ previewPlaceholder }}</span>
                        </div>
                      </div>
                      <div v-if="isProbingAsset || selectedAssetProbe || assetProbeError" class="mt-vt-2 rounded-vt-sm border border-border bg-card p-vt-2 text-muted">
                        <div v-if="isProbingAsset">{{ t('creativeResources.assets.probeLoading') }}</div>
                        <div v-else-if="selectedAssetProbe" class="grid gap-vt-1">
                          <div>{{ probeSummary(selectedAssetProbe) }}</div>
                          <div class="break-all font-mono text-[11px]">{{ selectedAssetProbe.path }}</div>
                        </div>
                        <div v-else class="break-all text-status-warning">{{ assetProbeError }}</div>
                      </div>
                    </div>

                    <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                      <div class="font-semibold text-primary">{{ t('creativeResources.assets.details') }}</div>
                      <dl class="mt-vt-2 grid gap-vt-2 text-muted">
                        <div class="grid grid-cols-[90px_minmax(0,1fr)] gap-vt-2">
                          <dt>{{ t('creativeResources.assets.fields.kind') }}</dt>
                          <dd class="break-all text-secondary">{{ selectedAsset.kind }}</dd>
                        </div>
                        <div class="grid grid-cols-[90px_minmax(0,1fr)] gap-vt-2">
                          <dt>{{ t('creativeResources.assets.fields.mediaKind') }}</dt>
                          <dd class="break-all text-secondary">{{ selectedAssetMediaKind }}</dd>
                        </div>
                        <div class="grid grid-cols-[90px_minmax(0,1fr)] gap-vt-2">
                          <dt>{{ t('creativeResources.assets.fields.sourceKind') }}</dt>
                          <dd class="break-all text-secondary">{{ selectedAsset.sourceKind }}</dd>
                        </div>
                        <div class="grid grid-cols-[90px_minmax(0,1fr)] gap-vt-2">
                          <dt>{{ t('creativeResources.assets.fields.mimeType') }}</dt>
                          <dd class="break-all text-secondary">{{ selectedAsset.mimeType || '-' }}</dd>
                        </div>
                        <div class="grid grid-cols-[90px_minmax(0,1fr)] gap-vt-2">
                          <dt>{{ t('creativeResources.assets.fields.size') }}</dt>
                          <dd class="break-all text-secondary">{{ selectedAsset.sizeBytes ? formatBytes(selectedAsset.sizeBytes) : '-' }}</dd>
                        </div>
                        <div class="grid grid-cols-[90px_minmax(0,1fr)] gap-vt-2">
                          <dt>{{ t('creativeResources.assets.fields.dimensions') }}</dt>
                          <dd class="break-all text-secondary">{{ assetDimensionsText }}</dd>
                        </div>
                        <div class="grid grid-cols-[90px_minmax(0,1fr)] gap-vt-2">
                          <dt>{{ t('creativeResources.assets.fields.duration') }}</dt>
                          <dd class="break-all text-secondary">{{ assetDurationText }}</dd>
                        </div>
                        <div class="grid grid-cols-[90px_minmax(0,1fr)] gap-vt-2">
                          <dt>{{ t('creativeResources.assets.fields.path') }}</dt>
                          <dd class="break-all font-mono text-[11px] text-secondary">{{ selectedAsset.relativePath }}</dd>
                        </div>
                        <div class="grid grid-cols-[90px_minmax(0,1fr)] gap-vt-2">
                          <dt>{{ t('creativeResources.assets.fields.checksum') }}</dt>
                          <dd class="break-all font-mono text-[11px] text-secondary">{{ selectedAsset.checksum || '-' }}</dd>
                        </div>
                      </dl>
                    </div>

                    <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                      <div class="flex items-center gap-vt-2">
                        <div class="font-semibold text-primary">{{ t('creativeResources.assets.references') }}</div>
                        <span class="ml-auto rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ selectedAssetReferences.length }}</span>
                      </div>
                      <div class="mt-vt-3 grid gap-vt-2">
                        <div v-for="reference in selectedAssetReferences" :key="reference.referenceId" class="rounded-vt-sm border border-border bg-card p-vt-2">
                          <div class="flex items-center gap-vt-2">
                            <span class="font-semibold text-primary">{{ reference.ownerKind }}</span>
                            <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-0.5 text-muted">{{ reference.usageKind }}</span>
                          </div>
                          <div class="mt-vt-1 break-all font-mono text-[11px] text-muted">{{ reference.ownerId }}</div>
                          <n-popconfirm :positive-text="t('common.confirm')" :negative-text="t('common.cancel')" @positive-click="handleDeleteReference(reference)">
                            <template #trigger>
                              <n-button class="mt-vt-2" size="tiny" :loading="deletingReferenceId === reference.referenceId">{{ t('creativeResources.assets.unlinkReference') }}</n-button>
                            </template>
                            {{ t('creativeResources.assets.unlinkConfirm') }}
                          </n-popconfirm>
                        </div>
                        <div v-if="selectedAssetReferences.length === 0" class="rounded-vt-sm border border-border bg-card p-vt-3 text-muted">{{ t('creativeResources.assets.noReferences') }}</div>
                      </div>
                    </div>

                    <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                      <div class="font-semibold text-primary">{{ t('creativeResources.assets.metadata') }}</div>
                      <pre class="mt-vt-2 max-h-56 overflow-auto rounded-vt-sm border border-border bg-card p-vt-3 font-mono text-[11px] leading-5 text-secondary">{{ selectedAssetMetadataJson }}</pre>
                    </div>
                  </div>

                  <div class="mt-vt-4 flex flex-wrap items-center gap-vt-2">
                    <n-popconfirm v-if="selectedAssetReferences.length === 0" :positive-text="t('common.confirm')" :negative-text="t('common.cancel')" @positive-click="handleDeleteAsset">
                      <template #trigger>
                        <n-button size="small" :loading="isDeletingAsset">{{ t('creativeResources.assets.delete') }}</n-button>
                      </template>
                      {{ t('creativeResources.assets.deleteConfirm') }}
                    </n-popconfirm>
                    <div v-else class="rounded-vt-sm border border-status-warning bg-page px-vt-3 py-vt-2 text-xs text-status-warning">{{ t('creativeResources.assets.deleteBlocked') }}</div>
                  </div>
                </template>
                <div v-else class="rounded-vt-sm border border-border bg-page p-vt-5 text-center text-xs text-muted">{{ t('creativeResources.assets.emptySelect') }}</div>
              </aside>
            </section>

            <section v-else class="grid gap-vt-4 2xl:grid-cols-[minmax(0,1fr)_430px]">
              <div class="rounded-vt-md border border-border bg-card p-vt-4 shadow-vt-md">
                <div class="flex flex-wrap items-center gap-vt-3">
                  <div class="min-w-0">
                    <h2 class="text-base font-semibold text-primary">{{ t('creativeResources.rules.title') }}</h2>
                    <p class="mt-vt-1 text-sm leading-6 text-secondary">{{ t('creativeResources.rules.desc') }}</p>
                  </div>
                  <n-button class="ml-auto" size="small" :loading="isLoadingRules" @click="loadRules">{{ t('creativeResources.rules.refresh') }}</n-button>
                </div>

                <div v-if="selectedRule" class="mt-vt-4 grid gap-vt-3">
                  <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                    <div class="flex flex-wrap items-center gap-vt-2 text-xs">
                      <span class="font-semibold text-primary">{{ selectedRule.name }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ selectedRule.sourceType }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ selectedRule.module }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ selectedRule.ruleType }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ selectedRule.enabled ? t('creativeResources.rules.enabled') : t('creativeResources.rules.disabled') }}</span>
                    </div>
                    <div class="mt-vt-2 break-all text-xs text-muted">{{ selectedRule.relativePath }}</div>
                    <p class="mt-vt-2 text-xs leading-5 text-secondary">{{ selectedRule.description }}</p>
                  </div>

                  <div class="grid gap-vt-2 rounded-vt-sm border border-border bg-page p-vt-3 text-xs text-muted sm:grid-cols-4">
                    <div>
                      <div class="text-primary">{{ selectedRule.referenceCounts.videoPacks }}</div>
                      <div>{{ t('creativeResources.rules.referenceCounts.videoPacks') }}</div>
                    </div>
                    <div>
                      <div class="text-primary">{{ selectedRule.referenceCounts.projects }}</div>
                      <div>{{ t('creativeResources.rules.referenceCounts.projects') }}</div>
                    </div>
                    <div>
                      <div class="text-primary">{{ selectedRule.referenceCounts.taskSteps }}</div>
                      <div>{{ t('creativeResources.rules.referenceCounts.taskSteps') }}</div>
                    </div>
                    <div>
                      <div class="text-primary">{{ selectedRule.referenceCounts.generationContexts }}</div>
                      <div>{{ t('creativeResources.rules.referenceCounts.generationContexts') }}</div>
                    </div>
                  </div>

                  <div class="grid gap-vt-3 lg:grid-cols-2">
                    <label class="grid gap-vt-1 text-xs text-muted">
                      <span>{{ t('creativeResources.rules.fields.key') }}</span>
                      <n-input v-model:value="ruleForm.key" size="small" :disabled="!canEditSelectedRule" />
                    </label>
                    <label class="grid gap-vt-1 text-xs text-muted">
                      <span>{{ t('creativeResources.rules.fields.name') }}</span>
                      <n-input v-model:value="ruleForm.name" size="small" :disabled="!canEditSelectedRule" />
                    </label>
                    <label class="grid gap-vt-1 text-xs text-muted">
                      <span>{{ t('creativeResources.rules.fields.module') }}</span>
                      <n-select v-model:value="ruleForm.module" size="small" :options="ruleModuleOptions" :disabled="!canEditSelectedRule" />
                    </label>
                    <label class="grid gap-vt-1 text-xs text-muted">
                      <span>{{ t('creativeResources.rules.fields.providerKind') }}</span>
                      <n-select v-model:value="ruleForm.providerKind" size="small" :options="providerKindOptions" :disabled="!canEditSelectedRule" />
                    </label>
                    <label class="grid gap-vt-1 text-xs text-muted">
                      <span>{{ t('creativeResources.rules.fields.ruleType') }}</span>
                      <n-select v-model:value="ruleForm.ruleType" size="small" :options="ruleTypeOptions" :disabled="!canEditSelectedRule" />
                    </label>
                    <label class="flex items-center gap-vt-2 pt-vt-5 text-xs text-secondary">
                      <n-switch v-model:value="ruleForm.enabled" size="small" :disabled="!canEditSelectedRule" />
                      <span>{{ t('creativeResources.rules.fields.enabled') }}</span>
                    </label>
                    <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                      <span>{{ t('creativeResources.rules.fields.description') }}</span>
                      <n-input v-model:value="ruleForm.description" size="small" :disabled="!canEditSelectedRule" />
                    </label>
                    <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                      <span>{{ t('creativeResources.rules.fields.outputSchema') }}</span>
                      <n-input v-model:value="outputSchemaJson" size="small" type="textarea" :disabled="!canEditSelectedRule" :autosize="{ minRows: 5, maxRows: 10 }" />
                    </label>
                    <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                      <span>{{ t('creativeResources.rules.fields.paramsSchema') }}</span>
                      <n-input v-model:value="paramsSchemaJson" size="small" type="textarea" :disabled="!canEditSelectedRule" :autosize="{ minRows: 4, maxRows: 10 }" />
                    </label>
                    <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                      <span>{{ t('creativeResources.rules.fields.body') }}</span>
                      <n-input v-model:value="ruleForm.body" size="small" type="textarea" :disabled="!canEditSelectedRule" :autosize="{ minRows: 8, maxRows: 18 }" />
                    </label>
                  </div>

                  <div class="flex flex-wrap items-center gap-vt-2">
                    <n-button v-if="selectedRule.sourceType === 'builtin'" size="small" type="primary" :loading="isCloningRule" @click="handleCloneRule">{{ t('creativeResources.rules.clone') }}</n-button>
                    <n-button v-else size="small" type="primary" :loading="isSavingRule" @click="handleSaveRule">{{ t('creativeResources.rules.save') }}</n-button>
                    <n-button v-if="selectedRule.sourceType === 'user'" size="small" :loading="isTogglingRule" :disabled="selectedRule.enabled && selectedRuleReferenceTotal > 0" @click="handleToggleRule">{{ selectedRule.enabled ? t('creativeResources.rules.disable') : t('creativeResources.rules.enable') }}</n-button>
                    <n-popconfirm v-if="selectedRule.sourceType === 'user'" :positive-text="t('common.confirm')" :negative-text="t('common.cancel')" @positive-click="handleDeleteRule">
                      <template #trigger>
                        <n-button size="small" :loading="isDeletingRule" :disabled="selectedRuleReferenceTotal > 0">{{ t('creativeResources.rules.delete') }}</n-button>
                      </template>
                      {{ t('creativeResources.rules.deleteConfirm') }}
                    </n-popconfirm>
                    <div v-if="selectedRule.sourceType === 'user' && selectedRuleReferenceTotal > 0" class="rounded-vt-sm border border-status-warning bg-page px-vt-3 py-vt-2 text-xs text-status-warning">
                      {{ t('creativeResources.rules.referenceBlocked', { count: selectedRuleReferenceTotal }) }}
                    </div>
                  </div>

                  <div class="rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-muted">
                    {{ selectedRule.sourceType === 'builtin' ? t('creativeResources.rules.builtinBoundary') : t('creativeResources.rules.userBoundary') }}
                  </div>

                  <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                    <div class="flex flex-wrap items-start gap-vt-3">
                      <div class="min-w-0">
                        <div class="text-sm font-semibold text-primary">{{ t('creativeResources.rules.validationTitle') }}</div>
                        <p class="mt-vt-1 text-xs leading-5 text-muted">{{ t('creativeResources.rules.validationDesc') }}</p>
                      </div>
                      <n-button class="ml-auto" size="small" type="primary" :loading="isValidatingOutput" :disabled="!sampleOutput.trim()" @click="handleValidateOutput">
                        {{ t('creativeResources.rules.validateOutput') }}
                      </n-button>
                    </div>
                    <div class="mt-vt-3 grid gap-vt-3 lg:grid-cols-2">
                      <label class="grid gap-vt-1 text-xs text-muted">
                        <span>{{ t('creativeResources.rules.expectedCount') }}</span>
                        <n-input-number v-model:value="expectedCount" size="small" clearable :min="0" :precision="0" />
                      </label>
                      <label class="grid gap-vt-1 text-xs text-muted">
                        <span>{{ t('creativeResources.rules.repairAttemptCount') }}</span>
                        <n-input-number v-model:value="repairAttemptCount" size="small" :min="0" :max="2" :precision="0" />
                      </label>
                    </div>
                    <label class="mt-vt-3 grid gap-vt-1 text-xs text-muted">
                      <span>{{ t('creativeResources.rules.sampleOutput') }}</span>
                      <n-input v-model:value="sampleOutput" size="small" type="textarea" :placeholder="t('creativeResources.rules.sampleOutputPlaceholder')" :autosize="{ minRows: 6, maxRows: 14 }" />
                    </label>
                    <div v-if="validationResult" class="mt-vt-3 grid gap-vt-3">
                      <div class="flex flex-wrap items-center gap-vt-2 text-xs">
                        <span class="rounded-vt-sm border px-vt-2 py-1 font-medium" :class="validationResult.valid ? 'border-status-succeeded text-status-succeeded' : 'border-status-failed text-status-failed'">
                          {{ validationResult.valid ? t('creativeResources.rules.validationPassed') : t('creativeResources.rules.validationFailed') }}
                        </span>
                        <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-1 text-muted">
                          {{ validationResult.repairNeeded ? t('creativeResources.rules.repairNeeded') : t('creativeResources.rules.repairNotNeeded') }}
                        </span>
                        <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-1 text-muted">
                          {{ t('creativeResources.rules.attemptInfo', { count: validationResult.attemptCount, max: validationResult.maxAttempts }) }}
                        </span>
                      </div>
                      <div v-if="validationResult.errors.length > 0" class="rounded-vt-sm border border-status-failed bg-card p-vt-3">
                        <div class="mb-vt-2 text-xs font-semibold text-status-failed">{{ t('creativeResources.rules.errors') }}</div>
                        <ul class="grid gap-vt-1 text-xs leading-5 text-secondary">
                          <li v-for="error in validationResult.errors" :key="error" class="break-all">{{ error }}</li>
                        </ul>
                      </div>
                      <div class="grid gap-vt-2">
                        <div class="text-xs font-semibold text-primary">{{ t('creativeResources.rules.parsedJson') }}</div>
                        <pre class="max-h-64 overflow-auto rounded-vt-sm border border-border bg-card p-vt-3 font-mono text-[11px] leading-5 text-secondary">{{ validationParsedJson }}</pre>
                      </div>
                    </div>
                  </div>
                </div>

                <div v-else class="mt-vt-4 rounded-vt-sm border border-border bg-page p-vt-4 text-center text-xs text-muted">
                  {{ t('creativeResources.rules.emptySelect') }}
                </div>
              </div>

              <aside class="rounded-vt-md border border-border bg-card p-vt-4 shadow-vt-md">
                <div class="grid gap-vt-2">
                  <n-select v-model:value="ruleFilterModule" size="small" clearable :placeholder="t('creativeResources.rules.filterModule')" :options="ruleModuleOptions" />
                  <n-select v-model:value="ruleFilterSource" size="small" clearable :placeholder="t('creativeResources.rules.filterSource')" :options="ruleSourceOptions" />
                </div>
                <div class="mt-vt-3 flex items-center gap-vt-2">
                  <h2 class="text-base font-semibold text-primary">{{ t('creativeResources.rules.listTitle') }}</h2>
                  <span class="ml-auto rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-xs text-muted">{{ t('creativeResources.rules.ruleCount', { count: filteredRules.length }) }}</span>
                </div>
                <div class="mt-vt-3 grid gap-vt-2">
                  <button v-for="rule in filteredRules" :key="rule.ruleId" type="button" class="rounded-vt-sm border p-vt-3 text-left text-xs transition" :class="selectedRuleId === rule.ruleId ? 'border-accent-line bg-accent-soft' : 'border-border bg-page hover:border-border-strong'" @click="selectRule(rule)">
                    <div class="flex items-center gap-vt-2">
                      <span class="min-w-0 truncate font-semibold text-primary">{{ rule.name }}</span>
                      <span class="ml-auto rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ rule.sourceType }}</span>
                    </div>
                    <div class="mt-vt-1 truncate text-muted">{{ rule.key }}</div>
                    <div class="mt-vt-2 flex flex-wrap gap-vt-1 text-muted">
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ rule.module }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ rule.ruleType }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ rule.providerKind }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ rule.enabled ? t('creativeResources.rules.enabled') : t('creativeResources.rules.disabled') }}</span>
                    </div>
                  </button>
                  <div v-if="filteredRules.length === 0" class="rounded-vt-sm border border-border bg-page p-vt-4 text-center text-xs text-muted">{{ t('creativeResources.rules.empty') }}</div>
                </div>
              </aside>
            </section>
          </main>
        </div>
      </div>
    </div>

    <n-modal v-model:show="isImportAssetModalOpen" preset="card" :title="t('creativeResources.assets.importTitle')" class="max-w-2xl">
      <div class="grid gap-vt-3">
        <input ref="assetFileInput" class="hidden" type="file" @change="handleAssetFilePicked" />
        <label class="grid gap-vt-1 text-xs text-muted">
          <span>{{ t('creativeResources.assets.fields.sourcePath') }}</span>
          <div class="grid gap-vt-2 sm:grid-cols-[minmax(0,1fr)_auto]">
            <n-input v-model:value="assetImportForm.sourcePath" size="small" :placeholder="t('creativeResources.assets.sourcePathPlaceholder')" @blur="syncAssetImportFromPath" />
            <n-button size="small" @click="assetFileInput?.click()">{{ t('creativeResources.assets.chooseFile') }}</n-button>
          </div>
        </label>
        <div class="grid gap-vt-3 sm:grid-cols-2">
          <label class="grid gap-vt-1 text-xs text-muted">
            <span>{{ t('creativeResources.assets.fields.kind') }}</span>
            <n-select v-model:value="assetImportForm.kind" size="small" :options="assetKindOptions" />
          </label>
          <label class="grid gap-vt-1 text-xs text-muted">
            <span>{{ t('creativeResources.assets.fields.mimeType') }}</span>
            <n-input v-model:value="assetImportForm.mimeType" size="small" :placeholder="t('creativeResources.assets.mimeTypePlaceholder')" />
          </label>
        </div>
        <label class="grid gap-vt-1 text-xs text-muted">
          <span>{{ t('creativeResources.assets.fields.displayName') }}</span>
          <n-input v-model:value="assetImportForm.displayName" size="small" :placeholder="t('creativeResources.assets.displayNamePlaceholder')" />
        </label>
        <label class="grid gap-vt-1 text-xs text-muted">
          <span>{{ t('creativeResources.assets.fields.metadataJson') }}</span>
          <n-input v-model:value="assetImportForm.metadataJson" size="small" type="textarea" :autosize="{ minRows: 4, maxRows: 8 }" />
        </label>
        <div class="rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-muted">
          {{ t('creativeResources.assets.importBoundary') }}
        </div>
        <div class="flex flex-wrap justify-end gap-vt-2">
          <n-button size="small" @click="isImportAssetModalOpen = false">{{ t('common.cancel') }}</n-button>
          <n-button size="small" type="primary" :loading="isImportingAsset" @click="handleImportAsset">{{ t('creativeResources.assets.importSubmit') }}</n-button>
        </div>
      </div>
    </n-modal>
  </section>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import { NButton, NInput, NInputNumber, NModal, NPopconfirm, NSelect, NSwitch, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import {
  cloneVideoPackToUser,
  cloneCreativeRuleToUser,
  deleteAsset,
  deleteAssetReference,
  deleteUserVideoPack,
  deleteUserCreativeRule,
  importAsset,
  listAssetReferences,
  listAssets,
  listCreativeRules,
  listVideoPacks,
  probeMedia,
  readAssetPreview,
  saveUserCreativeRule,
  setVideoPackEnabled,
  setUserCreativeRuleEnabled,
  upsertUserVideoPack,
  validateStructuredOutput,
} from '@/entities/config/api'
import type { AssetDto, AssetReferenceDto, CreativeRuleDto, MediaProbeDto, SaveCreativeRuleRequest, StructuredOutputValidationResult, UpsertUserVideoPackRequest, VideoPackDto } from '@/entities/config/types'
import type { ProviderKind } from '@/shared/enums/generated'

type ResourceEntryKey = 'packs' | 'rules' | 'assets'
type CreativeRuleRefJson = UpsertUserVideoPackRequest['ruleRefs']
type RuleFormState = Omit<SaveCreativeRuleRequest, 'outputSchema'>
type PackFormState = Omit<UpsertUserVideoPackRequest, 'ruleRefs' | 'recommendedExecutableRefs' | 'assetRefs' | 'defaultDurationSeconds' | 'defaultSceneCount'> & {
  defaultDurationSeconds: number | null
  defaultSceneCount: number | null
}
type AssetImportFormState = {
  sourcePath: string
  kind: string
  displayName: string
  mimeType: string
  metadataJson: string
}

const { t } = useI18n()
const message = useMessage()

const resourceEntries = [
  { key: 'packs', icon: '▦' },
  { key: 'rules', icon: '⌁' },
  { key: 'assets', icon: '□' },
] as const

const activeEntry = ref<ResourceEntryKey>('rules')
const packs = ref<VideoPackDto[]>([])
const selectedPackId = ref<string | null>(null)
const packFilterSource = ref<'builtin' | 'user' | null>(null)
const includeDisabledPacks = ref(true)
const ruleRefsJson = ref('{}')
const executableRefsJson = ref('{}')
const assetRefsJson = ref('[]')
const isLoadingPacks = ref(false)
const isCloningPack = ref(false)
const isSavingPack = ref(false)
const isTogglingPack = ref(false)
const isDeletingPack = ref(false)
const rules = ref<CreativeRuleDto[]>([])
const selectedRuleId = ref<string | null>(null)
const ruleFilterModule = ref<string | null>(null)
const ruleFilterSource = ref<'builtin' | 'user' | null>(null)
const outputSchemaJson = ref('{}')
const paramsSchemaJson = ref('{}')
const sampleOutput = ref('')
const expectedCount = ref<number | null>(null)
const repairAttemptCount = ref(0)
const validationResult = ref<StructuredOutputValidationResult | null>(null)
const isLoadingRules = ref(false)
const isCloningRule = ref(false)
const isSavingRule = ref(false)
const isTogglingRule = ref(false)
const isDeletingRule = ref(false)
const isValidatingOutput = ref(false)
const assets = ref<AssetDto[]>([])
const selectedAssetId = ref<string | null>(null)
const selectedAssetReferences = ref<AssetReferenceDto[]>([])
const selectedAssetProbe = ref<MediaProbeDto | null>(null)
const assetProbeError = ref('')
const assetPreviewUrl = ref('')
const assetPreviewError = ref('')
const assetFilterGroup = ref<string | null>(null)
const includeDeletedAssets = ref(false)
const isLoadingAssets = ref(false)
const isImportAssetModalOpen = ref(false)
const isImportingAsset = ref(false)
const isLoadingAssetPreview = ref(false)
const isProbingAsset = ref(false)
const isDeletingAsset = ref(false)
const deletingReferenceId = ref<string | null>(null)
const assetFileInput = ref<HTMLInputElement | null>(null)

const ruleForm = reactive<RuleFormState>({
  key: '',
  name: '',
  module: 'script',
  ruleType: 'script_rule',
  providerKind: 'llm',
  description: '',
  enabled: true,
  body: '',
})

const packForm = reactive<PackFormState>({
  packId: undefined,
  name: '',
  description: '',
  applicableInputTypes: ['topic', 'paste', 'article'],
  contentCategory: '',
  defaultTone: '',
  defaultAspectRatio: '9:16',
  defaultDurationSeconds: 60,
  defaultSceneCount: 8,
  isEnabled: true,
})

const assetImportForm = reactive<AssetImportFormState>({
  sourcePath: '',
  kind: 'user_image',
  displayName: '',
  mimeType: '',
  metadataJson: '{}',
})

const packSourceOptions = computed(() => [
  { label: t('creativeResources.packs.sourceBuiltin'), value: 'builtin' },
  { label: t('creativeResources.packs.sourceUser'), value: 'user' },
])
const packInputTypeOptions = computed(() =>
  ['topic', 'paste', 'article', 'novel', 'material'].map((value) => ({
    label: t(`dict.inputType.${normalizeI18nKey(value)}`),
    value,
  })),
)
const packAspectRatioOptions = computed(() =>
  ['9:16', '16:9', '1:1'].map((value) => ({
    label: value,
    value,
  })),
)

const ruleModuleOptions = computed(() =>
  ['script', 'storyboard', 'character', 'scene', 'style', 'image_prompt', 'storyboard_image', 'video_prompt', 'subtitle', 'cover', 'review'].map((value) => ({
    label: t(`creativeResources.rules.modules.${value}`),
    value,
  })),
)
const ruleTypeOptions = computed(() =>
  ['script_rule', 'storyboard_rule', 'character_rule', 'scene_rule', 'style_rule', 'image_prompt_rule', 'storyboard_image_rule', 'video_prompt_rule', 'subtitle_rule', 'cover_rule', 'review_rule'].map((value) => ({
    label: t(`creativeResources.rules.ruleTypes.${value}`),
    value,
  })),
)
const providerKindOptions = computed(() =>
  (['llm', 'image', 'video', 'tts', 'vlm', 'workflow'] as ProviderKind[]).map((value) => ({
    label: t(`dict.providerKind.${value}`),
    value,
  })),
)
const ruleSourceOptions = computed(() => [
  { label: t('creativeResources.rules.sources.builtin'), value: 'builtin' },
  { label: t('creativeResources.rules.sources.user'), value: 'user' },
])
const assetGroupOptions = computed(() =>
  assetGroups.map((group) => ({
    label: t(`creativeResources.assets.groups.${group.key}`),
    value: group.key,
  })),
)
const assetKindOptions = computed(() =>
  assetImportKinds.map((kind) => ({
    label: t(`creativeResources.assets.kinds.${kind}`),
    value: kind,
  })),
)

const filteredRules = computed(() =>
  rules.value.filter((rule) => {
    if (ruleFilterModule.value && rule.module !== ruleFilterModule.value) return false
    if (ruleFilterSource.value && rule.sourceType !== ruleFilterSource.value) return false
    return true
  }),
)
const filteredPacks = computed(() =>
  packs.value.filter((pack) => {
    if (packFilterSource.value && pack.sourceType !== packFilterSource.value) return false
    return true
  }),
)
const selectedPack = computed(() => packs.value.find((pack) => pack.packId === selectedPackId.value) ?? null)
const canEditSelectedPack = computed(() => selectedPack.value?.sourceType === 'user')
const selectedRule = computed(() => rules.value.find((rule) => rule.ruleId === selectedRuleId.value) ?? null)
const canEditSelectedRule = computed(() => selectedRule.value?.sourceType === 'user')
const selectedRuleReferenceTotal = computed(() => selectedRule.value ? ruleReferenceTotal(selectedRule.value) : 0)
const filteredAssets = computed(() =>
  assets.value.filter((asset) => {
    if (!assetFilterGroup.value) return true
    return assetKindsForGroup(assetFilterGroup.value).includes(asset.kind)
  }),
)
const selectedAsset = computed(() => assets.value.find((asset) => asset.assetId === selectedAssetId.value) ?? null)
const selectedAssetMetadataJson = computed(() => selectedAsset.value ? JSON.stringify(selectedAsset.value.metadata ?? {}, null, 2) : '')
const selectedAssetMediaKind = computed(() => selectedAsset.value ? resolveAssetMediaKind(selectedAsset.value) : 'unknown')
const assetDimensionsText = computed(() => {
  const width = selectedAssetProbe.value?.width ?? readNumberMetadata(selectedAsset.value, 'width')
  const height = selectedAssetProbe.value?.height ?? readNumberMetadata(selectedAsset.value, 'height')
  return width && height ? `${width} × ${height}` : '-'
})
const assetDurationText = computed(() => {
  const seconds = selectedAssetProbe.value?.durationSeconds ?? readNumberMetadata(selectedAsset.value, 'durationSeconds')
  return typeof seconds === 'number' && seconds > 0 ? formatDuration(seconds) : '-'
})
const previewPlaceholder = computed(() => {
  const mediaKind = selectedAssetMediaKind.value
  if (mediaKind === 'audio') return t('creativeResources.assets.audioPreviewPlaceholder')
  if (mediaKind === 'template' || mediaKind === 'font') return t('creativeResources.assets.filePreviewPlaceholder')
  return t('creativeResources.assets.previewUnavailable')
})
const validationParsedJson = computed(() => {
  if (!validationResult.value || validationResult.value.parsedJson === undefined) return ''
  return JSON.stringify(validationResult.value.parsedJson, null, 2)
})

const assetGroups: Array<{ key: string, kinds: string[] }> = [
  { key: 'character', kinds: ['character_reference', 'character_reference_image'] },
  { key: 'style', kinds: ['style_reference', 'style_reference_image'] },
  { key: 'scene', kinds: ['scene_reference', 'scene_reference_image'] },
  { key: 'control', kinds: ['pose_reference', 'depth_reference', 'mask_reference'] },
  { key: 'audio', kinds: ['bgm', 'source_audio', 'generated_audio'] },
  { key: 'template', kinds: ['template_resource', 'font'] },
  { key: 'generated', kinds: ['generated_image_candidate', 'generated_video_segment', 'generated_output', 'task_artifact', 'final_export', 'cover_source'] },
  { key: 'user', kinds: ['user_image', 'user_video', 'source_video', 'source_material', 'reference_image'] },
]

const assetImportKinds = [
  'character_reference_image',
  'style_reference_image',
  'scene_reference_image',
  'pose_reference',
  'depth_reference',
  'mask_reference',
  'source_video',
  'source_audio',
  'bgm',
  'font',
  'template_resource',
  'user_image',
  'user_video',
  'source_material',
] as const

watch(filteredRules, (items) => {
  if (selectedRuleId.value && items.some((rule) => rule.ruleId === selectedRuleId.value)) return
  selectedRuleId.value = items[0]?.ruleId ?? null
  if (items[0]) applyRuleToForm(items[0])
}, { flush: 'post' })

onMounted(loadRules)
onBeforeUnmount(() => {
  clearAssetPreviewUrl()
})

watch(activeEntry, (entry) => {
  if (entry === 'packs' && packs.value.length === 0) {
    void loadVideoPacks()
  }
  if (entry === 'assets' && assets.value.length === 0) {
    void loadAssets()
  }
})

watch(filteredPacks, (items) => {
  if (selectedPackId.value && items.some((pack) => pack.packId === selectedPackId.value)) return
  selectedPackId.value = items[0]?.packId ?? null
  if (items[0]) applyPackToForm(items[0])
}, { flush: 'post' })

watch(filteredAssets, (items) => {
  if (selectedAssetId.value && items.some((asset) => asset.assetId === selectedAssetId.value)) return
  selectedAssetId.value = items[0]?.assetId ?? null
  if (items[0]) {
    void loadAssetReferences(items[0].assetId)
  } else {
    selectedAssetReferences.value = []
    selectedAssetProbe.value = null
    assetProbeError.value = ''
    assetPreviewError.value = ''
    clearAssetPreviewUrl()
  }
}, { flush: 'post' })

async function loadRules() {
  isLoadingRules.value = true
  try {
    rules.value = await listCreativeRules()
    const selected = selectedRuleId.value
      ? rules.value.find((rule) => rule.ruleId === selectedRuleId.value)
      : rules.value[0]
    if (selected) {
      selectRule(selected)
    }
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isLoadingRules.value = false
  }
}

async function loadVideoPacks() {
  isLoadingPacks.value = true
  try {
    packs.value = await listVideoPacks({ includeDisabled: includeDisabledPacks.value })
    const selected = selectedPackId.value
      ? packs.value.find((pack) => pack.packId === selectedPackId.value)
      : filteredPacks.value[0]
    if (selected) {
      selectPack(selected)
    }
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isLoadingPacks.value = false
  }
}

async function loadAssets() {
  isLoadingAssets.value = true
  try {
    assets.value = await listAssets({ includeDeleted: includeDeletedAssets.value })
    const selected = selectedAssetId.value
      ? assets.value.find((asset) => asset.assetId === selectedAssetId.value)
      : filteredAssets.value[0]
    if (selected) {
      await selectAsset(selected)
    } else {
      selectedAssetId.value = null
      selectedAssetReferences.value = []
    }
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isLoadingAssets.value = false
  }
}

function toggleDisabledPacks() {
  includeDisabledPacks.value = !includeDisabledPacks.value
  void loadVideoPacks()
}

function toggleDeletedAssets() {
  includeDeletedAssets.value = !includeDeletedAssets.value
  void loadAssets()
}

function openImportAssetModal() {
  Object.assign(assetImportForm, {
    sourcePath: '',
    kind: 'user_image',
    displayName: '',
    mimeType: '',
    metadataJson: '{}',
  })
  isImportAssetModalOpen.value = true
}

function handleAssetFilePicked(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  assetImportForm.sourcePath = readFilePath(file) || assetImportForm.sourcePath
  assetImportForm.displayName = trimExtension(file.name)
  assetImportForm.mimeType = file.type || guessMimeType(file.name) || assetImportForm.mimeType
  assetImportForm.kind = guessAssetKind(file.name, assetImportForm.mimeType)
  assetImportForm.metadataJson = JSON.stringify({
    displayName: assetImportForm.displayName,
    fileName: file.name,
    sizeBytes: file.size,
    mediaKind: guessMediaKind(file.name, assetImportForm.mimeType),
  }, null, 2)
  input.value = ''
}

function syncAssetImportFromPath() {
  const fileName = fileNameFromPath(assetImportForm.sourcePath)
  if (!fileName) return
  if (!assetImportForm.displayName) {
    assetImportForm.displayName = trimExtension(fileName)
  }
  if (!assetImportForm.mimeType) {
    assetImportForm.mimeType = guessMimeType(fileName) || ''
  }
  assetImportForm.kind = guessAssetKind(fileName, assetImportForm.mimeType)
}

async function handleImportAsset() {
  const sourcePath = assetImportForm.sourcePath.trim()
  if (!sourcePath) {
    message.error(t('creativeResources.assets.sourcePathRequired'))
    return
  }

  isImportingAsset.value = true
  try {
    const imported = await importAsset({
      sourcePath,
      kind: assetImportForm.kind,
      displayName: optionalString(assetImportForm.displayName),
      mimeType: optionalString(assetImportForm.mimeType),
      metadata: parseJsonObject(assetImportForm.metadataJson, t('creativeResources.assets.fields.metadataJson')),
    })
    isImportAssetModalOpen.value = false
    selectedAssetId.value = imported.assetId
    await loadAssets()
    await selectAsset(imported)
    message.success(t('creativeResources.assets.importSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isImportingAsset.value = false
  }
}

async function selectAsset(asset: AssetDto) {
  selectedAssetId.value = asset.assetId
  await Promise.all([
    loadAssetReferences(asset.assetId),
    loadAssetPreview(asset),
    loadAssetProbe(asset),
  ])
}

async function loadAssetReferences(assetId: string) {
  try {
    selectedAssetReferences.value = await listAssetReferences(assetId)
  } catch (error) {
    selectedAssetReferences.value = []
    message.error(errorMessage(error))
  }
}

async function loadAssetPreview(asset: AssetDto) {
  clearAssetPreviewUrl()
  assetPreviewError.value = ''
  if (!['image', 'video'].includes(resolveAssetMediaKind(asset))) return

  isLoadingAssetPreview.value = true
  try {
    const preview = await readAssetPreview({ assetId: asset.assetId })
    if (selectedAssetId.value !== asset.assetId) return
    const bytes = new Uint8Array(preview.bytes)
    const blob = new Blob([bytes], { type: preview.mimeType })
    assetPreviewUrl.value = URL.createObjectURL(blob)
  } catch (error) {
    if (selectedAssetId.value === asset.assetId) {
      assetPreviewError.value = errorMessage(error)
    }
  } finally {
    if (selectedAssetId.value === asset.assetId) {
      isLoadingAssetPreview.value = false
    }
  }
}

async function loadAssetProbe(asset: AssetDto) {
  selectedAssetProbe.value = null
  assetProbeError.value = ''
  const mediaKind = resolveAssetMediaKind(asset)
  if (!['video', 'audio'].includes(mediaKind)) return

  isProbingAsset.value = true
  try {
    const probe = await probeMedia({ relativePath: asset.relativePath, mediaKind })
    if (selectedAssetId.value === asset.assetId) {
      selectedAssetProbe.value = probe
    }
  } catch (error) {
    if (selectedAssetId.value === asset.assetId) {
      assetProbeError.value = errorMessage(error)
    }
  } finally {
    if (selectedAssetId.value === asset.assetId) {
      isProbingAsset.value = false
    }
  }
}

function clearAssetPreviewUrl() {
  if (!assetPreviewUrl.value) return
  URL.revokeObjectURL(assetPreviewUrl.value)
  assetPreviewUrl.value = ''
}

async function handleDeleteReference(reference: AssetReferenceDto) {
  deletingReferenceId.value = reference.referenceId
  try {
    await deleteAssetReference({ referenceId: reference.referenceId })
    if (selectedAsset.value) await loadAssetReferences(selectedAsset.value.assetId)
    message.success(t('creativeResources.assets.unlinkSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    deletingReferenceId.value = null
  }
}

async function handleDeleteAsset() {
  const asset = selectedAsset.value
  if (!asset) return

  isDeletingAsset.value = true
  try {
    await deleteAsset({ assetId: asset.assetId, physical: false })
    await loadAssets()
    message.success(t('creativeResources.assets.deleteSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isDeletingAsset.value = false
  }
}

function selectPack(pack: VideoPackDto) {
  selectedPackId.value = pack.packId
  applyPackToForm(pack)
}

function applyPackToForm(pack: VideoPackDto) {
  Object.assign(packForm, {
    packId: pack.packId,
    name: pack.name,
    description: pack.description,
    applicableInputTypes: [...pack.applicableInputTypes],
    contentCategory: pack.contentCategory ?? '',
    defaultTone: pack.defaultTone ?? '',
    defaultAspectRatio: pack.defaultAspectRatio,
    defaultDurationSeconds: pack.defaultDurationSeconds,
    defaultSceneCount: pack.defaultSceneCount,
    isEnabled: pack.isEnabled,
  })
  ruleRefsJson.value = JSON.stringify(pack.ruleRefs ?? {}, null, 2)
  executableRefsJson.value = JSON.stringify(pack.recommendedExecutableRefs ?? {}, null, 2)
  assetRefsJson.value = JSON.stringify(pack.assetRefs ?? [], null, 2)
}

async function handleClonePack() {
  const pack = selectedPack.value
  if (!pack) return

  isCloningPack.value = true
  try {
    const cloned = await cloneVideoPackToUser({ packId: pack.packId })
    await loadVideoPacks()
    selectPack(cloned)
    message.success(t('creativeResources.packs.cloneSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isCloningPack.value = false
  }
}

async function handleSavePack() {
  const pack = selectedPack.value
  if (!pack || pack.sourceType !== 'user') return

  isSavingPack.value = true
  try {
    const saved = await upsertUserVideoPack({
      ...packForm,
      packId: pack.packId,
      contentCategory: optionalString(packForm.contentCategory),
      defaultTone: optionalString(packForm.defaultTone),
      defaultDurationSeconds: packForm.defaultDurationSeconds ?? 60,
      defaultSceneCount: packForm.defaultSceneCount ?? 8,
      ruleRefs: parseRuleRefsJson(ruleRefsJson.value, t('creativeResources.packs.fields.ruleRefs')),
      recommendedExecutableRefs: parseJsonObject(executableRefsJson.value, t('creativeResources.packs.fields.executableRefs')),
      assetRefs: parseJsonArray(assetRefsJson.value, t('creativeResources.packs.fields.assetRefs')),
    })
    await loadVideoPacks()
    selectPack(saved)
    message.success(t('creativeResources.packs.saveSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isSavingPack.value = false
  }
}

async function handleTogglePack() {
  const pack = selectedPack.value
  if (!pack || pack.sourceType !== 'user') return

  isTogglingPack.value = true
  try {
    const updated = await setVideoPackEnabled({
      packId: pack.packId,
      isEnabled: !pack.isEnabled,
    })
    await loadVideoPacks()
    selectPack(updated)
    message.success(t('creativeResources.packs.toggleSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isTogglingPack.value = false
  }
}

async function handleDeletePack() {
  const pack = selectedPack.value
  if (!pack || pack.sourceType !== 'user') return

  isDeletingPack.value = true
  try {
    await deleteUserVideoPack({ packId: pack.packId })
    selectedPackId.value = null
    await loadVideoPacks()
    message.success(t('creativeResources.packs.deleteSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isDeletingPack.value = false
  }
}

function selectRule(rule: CreativeRuleDto) {
  selectedRuleId.value = rule.ruleId
  applyRuleToForm(rule)
}

function applyRuleToForm(rule: CreativeRuleDto) {
  Object.assign(ruleForm, {
    key: rule.key,
    name: rule.name,
    module: rule.module,
    ruleType: rule.ruleType,
    providerKind: rule.providerKind,
    description: rule.description,
    enabled: rule.enabled,
    body: rule.body,
  })
  outputSchemaJson.value = JSON.stringify(rule.outputSchema ?? {}, null, 2)
  paramsSchemaJson.value = JSON.stringify(rule.paramsSchema ?? {}, null, 2)
  validationResult.value = null
}

async function handleCloneRule() {
  const rule = selectedRule.value
  if (!rule) return

  isCloningRule.value = true
  try {
    const cloned = await cloneCreativeRuleToUser({ ruleId: rule.ruleId })
    await loadRules()
    selectRule(cloned)
    message.success(t('creativeResources.rules.cloneSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isCloningRule.value = false
  }
}

async function handleSaveRule() {
  const rule = selectedRule.value
  if (!rule || rule.sourceType !== 'user') return

  isSavingRule.value = true
  try {
    const saved = await saveUserCreativeRule({
      ...ruleForm,
      outputSchema: parseOutputSchema(),
      paramsSchema: parseParamsSchema(),
    })
    await loadRules()
    selectRule(saved)
    message.success(t('creativeResources.rules.saveSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isSavingRule.value = false
  }
}

async function handleToggleRule() {
  const rule = selectedRule.value
  if (!rule || rule.sourceType !== 'user') return

  isTogglingRule.value = true
  try {
    const updated = await setUserCreativeRuleEnabled({
      ruleId: rule.ruleId,
      enabled: !rule.enabled,
    })
    await loadRules()
    selectRule(updated)
    message.success(t('creativeResources.rules.toggleSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isTogglingRule.value = false
  }
}

async function handleDeleteRule() {
  const rule = selectedRule.value
  if (!rule || rule.sourceType !== 'user') return

  isDeletingRule.value = true
  try {
    await deleteUserCreativeRule({ ruleId: rule.ruleId })
    selectedRuleId.value = null
    await loadRules()
    message.success(t('creativeResources.rules.deleteSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isDeletingRule.value = false
  }
}

async function handleValidateOutput() {
  const rule = selectedRule.value
  if (!rule) return

  isValidatingOutput.value = true
  try {
    validationResult.value = await validateStructuredOutput({
      rawOutput: sampleOutput.value,
      outputSchema: parseOutputSchema(),
      expectedCount: expectedCount.value ?? undefined,
      repairAttemptCount: repairAttemptCount.value,
      maxRepairAttempts: 2,
    })
    if (validationResult.value.valid) {
      message.success(t('creativeResources.rules.validationPassed'))
    } else {
      message.warning(t('creativeResources.rules.validationFailed'))
    }
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isValidatingOutput.value = false
  }
}

function parseOutputSchema(): Record<string, unknown> {
  try {
    const parsed = JSON.parse(outputSchemaJson.value || '{}')
    if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) return parsed as Record<string, unknown>
  } catch {
    throw new Error(t('creativeResources.rules.invalidOutputSchema'))
  }
  throw new Error(t('creativeResources.rules.invalidOutputSchema'))
}

function parseParamsSchema(): Record<string, unknown> {
  try {
    const parsed = JSON.parse(paramsSchemaJson.value || '{}')
    if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) return parsed as Record<string, unknown>
  } catch {
    throw new Error(t('creativeResources.rules.invalidParamsSchema'))
  }
  throw new Error(t('creativeResources.rules.invalidParamsSchema'))
}

function parseJsonObject(raw: string, field: string): Record<string, unknown> {
  try {
    const parsed = JSON.parse(raw || '{}')
    if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) return parsed as Record<string, unknown>
  } catch {
    throw new Error(t('creativeResources.packs.invalidJson', { field }))
  }
  throw new Error(t('creativeResources.packs.invalidJson', { field }))
}

function parseRuleRefsJson(raw: string, field: string): CreativeRuleRefJson {
  const parsed = parseJsonObject(raw, field)
  for (const [slot, value] of Object.entries(parsed)) {
    if (!value || typeof value !== 'object' || Array.isArray(value)) {
      throw new Error(t('creativeResources.packs.invalidRuleRefs', { field, slot }))
    }
    const record = value as Record<string, unknown>
    for (const key of ['ruleKey', 'ruleId', 'sourceType', 'ruleType']) {
      if (typeof record[key] !== 'string' || !record[key]) {
        throw new Error(t('creativeResources.packs.invalidRuleRefs', { field, slot }))
      }
    }
  }
  return parsed as CreativeRuleRefJson
}

function parseJsonArray(raw: string, field: string): unknown[] {
  try {
    const parsed = JSON.parse(raw || '[]')
    if (Array.isArray(parsed)) return parsed
  } catch {
    throw new Error(t('creativeResources.packs.invalidJson', { field }))
  }
  throw new Error(t('creativeResources.packs.invalidJson', { field }))
}

function optionalString(value: string | undefined) {
  const trimmed = value?.trim()
  return trimmed ? trimmed : undefined
}

function ruleReferenceTotal(rule: CreativeRuleDto) {
  const counts = rule.referenceCounts
  return counts.videoPacks + counts.projects + counts.taskSteps + counts.generationContexts
}

function assetDisplayName(asset: AssetDto) {
  const displayName = asset.metadata?.displayName
  if (typeof displayName === 'string' && displayName.trim()) return displayName
  return asset.relativePath.split('/').at(-1) || asset.assetId
}

function assetGroupLabel(asset: AssetDto) {
  const group = assetGroups.find((item) => item.kinds.includes(asset.kind))
  return group ? t(`creativeResources.assets.groups.${group.key}`) : t('creativeResources.assets.groups.other')
}

function assetKindsForGroup(groupKey: string) {
  return assetGroups.find((group) => group.key === groupKey)?.kinds ?? []
}

function resolveAssetMediaKind(asset: AssetDto) {
  const metadataKind = asset.metadata?.mediaKind
  if (typeof metadataKind === 'string' && metadataKind.trim()) return metadataKind
  if (asset.mimeType?.startsWith('image/')) return 'image'
  if (asset.mimeType?.startsWith('video/')) return 'video'
  if (asset.mimeType?.startsWith('audio/')) return 'audio'
  return guessMediaKind(asset.relativePath, asset.mimeType)
}

function readNumberMetadata(asset: AssetDto | null, key: string) {
  const value = asset?.metadata?.[key]
  return typeof value === 'number' && Number.isFinite(value) ? value : undefined
}

function probeSummary(probe: MediaProbeDto) {
  const parts = [
    probe.mediaKind,
    probe.width && probe.height ? `${probe.width}×${probe.height}` : undefined,
    probe.durationSeconds > 0 ? formatDuration(probe.durationSeconds) : undefined,
    probe.fps ? `${probe.fps.toFixed(2)} fps` : undefined,
    probe.videoCodec,
    probe.audioCodec,
  ].filter(Boolean)
  return parts.join(' · ')
}

function formatBytes(size: number) {
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  return `${(size / 1024 / 1024).toFixed(1)} MB`
}

function formatDuration(seconds: number) {
  const rounded = Math.max(0, Math.round(seconds))
  const minutes = Math.floor(rounded / 60)
  const rest = rounded % 60
  if (minutes === 0) return `${rest}s`
  return `${minutes}m ${rest}s`
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

function fileExtension(fileName: string) {
  const index = fileName.lastIndexOf('.')
  return index >= 0 ? fileName.slice(index + 1).toLowerCase() : ''
}

function guessAssetKind(fileName: string, mimeType?: string) {
  const mediaKind = guessMediaKind(fileName, mimeType)
  if (mediaKind === 'video') return 'source_video'
  if (mediaKind === 'audio') return 'source_audio'
  if (mediaKind === 'font') return 'font'
  if (mediaKind === 'template') return 'template_resource'
  return 'user_image'
}

function guessMediaKind(fileName: string, mimeType?: string) {
  if (mimeType?.startsWith('image/')) return 'image'
  if (mimeType?.startsWith('video/')) return 'video'
  if (mimeType?.startsWith('audio/')) return 'audio'
  if (mimeType?.startsWith('font/')) return 'font'
  const extension = fileExtension(fileName)
  if (['jpg', 'jpeg', 'png', 'webp', 'gif', 'bmp'].includes(extension)) return 'image'
  if (['mp4', 'mov', 'webm', 'mkv', 'avi'].includes(extension)) return 'video'
  if (['mp3', 'wav', 'm4a', 'aac', 'flac', 'ogg'].includes(extension)) return 'audio'
  if (['ttf', 'otf', 'woff', 'woff2'].includes(extension)) return 'font'
  if (['html', 'json', 'css', 'srt', 'vtt'].includes(extension)) return 'template'
  return 'unknown'
}

function guessMimeType(fileName: string) {
  const map: Record<string, string> = {
    jpg: 'image/jpeg',
    jpeg: 'image/jpeg',
    png: 'image/png',
    webp: 'image/webp',
    gif: 'image/gif',
    bmp: 'image/bmp',
    mp4: 'video/mp4',
    mov: 'video/quicktime',
    webm: 'video/webm',
    mkv: 'video/x-matroska',
    mp3: 'audio/mpeg',
    wav: 'audio/wav',
    m4a: 'audio/mp4',
    aac: 'audio/aac',
    flac: 'audio/flac',
    ogg: 'audio/ogg',
    ttf: 'font/ttf',
    otf: 'font/otf',
    woff: 'font/woff',
    woff2: 'font/woff2',
    html: 'text/html',
    json: 'application/json',
    css: 'text/css',
    srt: 'application/x-subrip',
    vtt: 'text/vtt',
  }
  return map[fileExtension(fileName)]
}

function normalizeI18nKey(value: string) {
  return value.replaceAll('-', '_')
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error)
}
</script>
