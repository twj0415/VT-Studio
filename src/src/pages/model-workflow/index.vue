<template>
  <section class="view">
    <div class="gpage">
      <div class="wrap flex min-h-0 flex-1 flex-col">
        <div class="phead">
          <div>
            <h1>{{ t('modelWorkflow.title') }}</h1>
            <div class="desc">{{ t('modelWorkflow.desc') }}</div>
          </div>
          <span class="rounded-vt-sm border border-accent-line bg-accent-soft px-vt-3 py-vt-2 text-xs font-medium text-accent">{{ t('modelWorkflow.stageBadge') }}</span>
        </div>

        <div class="grid min-h-0 gap-vt-4 xl:grid-cols-[280px_minmax(0,1fr)]">
          <aside class="rounded-vt-md border border-border bg-card p-vt-3 shadow-vt-md">
            <button v-for="entry in layerEntries" :key="entry.key" type="button" class="mb-vt-2 flex w-full gap-vt-3 rounded-vt-sm border px-vt-3 py-vt-3 text-left transition" :class="activeLayer === entry.key ? 'border-accent-line bg-accent-soft text-accent' : 'border-border bg-page text-secondary hover:border-border-strong hover:text-primary'" @click="activeLayer = entry.key">
              <span class="grid size-7 flex-none place-items-center rounded-vt-sm border border-current text-[11px] font-semibold">{{ entry.index }}</span>
              <span class="min-w-0">
                <span class="block font-medium">{{ t(`modelWorkflow.layers.${entry.key}.title`) }}</span>
                <span class="mt-vt-1 block text-xs leading-5 text-muted">{{ t(`modelWorkflow.layers.${entry.key}.short`) }}</span>
              </span>
            </button>

            <div class="mt-vt-4 rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-muted">
              <div class="font-semibold text-primary">{{ t('modelWorkflow.boundaryTitle') }}</div>
              <p class="mt-vt-2">{{ t('modelWorkflow.boundary.provider') }}</p>
              <p class="mt-vt-2">{{ t('modelWorkflow.boundary.security') }}</p>
            </div>
          </aside>

          <main class="min-w-0">
            <section v-if="activeLayer === 'provider'" class="grid gap-vt-4 2xl:grid-cols-[minmax(0,1fr)_390px]">
              <div class="rounded-vt-md border border-border bg-card p-vt-4 shadow-vt-md">
                <div class="flex flex-wrap items-center gap-vt-3">
                  <div class="min-w-0">
                    <h2 class="text-base font-semibold text-primary">{{ t('modelWorkflow.provider.title') }}</h2>
                    <p class="mt-vt-1 text-sm leading-6 text-secondary">{{ t('modelWorkflow.provider.desc') }}</p>
                  </div>
                  <n-button class="ml-auto" size="small" @click="resetProviderForm">{{ t('modelWorkflow.provider.newProvider') }}</n-button>
                </div>

                <div class="mt-vt-4 grid gap-vt-3 lg:grid-cols-2">
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.provider.fields.providerId') }}</span>
                    <n-input v-model:value="providerForm.providerId" size="small" :disabled="isEditingProvider" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.provider.fields.displayName') }}</span>
                    <n-input v-model:value="providerForm.displayName" size="small" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.provider.fields.kind') }}</span>
                    <n-select v-model:value="providerForm.providerKind" size="small" :options="providerKindOptions" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.provider.fields.vendor') }}</span>
                    <n-input v-model:value="providerForm.vendor" size="small" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.provider.fields.authType') }}</span>
                    <n-select v-model:value="providerForm.authType" size="small" :options="authTypeOptions" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.provider.fields.keyAlias') }}</span>
                    <n-input v-model:value="providerForm.keyAlias" size="small" :disabled="providerForm.authType === 'none'" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                    <span>{{ t('modelWorkflow.provider.fields.baseUrl') }}</span>
                    <n-input v-model:value="providerForm.baseUrl" size="small" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.provider.fields.status') }}</span>
                    <n-select v-model:value="providerForm.status" size="small" :options="providerStatusOptions" />
                  </label>
                  <label class="flex items-center gap-vt-2 pt-vt-5 text-xs text-secondary">
                    <n-switch v-model:value="providerForm.isEnabled" size="small" />
                    <span>{{ t('modelWorkflow.provider.fields.enabled') }}</span>
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                    <span>{{ t('modelWorkflow.provider.fields.configJson') }}</span>
                    <n-input v-model:value="providerConfigJson" size="small" type="textarea" :autosize="{ minRows: 4, maxRows: 8 }" />
                  </label>
                </div>

                <div class="mt-vt-4 flex flex-wrap items-center gap-vt-2">
                  <n-button type="primary" size="small" :loading="isSavingProvider" @click="handleSaveProvider">{{ t('modelWorkflow.provider.save') }}</n-button>
                  <n-button size="small" :loading="isTesting" :disabled="!selectedProviderId" @click="handleDryRun(false)">{{ t('modelWorkflow.dryRun.test') }}</n-button>
                  <n-button size="small" :loading="isTestingFailure" :disabled="!selectedProviderId" @click="handleDryRun(true)">{{ t('modelWorkflow.dryRun.testFailure') }}</n-button>
                  <n-button size="small" :loading="isTestingCancel" :disabled="!selectedProviderId" @click="handleCancelledDryRun">{{ t('modelWorkflow.dryRun.testCancel') }}</n-button>
                  <n-popconfirm :positive-text="t('modelWorkflow.dryRun.confirmRealGenerate')" :negative-text="t('common.cancel')" @positive-click="handleProviderRealGenerate">
                    <template #trigger>
                      <n-button size="small" :loading="isRealGenerateTesting" :disabled="!selectedProviderId">{{ t('modelWorkflow.dryRun.realGenerate') }}</n-button>
                    </template>
                    {{ t(providerForm.providerKind === 'video' ? 'modelWorkflow.dryRun.videoRealGenerateConfirm' : 'modelWorkflow.dryRun.realGenerateConfirm') }}
                  </n-popconfirm>
                  <n-popconfirm :positive-text="t('common.confirm')" :negative-text="t('common.cancel')" @positive-click="handleDeleteProvider">
                    <template #trigger>
                      <n-button size="small" :disabled="!selectedProviderId" :loading="isDeletingProvider">{{ t('modelWorkflow.provider.deleteProvider') }}</n-button>
                    </template>
                    {{ t('modelWorkflow.provider.deleteConfirm') }}
                  </n-popconfirm>
                </div>

                <div v-if="providerForm.authType !== 'none'" class="mt-vt-4 rounded-vt-sm border border-border bg-page p-vt-3">
                  <div class="flex flex-wrap items-center gap-vt-2 text-xs">
                    <div class="font-semibold text-primary">{{ t('modelWorkflow.secret.title') }}</div>
                    <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-1 text-muted">{{ currentSecretStatus }}</span>
                    <span class="ml-auto text-muted">{{ t('modelWorkflow.secret.noEcho') }}</span>
                  </div>
                  <div class="mt-vt-3 grid gap-vt-2 lg:grid-cols-[1fr_auto_auto]">
                    <n-input v-model:value="secretInput" size="small" type="password" show-password-on="click" :placeholder="t('modelWorkflow.secret.placeholder')" />
                    <n-button size="small" :loading="isSavingSecret" :disabled="!providerForm.providerId || !providerForm.keyAlias || !secretInput" @click="handleSaveSecret">{{ t('modelWorkflow.secret.save') }}</n-button>
                    <n-popconfirm :positive-text="t('common.confirm')" :negative-text="t('common.cancel')" @positive-click="handleDeleteSecret">
                      <template #trigger>
                        <n-button size="small" :loading="isDeletingSecret" :disabled="!providerForm.keyAlias">{{ t('modelWorkflow.secret.delete') }}</n-button>
                      </template>
                      {{ t('modelWorkflow.secret.deleteConfirm') }}
                    </n-popconfirm>
                  </div>
                </div>

                <div v-if="dryRunResult || dryRunError" class="mt-vt-4 rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5">
                  <template v-if="dryRunResult">
                    <div class="font-semibold text-primary">{{ t('modelWorkflow.dryRun.success') }}</div>
                    <div class="mt-vt-1 text-muted">{{ t('modelWorkflow.dryRun.mode') }}: {{ dryRunResult.testMode }}</div>
                    <div class="mt-vt-1 text-muted">{{ dryRunResult.message }}</div>
                    <div class="mt-vt-1 text-muted">{{ t('modelWorkflow.dryRun.billable') }}: {{ dryRunResult.billable ? t('modelWorkflow.dryRun.billableYes') : t('modelWorkflow.dryRun.billableNo') }}</div>
                    <div class="mt-vt-2 break-all font-mono text-muted">{{ dryRunResult.traceId }}</div>
                    <pre class="mt-vt-2 max-h-48 overflow-auto rounded-vt-sm border border-border bg-card p-vt-2 text-muted">{{ JSON.stringify(dryRunResult.outputSummary, null, 2) }}</pre>
                  </template>
                  <template v-else>
                    <div class="font-semibold text-status-failed">{{ t('modelWorkflow.dryRun.failed') }}</div>
                    <div class="mt-vt-1 break-all text-muted">{{ dryRunError }}</div>
                  </template>
                </div>
              </div>

              <aside class="rounded-vt-md border border-border bg-card p-vt-4 shadow-vt-md">
                <div class="flex items-center gap-vt-2">
                  <h2 class="text-base font-semibold text-primary">{{ t('modelWorkflow.provider.listTitle') }}</h2>
                  <span class="ml-auto rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-xs text-muted">{{ t('modelWorkflow.dryRun.providerCount', { count: providerConfigs.length }) }}</span>
                </div>
                <div class="mt-vt-3 grid gap-vt-2">
                  <button v-for="provider in providerConfigs" :key="provider.providerId" type="button" class="rounded-vt-sm border p-vt-3 text-left text-xs transition" :class="selectedProviderId === provider.providerId ? 'border-accent-line bg-accent-soft' : 'border-border bg-page hover:border-border-strong'" @click="selectProvider(provider)">
                    <div class="flex items-center gap-vt-2">
                      <span class="font-semibold text-primary">{{ provider.displayName }}</span>
                      <span class="ml-auto rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ provider.providerKind }}</span>
                    </div>
                    <div class="mt-vt-1 truncate text-muted">{{ provider.providerId }}</div>
                    <div class="mt-vt-2 flex flex-wrap gap-vt-1 text-muted">
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ provider.vendor }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ provider.authType }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ provider.status }}</span>
                    </div>
                  </button>
                  <div v-if="providerConfigs.length === 0" class="rounded-vt-sm border border-border bg-page p-vt-4 text-center text-xs text-muted">{{ t('modelWorkflow.provider.empty') }}</div>
                </div>
              </aside>
            </section>

            <section v-else-if="activeLayer === 'models'" class="grid gap-vt-4 2xl:grid-cols-[minmax(0,1fr)_390px]">
              <div class="rounded-vt-md border border-border bg-card p-vt-4 shadow-vt-md">
                <div class="flex flex-wrap items-center gap-vt-3">
                  <div class="min-w-0">
                    <h2 class="text-base font-semibold text-primary">{{ t('modelWorkflow.models.title') }}</h2>
                    <p class="mt-vt-1 text-sm leading-6 text-secondary">{{ t('modelWorkflow.models.desc') }}</p>
                  </div>
                  <n-button class="ml-auto" size="small" @click="resetModelForm">{{ t('modelWorkflow.models.newModel') }}</n-button>
                </div>

                <div class="mt-vt-4 grid gap-vt-3 lg:grid-cols-2">
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.models.fields.modelId') }}</span>
                    <n-input v-model:value="modelForm.modelId" size="small" :disabled="isEditingModel" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.models.fields.provider') }}</span>
                    <n-select v-model:value="modelForm.providerId" size="small" :options="modelProviderOptions" :disabled="isEditingModel" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.models.fields.displayName') }}</span>
                    <n-input v-model:value="modelForm.displayName" size="small" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.models.fields.modelName') }}</span>
                    <n-input v-model:value="modelForm.modelName" size="small" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.models.fields.providerModelId') }}</span>
                    <n-input v-model:value="modelForm.providerModelId" size="small" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.models.fields.status') }}</span>
                    <n-select v-model:value="modelForm.status" size="small" :options="modelStatusOptions" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                    <span>{{ t('modelWorkflow.models.fields.abilityTypes') }}</span>
                    <n-select v-model:value="modelForm.abilityTypes" size="small" multiple :options="abilityTypeOptions" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.models.fields.inputModalities') }}</span>
                    <n-select v-model:value="modelForm.inputModalities" size="small" multiple :options="modalityOptions" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.models.fields.outputModalities') }}</span>
                    <n-select v-model:value="modelForm.outputModalities" size="small" multiple :options="modalityOptions" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                    <span>{{ t('modelWorkflow.models.fields.featureFlags') }}</span>
                    <n-select v-model:value="modelForm.featureFlags" size="small" multiple filterable tag :options="featureFlagOptions" />
                  </label>
                  <label class="flex items-center gap-vt-2 pt-vt-5 text-xs text-secondary">
                    <n-switch v-model:value="modelForm.isEnabled" size="small" />
                    <span>{{ t('modelWorkflow.models.fields.enabled') }}</span>
                  </label>
                  <label class="flex items-center gap-vt-2 pt-vt-5 text-xs text-secondary">
                    <n-switch v-model:value="modelForm.apiContractVerified" size="small" />
                    <span>{{ t('modelWorkflow.models.fields.verified') }}</span>
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                    <span>{{ t('modelWorkflow.models.fields.limitsJson') }}</span>
                    <n-input v-model:value="modelLimitsJson" size="small" type="textarea" :autosize="{ minRows: 5, maxRows: 10 }" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                    <span>{{ t('modelWorkflow.models.fields.inputRequirementsJson') }}</span>
                    <n-input v-model:value="modelInputRequirementsJson" size="small" type="textarea" :autosize="{ minRows: 3, maxRows: 8 }" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                    <span>{{ t('modelWorkflow.models.fields.configJson') }}</span>
                    <n-input v-model:value="modelConfigJson" size="small" type="textarea" :autosize="{ minRows: 3, maxRows: 8 }" />
                  </label>
                </div>

                <div class="mt-vt-4 flex flex-wrap items-center gap-vt-2">
                  <n-button type="primary" size="small" :loading="isSavingModel" :disabled="modelProviderOptions.length === 0" @click="handleSaveModel">{{ t('modelWorkflow.models.save') }}</n-button>
                  <n-button size="small" :loading="isTestingModel" :disabled="!selectedModelId" @click="handleModelDryRun">{{ t('modelWorkflow.models.test') }}</n-button>
                  <n-popconfirm :positive-text="t('modelWorkflow.dryRun.confirmRealGenerate')" :negative-text="t('common.cancel')" @positive-click="handleModelRealGenerate">
                    <template #trigger>
                      <n-button size="small" :loading="isRealGenerateTesting" :disabled="!selectedModelId">{{ t('modelWorkflow.dryRun.realGenerate') }}</n-button>
                    </template>
                    {{ t(selectedModel?.providerKind === 'video' ? 'modelWorkflow.dryRun.videoRealGenerateConfirm' : 'modelWorkflow.dryRun.realGenerateConfirm') }}
                  </n-popconfirm>
                  <n-popconfirm :positive-text="t('common.confirm')" :negative-text="t('common.cancel')" @positive-click="handleDeleteModel">
                    <template #trigger>
                      <n-button size="small" :disabled="!selectedModelId" :loading="isDeletingModel">{{ t('modelWorkflow.models.deleteModel') }}</n-button>
                    </template>
                    {{ t('modelWorkflow.models.deleteConfirm') }}
                  </n-popconfirm>
                </div>

                <div class="mt-vt-4 rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-muted">
                  {{ t('modelWorkflow.models.boundary') }}
                </div>
              </div>

              <aside class="rounded-vt-md border border-border bg-card p-vt-4 shadow-vt-md">
                <div class="flex items-center gap-vt-2">
                  <h2 class="text-base font-semibold text-primary">{{ t('modelWorkflow.models.listTitle') }}</h2>
                  <span class="ml-auto rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-xs text-muted">{{ t('modelWorkflow.models.modelCount', { count: providerModels.length }) }}</span>
                </div>
                <div class="mt-vt-3 grid gap-vt-2">
                  <button v-for="model in providerModels" :key="model.modelId" type="button" class="rounded-vt-sm border p-vt-3 text-left text-xs transition" :class="selectedModelId === model.modelId ? 'border-accent-line bg-accent-soft' : 'border-border bg-page hover:border-border-strong'" @click="selectModel(model)">
                    <div class="flex items-center gap-vt-2">
                      <span class="font-semibold text-primary">{{ model.displayName }}</span>
                      <span class="ml-auto rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ model.providerKind }}</span>
                    </div>
                    <div class="mt-vt-1 truncate text-muted">{{ model.modelId }}</div>
                    <div class="mt-vt-2 flex flex-wrap gap-vt-1 text-muted">
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ model.modelName }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ model.status }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ model.isEnabled ? t('modelWorkflow.models.enabled') : t('modelWorkflow.models.disabled') }}</span>
                    </div>
                    <div class="mt-vt-2 truncate text-muted">{{ model.abilityTypes.join(', ') }}</div>
                  </button>
                  <div v-if="providerModels.length === 0" class="rounded-vt-sm border border-border bg-page p-vt-4 text-center text-xs text-muted">{{ t('modelWorkflow.models.empty') }}</div>
                </div>
              </aside>
            </section>

            <section v-else-if="activeLayer === 'workflow'" class="grid gap-vt-4 2xl:grid-cols-[minmax(0,1fr)_390px]">
              <div class="rounded-vt-md border border-border bg-card p-vt-4 shadow-vt-md">
                <div class="flex flex-wrap items-center gap-vt-3">
                  <div class="min-w-0">
                    <h2 class="text-base font-semibold text-primary">{{ t('modelWorkflow.workflow.title') }}</h2>
                    <p class="mt-vt-1 text-sm leading-6 text-secondary">{{ t('modelWorkflow.workflow.desc') }}</p>
                  </div>
                  <n-button class="ml-auto" size="small" @click="resetWorkflowForm">{{ t('modelWorkflow.workflow.newPreset') }}</n-button>
                </div>

                <div class="mt-vt-4 grid gap-vt-3 lg:grid-cols-2">
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.workflow.fields.workflowPresetId') }}</span>
                    <n-input v-model:value="workflowForm.workflowPresetId" size="small" :disabled="isEditingWorkflow" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.workflow.fields.provider') }}</span>
                    <n-select v-model:value="workflowForm.providerId" size="small" :options="workflowProviderOptions" :disabled="isEditingWorkflow" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.workflow.fields.displayName') }}</span>
                    <n-input v-model:value="workflowForm.displayName" size="small" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.workflow.fields.vendor') }}</span>
                    <n-select v-model:value="workflowForm.vendor" size="small" :options="workflowVendorOptions" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.workflow.fields.workflowKey') }}</span>
                    <n-input v-model:value="workflowForm.workflowKey" size="small" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.workflow.fields.workflowId') }}</span>
                    <n-input v-model:value="workflowForm.workflowId" size="small" :disabled="workflowForm.vendor !== 'runninghub'" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.workflow.fields.workflowVersion') }}</span>
                    <n-input v-model:value="workflowForm.workflowVersion" size="small" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.workflow.fields.status') }}</span>
                    <n-select v-model:value="workflowForm.status" size="small" :options="workflowStatusOptions" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                    <span>{{ t('modelWorkflow.workflow.fields.abilityTypes') }}</span>
                    <n-select v-model:value="workflowForm.abilityTypes" size="small" multiple :options="workflowAbilityOptions" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.workflow.fields.inputModalities') }}</span>
                    <n-select v-model:value="workflowForm.inputModalities" size="small" multiple :options="modalityOptions" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted">
                    <span>{{ t('modelWorkflow.workflow.fields.outputModalities') }}</span>
                    <n-select v-model:value="workflowForm.outputModalities" size="small" multiple :options="workflowOutputModalityOptions" />
                  </label>
                  <label class="flex items-center gap-vt-2 pt-vt-5 text-xs text-secondary">
                    <n-switch v-model:value="workflowForm.isEnabled" size="small" />
                    <span>{{ t('modelWorkflow.workflow.fields.enabled') }}</span>
                  </label>
                  <label class="flex items-center gap-vt-2 pt-vt-5 text-xs text-secondary">
                    <n-switch v-model:value="workflowForm.isBuiltin" size="small" />
                    <span>{{ t('modelWorkflow.workflow.fields.builtin') }}</span>
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                    <span>{{ t('modelWorkflow.workflow.fields.paramSchemaJson') }}</span>
                    <n-input v-model:value="workflowParamSchemaJson" size="small" type="textarea" :autosize="{ minRows: 4, maxRows: 8 }" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                    <span>{{ t('modelWorkflow.workflow.fields.nodeMapJson') }}</span>
                    <n-input v-model:value="workflowNodeMapJson" size="small" type="textarea" :autosize="{ minRows: 4, maxRows: 8 }" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                    <span>{{ t('modelWorkflow.workflow.fields.outputMapJson') }}</span>
                    <n-input v-model:value="workflowOutputMapJson" size="small" type="textarea" :autosize="{ minRows: 3, maxRows: 8 }" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                    <span>{{ t('modelWorkflow.workflow.fields.defaultParamsJson') }}</span>
                    <n-input v-model:value="workflowDefaultParamsJson" size="small" type="textarea" :autosize="{ minRows: 3, maxRows: 8 }" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                    <span>{{ t('modelWorkflow.workflow.fields.limitsJson') }}</span>
                    <n-input v-model:value="workflowLimitsJson" size="small" type="textarea" :autosize="{ minRows: 3, maxRows: 8 }" />
                  </label>
                  <label class="grid gap-vt-1 text-xs text-muted lg:col-span-2">
                    <span>{{ t('modelWorkflow.workflow.fields.configJson') }}</span>
                    <n-input v-model:value="workflowConfigJson" size="small" type="textarea" :autosize="{ minRows: 3, maxRows: 8 }" />
                  </label>
                </div>

                <div class="mt-vt-4 flex flex-wrap items-center gap-vt-2">
                  <n-button type="primary" size="small" :loading="isSavingWorkflow" :disabled="workflowProviderOptions.length === 0" @click="handleSaveWorkflow">{{ t('modelWorkflow.workflow.save') }}</n-button>
                  <n-button size="small" :loading="isTestingWorkflow" :disabled="!selectedWorkflowPresetId" @click="handleWorkflowDryRun">{{ t('modelWorkflow.workflow.test') }}</n-button>
                  <n-popconfirm :positive-text="t('modelWorkflow.dryRun.confirmRealGenerate')" :negative-text="t('common.cancel')" @positive-click="handleWorkflowRealGenerate">
                    <template #trigger>
                      <n-button size="small" :loading="isRealGenerateTesting" :disabled="!selectedWorkflowPresetId">{{ t('modelWorkflow.dryRun.realGenerate') }}</n-button>
                    </template>
                    {{ t(selectedWorkflowIsVideo ? 'modelWorkflow.dryRun.videoRealGenerateConfirm' : 'modelWorkflow.dryRun.realGenerateConfirm') }}
                  </n-popconfirm>
                  <n-popconfirm :positive-text="t('common.confirm')" :negative-text="t('common.cancel')" @positive-click="handleDeleteWorkflow">
                    <template #trigger>
                      <n-button size="small" :disabled="!selectedWorkflowPresetId" :loading="isDeletingWorkflow">{{ t('modelWorkflow.workflow.deletePreset') }}</n-button>
                    </template>
                    {{ t('modelWorkflow.workflow.deleteConfirm') }}
                  </n-popconfirm>
                </div>

                <div class="mt-vt-4 rounded-vt-sm border border-border bg-page p-vt-3 text-xs leading-5 text-muted">
                  {{ t('modelWorkflow.workflow.boundary') }}
                </div>
              </div>

              <aside class="rounded-vt-md border border-border bg-card p-vt-4 shadow-vt-md">
                <div class="flex items-center gap-vt-2">
                  <h2 class="text-base font-semibold text-primary">{{ t('modelWorkflow.workflow.listTitle') }}</h2>
                  <span class="ml-auto rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-xs text-muted">{{ t('modelWorkflow.workflow.presetCount', { count: workflowPresets.length }) }}</span>
                </div>
                <div class="mt-vt-3 grid gap-vt-2">
                  <button v-for="preset in workflowPresets" :key="preset.workflowPresetId" type="button" class="rounded-vt-sm border p-vt-3 text-left text-xs transition" :class="selectedWorkflowPresetId === preset.workflowPresetId ? 'border-accent-line bg-accent-soft' : 'border-border bg-page hover:border-border-strong'" @click="selectWorkflow(preset)">
                    <div class="flex items-center gap-vt-2">
                      <span class="font-semibold text-primary">{{ preset.displayName }}</span>
                      <span class="ml-auto rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ preset.vendor }}</span>
                    </div>
                    <div class="mt-vt-1 truncate text-muted">{{ preset.workflowPresetId }}</div>
                    <div class="mt-vt-2 flex flex-wrap gap-vt-1 text-muted">
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ preset.workflowVersion }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ preset.status }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ preset.isEnabled ? t('modelWorkflow.workflow.enabled') : t('modelWorkflow.workflow.disabled') }}</span>
                    </div>
                    <div class="mt-vt-2 truncate text-muted">{{ preset.abilityTypes.join(', ') }}</div>
                  </button>
                  <div v-if="workflowPresets.length === 0" class="rounded-vt-sm border border-border bg-page p-vt-4 text-center text-xs text-muted">{{ t('modelWorkflow.workflow.empty') }}</div>
                </div>
              </aside>
            </section>

            <section v-else class="grid gap-vt-4 2xl:grid-cols-[minmax(0,1fr)_430px]">
              <div class="rounded-vt-md border border-border bg-card p-vt-4 shadow-vt-md">
                <div class="flex flex-wrap items-center gap-vt-3">
                  <div class="min-w-0">
                    <h2 class="text-base font-semibold text-primary">{{ t('modelWorkflow.options.title') }}</h2>
                    <p class="mt-vt-1 text-sm leading-6 text-secondary">{{ t('modelWorkflow.options.desc') }}</p>
                  </div>
                  <n-button class="ml-auto" size="small" :loading="isLoadingExecutableOptions" @click="loadExecutableOptions">{{ t('modelWorkflow.options.refresh') }}</n-button>
                </div>

                <div v-if="selectedExecutableOption" class="mt-vt-4 grid gap-vt-3">
                  <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                    <div class="flex flex-wrap items-center gap-vt-2 text-xs">
                      <span class="font-semibold text-primary">{{ selectedExecutableOption.label }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ selectedExecutableOption.sourceType }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ selectedExecutableOption.capability }}</span>
                      <span class="rounded-vt-sm border px-vt-2 py-0.5" :class="selectedExecutableOption.enabled ? 'border-status-succeeded/40 bg-status-succeeded/10 text-status-succeeded' : 'border-status-failed/40 bg-status-failed/10 text-status-failed'">
                        {{ selectedExecutableOption.enabled ? t('modelWorkflow.options.executable') : t('modelWorkflow.options.notExecutable') }}
                      </span>
                    </div>
                    <div class="mt-vt-2 break-all text-xs text-muted">{{ selectedExecutableOption.sourceId }}</div>
                    <div v-if="selectedExecutableOption.disabledReason" class="mt-vt-2 text-xs text-status-failed">{{ selectedExecutableOption.disabledReason }}</div>
                  </div>

                  <div class="grid gap-vt-2 sm:grid-cols-3">
                    <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                      <div class="text-xs text-muted">{{ t('modelWorkflow.options.required') }}</div>
                      <div class="mt-vt-1 text-lg font-semibold text-primary">{{ selectedExecutableOption.inputPlan.requiredCount }}</div>
                    </div>
                    <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                      <div class="text-xs text-muted">{{ t('modelWorkflow.options.optional') }}</div>
                      <div class="mt-vt-1 text-lg font-semibold text-primary">{{ selectedExecutableOption.inputPlan.optionalCount }}</div>
                    </div>
                    <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                      <div class="text-xs text-muted">{{ t('modelWorkflow.options.unused') }}</div>
                      <div class="mt-vt-1 text-lg font-semibold text-primary">{{ selectedExecutableOption.inputPlan.unusedCount }}</div>
                    </div>
                  </div>

                  <div class="rounded-vt-sm border border-border bg-page p-vt-3">
                    <div class="flex flex-wrap items-center gap-vt-2 text-xs">
                      <span class="font-semibold text-primary">{{ t('modelWorkflow.options.inputPlan') }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ selectedExecutableOption.inputPlan.planKind }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ selectedExecutableOption.inputPlan.abilityType }}</span>
                    </div>
                    <div class="mt-vt-3 grid gap-vt-2">
                      <div v-for="item in selectedExecutableOption.inputPlan.items" :key="item.inputKey" class="rounded-vt-sm border border-border bg-card p-vt-2 text-xs">
                        <div class="flex flex-wrap items-center gap-vt-2">
                          <span class="font-medium text-primary">{{ item.inputKey }}</span>
                          <span class="rounded-vt-sm border border-border bg-page px-vt-2 py-0.5 text-muted">{{ item.inputGroup }}</span>
                          <span class="rounded-vt-sm border px-vt-2 py-0.5" :class="inputRequirementClass(item.requirement)">{{ item.requirement }}</span>
                        </div>
                        <div v-if="item.missingReason" class="mt-vt-1 text-muted">{{ item.missingReason }}</div>
                        <div class="mt-vt-1 truncate text-muted">{{ item.sourceOptions.join(' / ') }}</div>
                      </div>
                    </div>
                  </div>

                  <div class="flex flex-wrap items-center gap-vt-2">
                    <n-button size="small" type="primary" :disabled="!selectedExecutableOption.enabled" :loading="isTestingExecutableOption" @click="handleExecutableOptionDryRun">{{ t('modelWorkflow.options.test') }}</n-button>
                    <n-popconfirm :positive-text="t('modelWorkflow.dryRun.confirmRealGenerate')" :negative-text="t('common.cancel')" @positive-click="handleExecutableOptionRealGenerate">
                      <template #trigger>
                        <n-button size="small" :disabled="!selectedExecutableOption.enabled" :loading="isRealGenerateTesting">{{ t('modelWorkflow.dryRun.realGenerate') }}</n-button>
                      </template>
                      {{ t(selectedExecutableOptionIsVideo ? 'modelWorkflow.dryRun.videoRealGenerateConfirm' : 'modelWorkflow.dryRun.realGenerateConfirm') }}
                    </n-popconfirm>
                  </div>
                </div>

                <div v-else class="mt-vt-4 rounded-vt-sm border border-border bg-page p-vt-4 text-center text-xs text-muted">
                  {{ t('modelWorkflow.options.selectEmpty') }}
                </div>
              </div>

              <aside class="rounded-vt-md border border-border bg-card p-vt-4 shadow-vt-md">
                <div class="flex items-center gap-vt-2">
                  <h2 class="text-base font-semibold text-primary">{{ t('modelWorkflow.options.listTitle') }}</h2>
                  <span class="ml-auto rounded-vt-sm border border-border bg-page px-vt-2 py-1 text-xs text-muted">{{ t('modelWorkflow.options.optionCount', { count: executableOptions.length }) }}</span>
                </div>
                <div class="mt-vt-3 grid gap-vt-2">
                  <button v-for="option in executableOptions" :key="option.optionId" type="button" class="rounded-vt-sm border p-vt-3 text-left text-xs transition" :class="selectedExecutableOptionId === option.optionId ? 'border-accent-line bg-accent-soft' : 'border-border bg-page hover:border-border-strong'" @click="selectExecutableOption(option)">
                    <div class="flex items-center gap-vt-2">
                      <span class="min-w-0 truncate font-semibold text-primary">{{ option.label }}</span>
                      <span class="ml-auto rounded-vt-sm border border-border bg-card px-vt-2 py-0.5 text-muted">{{ option.providerKind }}</span>
                    </div>
                    <div class="mt-vt-1 truncate text-muted">{{ option.sourceId }}</div>
                    <div class="mt-vt-2 flex flex-wrap gap-vt-1 text-muted">
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ option.sourceType }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ option.vendor }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ option.status }}</span>
                      <span class="rounded-vt-sm border border-border bg-card px-vt-2 py-0.5">{{ option.enabled ? t('modelWorkflow.options.executable') : t('modelWorkflow.options.notExecutable') }}</span>
                    </div>
                    <div v-if="option.disabledReason" class="mt-vt-2 truncate text-status-failed">{{ option.disabledReason }}</div>
                  </button>
                  <div v-if="executableOptions.length === 0" class="rounded-vt-sm border border-border bg-page p-vt-4 text-center text-xs text-muted">{{ t('modelWorkflow.options.empty') }}</div>
                </div>
              </aside>
            </section>
          </main>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { NButton, NInput, NPopconfirm, NSelect, NSwitch, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { deleteProviderConfig, deleteProviderModel, deleteProviderSecret, deleteWorkflowPreset, hasProviderSecret, listExecutableMediaOptions, listProviderConfigs, listProviderModels, listWorkflowPresets, providerGenerationTest, saveProviderSecret, upsertProviderConfig, upsertProviderModel, upsertWorkflowPreset } from '@/entities/config/api'
import type { ExecutableMediaOptionDto, ProviderConfigDto, ProviderGenerationTestMode, ProviderGenerationTestRequest, ProviderGenerationTestResponse, ProviderModelDto, WorkflowPresetDto } from '@/entities/config/types'
import type { ModelCapability, ProviderAuthType, ProviderKind } from '@/shared/enums/generated'

type ModelWorkflowLayer = 'provider' | 'models' | 'workflow' | 'options'
type ProviderFormState = Omit<ProviderConfigDto, 'providerKind' | 'authType' | 'config'> & {
  providerKind: ProviderKind
  authType: ProviderAuthType | 'oauth'
}
type ApiProviderKind = Exclude<ProviderKind, 'workflow'>
type ModelFormState = Omit<ProviderModelDto, 'providerKind' | 'abilityTypes' | 'limits' | 'inputRequirements' | 'config'> & {
  providerKind: ApiProviderKind
  abilityTypes: string[]
}
type WorkflowVendor = WorkflowPresetDto['vendor']
type WorkflowFormState = Omit<WorkflowPresetDto, 'limits' | 'paramSchema' | 'nodeMap' | 'outputMap' | 'defaultParams' | 'config'>

const { t } = useI18n()
const message = useMessage()

const layerEntries = [
  { key: 'provider', index: '01' },
  { key: 'models', index: '02' },
  { key: 'workflow', index: '03' },
  { key: 'options', index: '04' },
] as const

const activeLayer = ref<ModelWorkflowLayer>('provider')
const providerConfigs = ref<ProviderConfigDto[]>([])
const providerModels = ref<ProviderModelDto[]>([])
const workflowPresets = ref<WorkflowPresetDto[]>([])
const executableOptions = ref<ExecutableMediaOptionDto[]>([])
const selectedProviderId = ref<string | null>(null)
const selectedModelId = ref<string | null>(null)
const selectedWorkflowPresetId = ref<string | null>(null)
const selectedExecutableOptionId = ref<string | null>(null)
const providerConfigJson = ref('{}')
const modelLimitsJson = ref(defaultModelLimitsJson())
const modelInputRequirementsJson = ref('{}')
const modelConfigJson = ref('{}')
const workflowLimitsJson = ref(defaultWorkflowLimitsJson())
const workflowParamSchemaJson = ref(defaultWorkflowParamSchemaJson())
const workflowNodeMapJson = ref(defaultWorkflowNodeMapJson())
const workflowOutputMapJson = ref(defaultWorkflowOutputMapJson())
const workflowDefaultParamsJson = ref('{}')
const workflowConfigJson = ref('{}')
const secretInput = ref('')
const secretStatusByAlias = ref<Record<string, boolean>>({})
const isSavingProvider = ref(false)
const isDeletingProvider = ref(false)
const isSavingSecret = ref(false)
const isDeletingSecret = ref(false)
const isTesting = ref(false)
const isTestingFailure = ref(false)
const isTestingCancel = ref(false)
const isRealGenerateTesting = ref(false)
const isSavingModel = ref(false)
const isDeletingModel = ref(false)
const isTestingModel = ref(false)
const isSavingWorkflow = ref(false)
const isDeletingWorkflow = ref(false)
const isTestingWorkflow = ref(false)
const isLoadingExecutableOptions = ref(false)
const isTestingExecutableOption = ref(false)
const dryRunResult = ref<ProviderGenerationTestResponse | null>(null)
const dryRunError = ref<string | null>(null)

const providerForm = reactive<ProviderFormState>(createDefaultProviderForm())
const modelForm = reactive<ModelFormState>(createDefaultModelForm())
const workflowForm = reactive<WorkflowFormState>(createDefaultWorkflowForm())

const providerKindOptions = computed(() =>
  (['llm', 'tts', 'image', 'video', 'vlm', 'workflow'] as ProviderKind[]).map((value) => ({
    label: t(`dict.providerKind.${value}`),
    value,
  }))
)
const authTypeOptions = computed(() =>
  (['none', 'api_key', 'bearer_token', 'basic', 'custom_header'] as const).map((value) => ({
    label: t(`dict.providerAuthType.${value}`),
    value,
  }))
)
const providerStatusOptions = computed(() =>
  ['unconfigured', 'ready', 'testing', 'failed', 'disabled'].map((value) => ({
    label: t(`modelWorkflow.provider.status.${value}`),
    value,
  }))
)
const modelStatusOptions = computed(() =>
  ['unconfigured', 'ready', 'testing', 'failed', 'disabled'].map((value) => ({
    label: t(`modelWorkflow.models.status.${value}`),
    value,
  }))
)
const modelProviderOptions = computed(() =>
  providerConfigs.value
    .filter((provider): provider is ProviderConfigDto & { providerKind: ApiProviderKind } => provider.providerKind !== 'workflow')
    .map((provider) => ({
      label: `${provider.displayName} · ${provider.providerKind}`,
      value: provider.providerId,
    }))
)
const abilityTypeOptions = computed(() =>
  ([
    'text_generation',
    'structured_output',
    'text_to_image',
    'image_to_image',
    'text_to_video',
    'image_to_video',
    'first_frame_i2v',
    'start_end_frame_i2v',
    'reference_to_video',
    'video_continuation',
    'video_editing',
    'action_transfer',
    'digital_human',
    'native_audio',
    'voice_reference',
    'multi_shot',
    'text_to_speech',
    'vision_analysis',
  ] as string[]).map((value) => ({
    label: t(`modelWorkflow.models.abilities.${value}`),
    value,
  }))
)
const modalityOptions = computed(() =>
  ['text', 'image', 'audio', 'video'].map((value) => ({
    label: t(`modelWorkflow.models.modalities.${value}`),
    value,
  }))
)
const featureFlagOptions = computed(() =>
  ['aspect_ratio', 'resolution', 'duration', 'fps', 'reference_image', 'structured_output', 'json_schema'].map((value) => ({
    label: t(`modelWorkflow.models.featureFlags.${value}`),
    value,
  }))
)
const workflowProviderOptions = computed(() =>
  providerConfigs.value
    .filter((provider) => provider.providerKind === 'workflow' && (provider.vendor === 'comfyui' || provider.vendor === 'runninghub'))
    .map((provider) => ({
      label: `${provider.displayName} · ${provider.vendor}`,
      value: provider.providerId,
    }))
)
const workflowVendorOptions = computed(() =>
  (['comfyui', 'runninghub'] as WorkflowVendor[]).map((value) => ({
    label: value === 'comfyui' ? 'ComfyUI' : 'RunningHub',
    value,
  }))
)
const workflowStatusOptions = computed(() =>
  ['unconfigured', 'ready', 'testing', 'failed', 'disabled'].map((value) => ({
    label: t(`modelWorkflow.workflow.status.${value}`),
    value,
  }))
)
const workflowAbilityOptions = computed(() =>
  [
    'text_to_image',
    'image_to_image',
    'text_to_video',
    'image_to_video',
    'first_frame_i2v',
    'start_end_frame_i2v',
    'reference_to_video',
    'video_continuation',
    'video_editing',
    'action_transfer',
    'digital_human',
    'native_audio',
    'voice_reference',
    'multi_shot',
    'text_to_speech',
    'vision_analysis',
    'workflow_execution',
  ].map((value) => ({
    label: t(`modelWorkflow.models.abilities.${value}`),
    value,
  }))
)
const workflowOutputModalityOptions = computed(() =>
  ['image', 'audio', 'video', 'metadata'].map((value) => ({
    label: t(`modelWorkflow.workflow.modalities.${value}`),
    value,
  }))
)
const selectedProvider = computed(() => providerConfigs.value.find((provider) => provider.providerId === selectedProviderId.value) ?? null)
const isEditingProvider = computed(() => Boolean(selectedProvider.value))
const selectedModel = computed(() => providerModels.value.find((model) => model.modelId === selectedModelId.value) ?? null)
const isEditingModel = computed(() => Boolean(selectedModel.value))
const selectedWorkflow = computed(() => workflowPresets.value.find((preset) => preset.workflowPresetId === selectedWorkflowPresetId.value) ?? null)
const isEditingWorkflow = computed(() => Boolean(selectedWorkflow.value))
const selectedExecutableOption = computed(() => executableOptions.value.find((option) => option.optionId === selectedExecutableOptionId.value) ?? null)
const selectedWorkflowIsVideo = computed(() => selectedWorkflow.value ? workflowPresetIsVideo(selectedWorkflow.value) : false)
const selectedExecutableOptionIsVideo = computed(() => selectedExecutableOption.value ? executableOptionIsVideo(selectedExecutableOption.value) : false)
const currentSecretStatus = computed(() => {
  const keyAlias = providerForm.keyAlias
  if (!keyAlias) return t('modelWorkflow.secret.noAlias')
  return secretStatusByAlias.value[keyAlias] ? t('modelWorkflow.secret.exists') : t('modelWorkflow.secret.missing')
})

watch(
  () => providerForm.authType,
  (authType) => {
    if (authType === 'none') {
      providerForm.keyAlias = undefined
      secretInput.value = ''
    } else if (!providerForm.keyAlias && providerForm.providerId) {
      providerForm.keyAlias = `${providerForm.providerId}:${authType}`
    }
  }
)

onMounted(loadProviders)

async function loadProviders() {
  providerConfigs.value = await listProviderConfigs()
  if (selectedProviderId.value) {
    const selected = providerConfigs.value.find((provider) => provider.providerId === selectedProviderId.value)
    if (selected) {
      applyProviderToForm(selected)
    } else {
      resetProviderForm()
    }
  }
  await refreshSecretStatuses()
  await loadModels()
  await loadWorkflows()
  await loadExecutableOptions()
}

async function loadModels() {
  providerModels.value = await listProviderModels()
  if (selectedModelId.value) {
    const selected = providerModels.value.find((model) => model.modelId === selectedModelId.value)
    if (selected) {
      applyModelToForm(selected)
    } else {
      resetModelForm()
    }
  } else if (!modelForm.providerId) {
    resetModelForm()
  }
}

async function loadWorkflows() {
  workflowPresets.value = await listWorkflowPresets()
  if (selectedWorkflowPresetId.value) {
    const selected = workflowPresets.value.find((preset) => preset.workflowPresetId === selectedWorkflowPresetId.value)
    if (selected) {
      applyWorkflowToForm(selected)
    } else {
      resetWorkflowForm()
    }
  } else if (!workflowForm.providerId) {
    resetWorkflowForm()
  }
}

async function loadExecutableOptions() {
  isLoadingExecutableOptions.value = true
  try {
    executableOptions.value = await listExecutableMediaOptions()
    if (selectedExecutableOptionId.value && !executableOptions.value.some((option) => option.optionId === selectedExecutableOptionId.value)) {
      selectedExecutableOptionId.value = null
    }
    if (!selectedExecutableOptionId.value && executableOptions.value.length > 0) {
      selectedExecutableOptionId.value = executableOptions.value[0].optionId
    }
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isLoadingExecutableOptions.value = false
  }
}

function resetProviderForm() {
  Object.assign(providerForm, createDefaultProviderForm())
  selectedProviderId.value = null
  providerConfigJson.value = '{}'
  secretInput.value = ''
  dryRunResult.value = null
  dryRunError.value = null
}

function selectProvider(provider: ProviderConfigDto) {
  selectedProviderId.value = provider.providerId
  applyProviderToForm(provider)
  secretInput.value = ''
  dryRunResult.value = null
  dryRunError.value = null
}

function resetModelForm() {
  Object.assign(modelForm, createDefaultModelForm())
  selectedModelId.value = null
  modelLimitsJson.value = defaultModelLimitsJson()
  modelInputRequirementsJson.value = '{}'
  modelConfigJson.value = '{}'
}

function selectModel(model: ProviderModelDto) {
  selectedModelId.value = model.modelId
  applyModelToForm(model)
}

function resetWorkflowForm() {
  Object.assign(workflowForm, createDefaultWorkflowForm())
  selectedWorkflowPresetId.value = null
  workflowLimitsJson.value = defaultWorkflowLimitsJson()
  workflowParamSchemaJson.value = defaultWorkflowParamSchemaJson()
  workflowNodeMapJson.value = defaultWorkflowNodeMapJson()
  workflowOutputMapJson.value = defaultWorkflowOutputMapJson()
  workflowDefaultParamsJson.value = '{}'
  workflowConfigJson.value = '{}'
}

function selectWorkflow(preset: WorkflowPresetDto) {
  selectedWorkflowPresetId.value = preset.workflowPresetId
  applyWorkflowToForm(preset)
}

function selectExecutableOption(option: ExecutableMediaOptionDto) {
  selectedExecutableOptionId.value = option.optionId
}

async function handleSaveProvider() {
  isSavingProvider.value = true
  try {
    const provider = normalizeProviderForm()
    const saved = await upsertProviderConfig(provider)
    await loadProviders()
    selectedProviderId.value = saved.providerId
    applyProviderToForm(saved)
    await loadExecutableOptions()
    message.success(t('modelWorkflow.provider.saveSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isSavingProvider.value = false
  }
}

async function handleDeleteProvider() {
  if (!selectedProviderId.value) return

  isDeletingProvider.value = true
  try {
    await deleteProviderConfig({ providerId: selectedProviderId.value })
    resetProviderForm()
    await loadProviders()
    await loadExecutableOptions()
    message.success(t('modelWorkflow.provider.deleteSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isDeletingProvider.value = false
  }
}

async function handleSaveModel() {
  isSavingModel.value = true
  try {
    const model = normalizeModelForm()
    const saved = await upsertProviderModel(model)
    await loadModels()
    selectedModelId.value = saved.modelId
    applyModelToForm(saved)
    await loadExecutableOptions()
    message.success(t('modelWorkflow.models.saveSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isSavingModel.value = false
  }
}

async function handleDeleteModel() {
  if (!selectedModelId.value) return

  isDeletingModel.value = true
  try {
    await deleteProviderModel({ modelId: selectedModelId.value })
    resetModelForm()
    await loadModels()
    await loadExecutableOptions()
    message.success(t('modelWorkflow.models.deleteSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isDeletingModel.value = false
  }
}

async function handleSaveWorkflow() {
  isSavingWorkflow.value = true
  try {
    const preset = normalizeWorkflowForm()
    const saved = await upsertWorkflowPreset(preset)
    await loadWorkflows()
    selectedWorkflowPresetId.value = saved.workflowPresetId
    applyWorkflowToForm(saved)
    await loadExecutableOptions()
    message.success(t('modelWorkflow.workflow.saveSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isSavingWorkflow.value = false
  }
}

async function handleDeleteWorkflow() {
  if (!selectedWorkflowPresetId.value) return

  isDeletingWorkflow.value = true
  try {
    await deleteWorkflowPreset({ workflowPresetId: selectedWorkflowPresetId.value })
    resetWorkflowForm()
    await loadWorkflows()
    await loadExecutableOptions()
    message.success(t('modelWorkflow.workflow.deleteSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isDeletingWorkflow.value = false
  }
}

async function handleSaveSecret() {
  if (!providerForm.providerId || !providerForm.keyAlias) return

  isSavingSecret.value = true
  try {
    const handle = await saveProviderSecret({
      providerId: providerForm.providerId,
      authType: providerForm.authType,
      keyAlias: providerForm.keyAlias,
      secret: secretInput.value,
    })
    secretStatusByAlias.value = { ...secretStatusByAlias.value, [handle.keyAlias]: handle.hasSecret }
    secretInput.value = ''
    message.success(t('modelWorkflow.secret.saveSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isSavingSecret.value = false
  }
}

async function handleDeleteSecret() {
  if (!providerForm.keyAlias) return

  isDeletingSecret.value = true
  try {
    const status = await deleteProviderSecret(providerForm.keyAlias)
    secretStatusByAlias.value = { ...secretStatusByAlias.value, [status.keyAlias]: status.hasSecret }
    message.success(t('modelWorkflow.secret.deleteSuccess'))
  } catch (error) {
    message.error(errorMessage(error))
  } finally {
    isDeletingSecret.value = false
  }
}

async function handleDryRun(simulateFailure: boolean) {
  if (!selectedProviderId.value) return

  const loadingRef = simulateFailure ? isTestingFailure : isTesting
  await runProviderTest(
    {
      providerId: selectedProviderId.value,
      providerKind: providerForm.providerKind,
      workflowPresetId: providerForm.providerKind === 'workflow' ? 'dummy_workflow' : undefined,
      testMode: 'dry_run',
      simulateFailure,
    },
    loadingRef,
  )
}

async function handleCancelledDryRun() {
  if (!selectedProviderId.value) return

  await runProviderTest(
    {
      providerId: selectedProviderId.value,
      providerKind: providerForm.providerKind,
      workflowPresetId: providerForm.providerKind === 'workflow' ? 'dummy_workflow' : undefined,
      testMode: 'dry_run',
      simulateCancelled: true,
    },
    isTestingCancel,
  )
}

async function handleProviderRealGenerate() {
  if (!selectedProviderId.value) return

  await runProviderTest(
    createRealGenerateRequest({
      providerId: selectedProviderId.value,
      providerKind: providerForm.providerKind,
      workflowPresetId: providerForm.providerKind === 'workflow' ? 'dummy_workflow' : undefined,
    }),
    isRealGenerateTesting,
  )
}

async function handleModelDryRun() {
  const model = selectedModel.value
  if (!model) return

  await runProviderTest(
    {
      providerId: model.providerId,
      providerKind: model.providerKind,
      providerModelId: model.modelId,
      testMode: 'dry_run',
    },
    isTestingModel,
  )
}

async function handleModelRealGenerate() {
  const model = selectedModel.value
  if (!model) return

  await runProviderTest(
    createRealGenerateRequest({
      providerId: model.providerId,
      providerKind: model.providerKind,
      providerModelId: model.modelId,
    }),
    isRealGenerateTesting,
  )
}

async function handleWorkflowDryRun() {
  const preset = selectedWorkflow.value
  if (!preset) return

  await runProviderTest(
    {
      providerId: preset.providerId,
      providerKind: 'workflow',
      workflowPresetId: preset.workflowPresetId,
      testMode: 'dry_run',
    },
    isTestingWorkflow,
  )
}

async function handleWorkflowRealGenerate() {
  const preset = selectedWorkflow.value
  if (!preset) return

  await runProviderTest(
    createRealGenerateRequest({
      providerId: preset.providerId,
      providerKind: 'workflow',
      workflowPresetId: preset.workflowPresetId,
    }),
    isRealGenerateTesting,
  )
}

async function handleExecutableOptionDryRun() {
  const option = selectedExecutableOption.value
  if (!option || !option.enabled) return

  await runProviderTest(
    {
      providerId: option.providerId,
      providerKind: option.providerKind,
      providerModelId: option.sourceType === 'provider_model' ? option.providerModelId : undefined,
      workflowPresetId: option.sourceType === 'workflow_preset' ? option.workflowPresetId : undefined,
      testMode: 'dry_run',
    },
    isTestingExecutableOption,
  )
}

async function handleExecutableOptionRealGenerate() {
  const option = selectedExecutableOption.value
  if (!option || !option.enabled) return

  await runProviderTest(
    createRealGenerateRequest({
      providerId: option.providerId,
      providerKind: option.providerKind,
      providerModelId: option.sourceType === 'provider_model' ? option.providerModelId : undefined,
      workflowPresetId: option.sourceType === 'workflow_preset' ? option.workflowPresetId : undefined,
    }),
    isRealGenerateTesting,
  )
}

function createRealGenerateRequest(
  base: Omit<ProviderGenerationTestRequest, 'testMode' | 'realGenerateConfirmed' | 'confirmToken'>,
): ProviderGenerationTestRequest {
  return {
    ...base,
    testMode: 'real_generate',
    realGenerateConfirmed: true,
    confirmToken: requestNeedsVideoConfirm(base) ? 'REAL_GENERATE_VIDEO' : undefined,
  }
}

function requestNeedsVideoConfirm(base: Omit<ProviderGenerationTestRequest, 'testMode' | 'realGenerateConfirmed' | 'confirmToken'>) {
  if (base.providerKind === 'video') return true
  if (base.providerKind !== 'workflow' || !base.workflowPresetId) return false
  const preset = workflowPresets.value.find((item) => item.workflowPresetId === base.workflowPresetId)
  return preset ? workflowPresetIsVideo(preset) : false
}

function workflowPresetIsVideo(preset: WorkflowPresetDto) {
  return [...preset.abilityTypes, ...preset.outputModalities].some(isVideoCapability)
}

function executableOptionIsVideo(option: ExecutableMediaOptionDto) {
  return option.providerKind === 'video' || [option.capability, ...option.capabilities].some(isVideoCapability)
}

function isVideoCapability(value: string) {
  const normalized = value.toLowerCase()
  return normalized.includes('video') || normalized.includes('i2v')
}

async function runProviderTest(
  request: ProviderGenerationTestRequest & { testMode: ProviderGenerationTestMode },
  loadingRef: { value: boolean },
) {
  loadingRef.value = true
  dryRunResult.value = null
  dryRunError.value = null
  try {
    dryRunResult.value = await providerGenerationTest(request)
    message.success(t('modelWorkflow.dryRun.success'))
  } catch (error) {
    dryRunError.value = errorMessage(error)
    message.error(t('modelWorkflow.dryRun.failed'))
  } finally {
    loadingRef.value = false
  }
}

async function refreshSecretStatuses() {
  const aliases = [...new Set(providerConfigs.value.map((provider) => provider.keyAlias).filter((alias): alias is string => Boolean(alias)))]
  const statuses = await Promise.all(aliases.map((keyAlias) => hasProviderSecret(keyAlias).catch(() => ({ keyAlias, hasSecret: false }))))
  secretStatusByAlias.value = Object.fromEntries(statuses.map((status) => [status.keyAlias, status.hasSecret]))
}

function applyProviderToForm(provider: ProviderConfigDto) {
  Object.assign(providerForm, {
    providerId: provider.providerId,
    providerKind: provider.providerKind,
    vendor: provider.vendor,
    displayName: provider.displayName,
    baseUrl: provider.baseUrl ?? '',
    authType: provider.authType,
    keyAlias: provider.keyAlias,
    status: provider.status,
    isEnabled: provider.isEnabled,
  })
  providerConfigJson.value = JSON.stringify(provider.config ?? {}, null, 2)
}

function applyModelToForm(model: ProviderModelDto) {
  Object.assign(modelForm, {
    modelId: model.modelId,
    providerId: model.providerId,
    providerKind: model.providerKind,
    vendor: model.vendor,
    providerModelId: model.providerModelId,
    modelName: model.modelName,
    displayName: model.displayName,
    abilityTypes: [...model.abilityTypes],
    inputModalities: [...model.inputModalities],
    outputModalities: [...model.outputModalities],
    featureFlags: [...model.featureFlags],
    apiContractVerified: model.apiContractVerified,
    status: model.status,
    isEnabled: model.isEnabled,
  })
  modelLimitsJson.value = JSON.stringify(model.limits ?? {}, null, 2)
  modelInputRequirementsJson.value = JSON.stringify(model.inputRequirements ?? {}, null, 2)
  modelConfigJson.value = JSON.stringify(model.config ?? {}, null, 2)
}

function applyWorkflowToForm(preset: WorkflowPresetDto) {
  Object.assign(workflowForm, {
    workflowPresetId: preset.workflowPresetId,
    providerId: preset.providerId,
    vendor: preset.vendor,
    workflowKey: preset.workflowKey,
    workflowId: preset.workflowId,
    displayName: preset.displayName,
    workflowVersion: preset.workflowVersion,
    abilityTypes: [...preset.abilityTypes],
    inputModalities: [...preset.inputModalities],
    outputModalities: [...preset.outputModalities],
    status: preset.status,
    isBuiltin: preset.isBuiltin,
    isEnabled: preset.isEnabled,
  })
  workflowLimitsJson.value = JSON.stringify(preset.limits ?? {}, null, 2)
  workflowParamSchemaJson.value = JSON.stringify(preset.paramSchema ?? {}, null, 2)
  workflowNodeMapJson.value = JSON.stringify(preset.nodeMap ?? {}, null, 2)
  workflowOutputMapJson.value = JSON.stringify(preset.outputMap ?? {}, null, 2)
  workflowDefaultParamsJson.value = JSON.stringify(preset.defaultParams ?? {}, null, 2)
  workflowConfigJson.value = JSON.stringify(preset.config ?? {}, null, 2)
}

function normalizeProviderForm(): ProviderConfigDto {
  const config = parseConfigJson()
  return {
    providerId: providerForm.providerId.trim(),
    providerKind: providerForm.providerKind,
    vendor: String(providerForm.vendor).trim(),
    displayName: providerForm.displayName.trim(),
    baseUrl: providerForm.baseUrl?.trim() || undefined,
    authType: providerForm.authType,
    keyAlias: providerForm.authType === 'none' ? undefined : providerForm.keyAlias?.trim() || undefined,
    status: providerForm.status,
    isEnabled: providerForm.isEnabled,
    config,
  }
}

function normalizeModelForm(): ProviderModelDto {
  const provider = providerConfigs.value.find((item) => item.providerId === modelForm.providerId)
  if (!provider) throw new Error(t('modelWorkflow.models.validation.providerRequired'))
  if (provider.providerKind === 'workflow') throw new Error(t('modelWorkflow.models.validation.workflowProvider'))
  const limits = parseJsonObject(modelLimitsJson.value, t('modelWorkflow.models.invalidLimitsJson'))
  const inputRequirements = parseConfigJsonValue(modelInputRequirementsJson.value, t('modelWorkflow.models.invalidInputRequirementsJson'))
  const config = parseJsonObject(modelConfigJson.value, t('modelWorkflow.models.invalidConfigJson'))
  return {
    modelId: modelForm.modelId.trim(),
    providerId: provider.providerId,
    providerKind: provider.providerKind,
    vendor: provider.vendor,
    providerModelId: modelForm.providerModelId.trim(),
    modelName: modelForm.modelName.trim(),
    displayName: modelForm.displayName.trim(),
    abilityTypes: [...modelForm.abilityTypes] as ModelCapability[] | string[],
    inputModalities: [...modelForm.inputModalities],
    outputModalities: [...modelForm.outputModalities],
    featureFlags: [...modelForm.featureFlags],
    limits,
    inputRequirements,
    apiContractVerified: modelForm.apiContractVerified,
    status: modelForm.status,
    isEnabled: modelForm.isEnabled,
    config,
  }
}

function normalizeWorkflowForm(): WorkflowPresetDto {
  const provider = providerConfigs.value.find((item) => item.providerId === workflowForm.providerId)
  if (!provider) throw new Error(t('modelWorkflow.workflow.validation.providerRequired'))
  if (provider.providerKind !== 'workflow') throw new Error(t('modelWorkflow.workflow.validation.providerKind'))
  if (provider.vendor !== workflowForm.vendor) throw new Error(t('modelWorkflow.workflow.validation.vendorMismatch'))
  const limits = parseJsonObject(workflowLimitsJson.value, t('modelWorkflow.workflow.invalidLimitsJson'))
  const paramSchema = parseJsonObject(workflowParamSchemaJson.value, t('modelWorkflow.workflow.invalidParamSchemaJson'))
  const nodeMap = parseStringRecord(workflowNodeMapJson.value, t('modelWorkflow.workflow.invalidNodeMapJson'))
  const outputMap = parseStringRecord(workflowOutputMapJson.value, t('modelWorkflow.workflow.invalidOutputMapJson'))
  const defaultParams = parseJsonObject(workflowDefaultParamsJson.value, t('modelWorkflow.workflow.invalidDefaultParamsJson'))
  const config = parseJsonObject(workflowConfigJson.value, t('modelWorkflow.workflow.invalidConfigJson'))
  return {
    workflowPresetId: workflowForm.workflowPresetId.trim(),
    providerId: provider.providerId,
    vendor: workflowForm.vendor,
    workflowKey: workflowForm.workflowKey.trim(),
    workflowId: workflowForm.vendor === 'runninghub' ? workflowForm.workflowId?.trim() || undefined : undefined,
    displayName: workflowForm.displayName.trim(),
    workflowVersion: workflowForm.workflowVersion.trim(),
    abilityTypes: [...workflowForm.abilityTypes],
    inputModalities: [...workflowForm.inputModalities],
    outputModalities: [...workflowForm.outputModalities],
    limits,
    paramSchema,
    nodeMap,
    outputMap,
    defaultParams,
    status: workflowForm.status,
    isBuiltin: workflowForm.isBuiltin,
    isEnabled: workflowForm.isEnabled,
    config,
  }
}

function parseConfigJson(): Record<string, unknown> {
  return parseJsonObject(providerConfigJson.value, t('modelWorkflow.provider.invalidConfigJson'))
}

function parseJsonObject(raw: string, errorText: string): Record<string, unknown> {
  try {
    const parsed = JSON.parse(raw || '{}')
    if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) return parsed as Record<string, unknown>
  } catch {
    throw new Error(errorText)
  }
  throw new Error(errorText)
}

function parseConfigJsonValue(raw: string, errorText: string): Record<string, unknown> | unknown[] {
  try {
    const parsed = JSON.parse(raw || '{}')
    if (parsed && typeof parsed === 'object') return parsed as Record<string, unknown> | unknown[]
  } catch {
    throw new Error(errorText)
  }
  throw new Error(errorText)
}

function parseStringRecord(raw: string, errorText: string): Record<string, string> {
  const parsed = parseJsonObject(raw, errorText)
  if (Object.values(parsed).every((value) => typeof value === 'string')) return parsed as Record<string, string>
  throw new Error(errorText)
}

function createDefaultProviderForm(): ProviderFormState {
  return {
    providerId: 'provider_dummy_image',
    providerKind: 'image',
    vendor: 'dummy',
    displayName: 'Dummy image',
    baseUrl: '',
    authType: 'none',
    keyAlias: undefined,
    status: 'ready',
    isEnabled: true,
  }
}

function createDefaultModelForm(): ModelFormState {
  const provider = providerConfigs.value.find((item) => item.providerKind !== 'workflow')
  const providerKind = provider?.providerKind === 'workflow' || !provider ? 'image' : provider.providerKind
  const providerId = provider?.providerId ?? ''
  return {
    modelId: providerId ? `${providerId}_model_image` : 'model_dummy_image',
    providerId,
    providerKind,
    vendor: provider?.vendor ?? 'dummy',
    providerModelId: 'dummy/image-v1',
    modelName: 'dummy-image-v1',
    displayName: 'Dummy image model',
    abilityTypes: ['text_to_image'],
    inputModalities: ['text'],
    outputModalities: ['image'],
    featureFlags: ['aspect_ratio', 'resolution'],
    apiContractVerified: false,
    status: 'ready',
    isEnabled: true,
  }
}

function createDefaultWorkflowForm(): WorkflowFormState {
  const provider = providerConfigs.value.find((item) => item.providerKind === 'workflow' && (item.vendor === 'comfyui' || item.vendor === 'runninghub'))
  const vendor = provider?.vendor === 'runninghub' ? 'runninghub' : 'comfyui'
  const providerId = provider?.providerId ?? ''
  return {
    workflowPresetId: providerId ? `${providerId}_workflow_i2v` : 'workflow_comfyui_i2v',
    providerId,
    vendor,
    workflowKey: vendor === 'runninghub' ? 'runninghub/video_wan_i2v_v1' : 'comfyui/video_wan_i2v_v1/workflow_api.json',
    workflowId: vendor === 'runninghub' ? 'rh_workflow_id' : undefined,
    displayName: vendor === 'runninghub' ? 'RunningHub Wan I2V' : 'ComfyUI Wan I2V',
    workflowVersion: '1.0.0',
    abilityTypes: ['image_to_video'],
    inputModalities: ['text', 'image'],
    outputModalities: ['video'],
    status: 'ready',
    isBuiltin: false,
    isEnabled: true,
  }
}

function defaultModelLimitsJson() {
  return JSON.stringify({
    supportedAspectRatios: ['9:16', '16:9', '1:1'],
    resolutions: ['720p', '1080p'],
    maxReferenceImages: 1,
  }, null, 2)
}

function defaultWorkflowLimitsJson() {
  return JSON.stringify({
    durationSeconds: { min: 3, max: 8, integer: true },
    supportedAspectRatios: ['9:16'],
    maxReferenceImages: 1,
  }, null, 2)
}

function defaultWorkflowParamSchemaJson() {
  return JSON.stringify({
    prompt: { type: 'string', required: true },
    input_image: { type: 'asset_path', required: true },
  }, null, 2)
}

function defaultWorkflowNodeMapJson() {
  return JSON.stringify({
    prompt: '12.inputs.text',
    input_image: '7.inputs.image',
  }, null, 2)
}

function defaultWorkflowOutputMapJson() {
  return JSON.stringify({
    video: '99.outputs.video',
  }, null, 2)
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error)
}

function inputRequirementClass(requirement: string) {
  if (requirement === 'required') return 'border-status-failed/40 bg-status-failed/10 text-status-failed'
  if (requirement === 'optional') return 'border-status-running/40 bg-status-running/10 text-status-running'
  return 'border-border bg-page text-muted'
}
</script>
