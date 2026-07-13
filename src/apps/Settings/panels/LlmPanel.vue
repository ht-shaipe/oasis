<template>
    <div class="section-panel">
        <div class="llm-header">
            <h2 class="section-heading">{{ t('settings.llm.title') }}</h2>
        </div>

        <el-tabs v-model="activeTab" class="llm-tabs">
            <el-tab-pane :label="t('settings.llm.tabRemote')" name="remote">
                <div class="tab-toolbar">
                    <el-button type="primary" size="small" @click="openAddDialog">
                        <el-icon><Plus /></el-icon>
                        {{ t('llm.addModel') }}
                    </el-button>
                </div>

                <div v-if="remoteModels.length === 0" class="llm-empty">
                    {{ t('settings.llm.empty') }}
                </div>
                <div v-else class="llm-list">
                    <div v-for="model in remoteModels" :key="model.id" class="llm-row">
                        <div class="llm-row-main">
                            <div class="llm-row-info">
                                <div class="llm-row-title">
                                    <span class="llm-row-name">{{ model.name }}</span>
                                    <span class="llm-provider-tag">{{ getProviderLabel(model.provider) }}</span>
                                    <span class="llm-type-tag" :class="model.model_type === 'embedding' ? 'type-embedding' : 'type-chat'">{{ model.model_type === 'embedding' ? t('llm.embedding') : t('llm.chat') }}</span>
                                </div>
                                <div class="llm-row-meta">
                                    <span class="llm-meta-item">
                                        <span class="llm-meta-label">{{ t('llm.modelId') }}:</span>
                                        <span class="llm-meta-value">{{ model.model_id }}</span>
                                    </span>
                                    <span class="llm-meta-item">
                                        <span class="llm-meta-label">{{ t('llm.baseUrl') }}:</span>
                                        <span class="llm-meta-value llm-base-url">{{ model.base_url }}</span>
                                    </span>
                                </div>
                            </div>
                            <div class="llm-row-actions">
                                <el-switch
                                    v-model="model.enabled"
                                    @change="toggleModelStatus(model)"
                                    :loading="model.statusLoading" />
                                <el-button link @click="openEditDialog(model)">
                                    <el-icon><Edit /></el-icon>
                                </el-button>
                                <el-button link type="danger" @click="deleteModel(model)">
                                    <el-icon><Delete /></el-icon>
                                </el-button>
                            </div>
                        </div>
                    </div>
                </div>
            </el-tab-pane>

            <el-tab-pane :label="t('settings.llm.tabLocal')" name="local">
                <div class="text-[13px] text-[var(--color-text-secondary)] mb-4 leading-7">
                    {{ t('settings.localModel.description') }}
                </div>

                <div class="mb-4">
                    <div class="mb-2">
                        <el-input
                            v-model="hfSearchQuery"
                            :placeholder="t('settings.localModel.hfSearchPlaceholder')"
                            size="small"
                            clearable
                            @keyup.enter="handleHfSearch"
                        >
                            <template #prefix>
                                <el-icon><Search /></el-icon>
                            </template>
                            <template #append>
                                <el-button :loading="hfSearching" @click="handleHfSearch">
                                    {{ t('settings.localModel.hfSearch') }}
                                </el-button>
                            </template>
                        </el-input>
                    </div>

                    <div v-if="hfSearchResults.length > 0" class="border border-solid border-[var(--color-card-border)] rounded-2 bg-[var(--color-card-bg)]">
                        <div class="flex items-center justify-between px-3 py-2 text-[13px] text-[var(--color-text-secondary)] border-b border-solid border-[var(--color-card-border)]">
                            <span>{{ t('settings.localModel.hfResultsCount', { count: hfSearchResults.length }) }}</span>
                            <el-button size="small" link type="primary" @click="hfSearchResults = []">
                                {{ t('common.close') }}
                            </el-button>
                        </div>
                        <el-scrollbar class="max-h-300px">
                            <div
                                v-for="m in hfSearchResults"
                                :key="m.modelId"
                                class="hf-result-item"
                            >
                                <div class="min-w-0 flex-1">
                                    <div class="text-[13px] text-[var(--color-text-primary)] font-500 flex items-center gap-1.5 flex-wrap">
                                        <span>{{ m.modelId }}</span>
                                        <span v-if="m.pipelineTag" class="hf-tag">{{ m.pipelineTag }}</span>
                                        <span v-if="m.libraryName" class="hf-tag lib">{{ m.libraryName }}</span>
                                    </div>
                                    <div class="flex gap-2 text-[11px] text-[var(--color-text-tertiary)] mt-0.5">
                                        <span v-if="m.downloads">{{ formatNumber(m.downloads) }} downloads</span>
                                        <span v-if="m.likes">{{ formatNumber(m.likes) }} likes</span>
                                    </div>
                                </div>
                                <div class="shrink-0">
                                    <template v-if="!isLocalModelKnown(m.modelId)">
                                        <el-button
                                            type="primary"
                                            size="small"
                                            :loading="addingFromHf === m.modelId"
                                            @click="handleAddFromHf(m.modelId)"
                                        >
                                            {{ t('settings.localModel.hfAddAndDownload') }}
                                        </el-button>
                                    </template>
                                    <template v-else-if="isLocalModelDownloaded(m.modelId)">
                                        <span class="text-[12px] font-500 text-primary">{{ t('settings.localModel.hfDownloaded') }}</span>
                                        <el-button
                                            type="danger"
                                            size="small"
                                            link
                                            @click="isLocalModelCustom(m.modelId) ? handleRemoveCustom(m.modelId) : handleLocalDelete(m.modelId)"
                                        >
                                            <el-icon><Delete /></el-icon>
                                        </el-button>
                                    </template>
                                    <template v-else>
                                        <span class="text-[12px] text-[var(--color-text-tertiary)]">{{ t('settings.localModel.hfAlreadyAdded') }}</span>
                                        <el-button
                                            v-if="isLocalModelCustom(m.modelId)"
                                            type="danger"
                                            size="small"
                                            link
                                            @click="handleRemoveCustom(m.modelId)"
                                        >
                                            <el-icon><Delete /></el-icon>
                                        </el-button>
                                    </template>
                                </div>
                            </div>
                        </el-scrollbar>
                    </div>
                </div>

                <div v-if="localModels.length > 0" class="section-divider">
                    <span>{{ t('settings.localModel.myModels') }}</span>
                </div>

                <div v-if="localModels.length > 0" class="local-model-list">
                    <div
                        v-for="model in sortedLocalModels"
                        :key="model.id"
                        class="local-model-card"
                        :class="{
                            active: model.isActive,
                            loaded: model.isLoaded && !model.isActive,
                            downloaded: model.downloaded && !model.isActive && !model.isLoaded,
                            pending: !model.downloaded,
                            downloading: !model.downloaded && downloadingId === model.id,
                        }"
                    >
                        <div class="flex items-center gap-2">
                            <span class="text-[14px] text-[var(--color-text-primary)] font-600">{{ model.name }}</span>
                            <span class="local-quant-tag">{{ model.paramsBillions }}B</span>
                            <span v-if="model.isCustom" class="custom-tag">Custom</span>
                            <span v-if="model.isActive" class="local-status-tag local-status-active">
                                <span class="running-dot"></span>
                                {{ t('settings.localModel.running') }}
                            </span>
                            <span v-else-if="model.isLoaded" class="local-status-tag local-status-loaded">
                                {{ t('settings.localModel.loaded') }}
                            </span>
                            <span v-else-if="model.downloaded" class="local-status-tag local-status-downloaded">
                                {{ t('settings.localModel.downloaded') }}
                            </span>
                            <span v-else class="local-status-tag local-status-pending">
                                {{ t('settings.localModel.pendingDownload') }}
                            </span>
                        </div>
                        <div class="flex gap-2.5 text-[12px] text-[var(--color-text-tertiary)]">
                            <span class="whitespace-nowrap">~{{ model.sizeMb >= 1000 ? (model.sizeMb / 1000).toFixed(1) + 'GB' : model.sizeMb + 'MB' }}</span>
                            <span class="whitespace-nowrap" :title="model.ggufFile || ''">{{ extractQuantLabel(model) }}</span>
                            <span class="whitespace-nowrap">{{ model.license }}</span>
                        </div>
                        <div class="text-[12px] text-[var(--color-text-secondary)] leading-5">{{ model.description }}</div>

                        <div v-if="loadingModel" class="text-[12px] text-[#e6a23c] flex items-center gap-1">
                            <span class="running-dot" style="background:#e6a23c"></span>
                            {{ t('settings.localModel.loadingModel') }}
                        </div>

                        <el-progress
                            v-if="downloadProgress[model.id] && downloadProgress[model.id].status === 'downloading'"
                            :percentage="downloadProgress[model.id].percentage"
                            :stroke-width="4"
                            :show-text="true"
                            :format="(p: number) => `${p}%`"
                            class="my-1"
                        />

                        <div v-if="downloadProgress[model.id]?.status === 'cancelled'" class="text-[12px] text-[#e6a23c]">
                            {{ t('settings.localModel.downloadCancelled') }}
                        </div>

                        <div class="flex items-center gap-2 mt-0.5">
                            <template v-if="!model.downloaded">
                                <el-button
                                    type="primary"
                                    size="small"
                                    :loading="downloadingId === model.id"
                                    @click="handleLocalDownload(model.id)"
                                >
                                    {{ t('settings.localModel.download') }}
                                </el-button>
                                <el-button
                                    v-if="downloadingId === model.id"
                                    size="small"
                                    type="warning"
                                    plain
                                    @click="handleLocalCancelDownload"
                                >
                                    {{ t('settings.localModel.cancel') }}
                                </el-button>
                                <el-button
                                    type="danger"
                                    size="small"
                                    link
                                    @click="model.isCustom ? handleRemoveCustom(model.id) : handleLocalDelete(model.id)"
                                >
                                    <el-icon><Delete /></el-icon>
                                </el-button>
                            </template>
                            <template v-else>
                                <el-button
                                    v-if="!model.isActive"
                                    type="success"
                                    size="small"
                                    plain
                                    :loading="activatingId === model.id"
                                    @click="handleLocalActivate(model.id)"
                                >
                                    {{ t('settings.localModel.activate') }}
                                </el-button>
                                <el-button
                                    v-if="model.isActive"
                                    type="warning"
                                    size="small"
                                    plain
                                    @click="handleLocalUnload"
                                >
                                    {{ t('settings.localModel.unload') }}
                                </el-button>
                                <el-button
                                    v-if="model.isCustom"
                                    type="danger"
                                    size="small"
                                    link
                                    @click="handleRemoveCustom(model.id)"
                                >
                                    <el-icon><Delete /></el-icon>
                                </el-button>
                                <el-button
                                    v-if="!model.isCustom"
                                    type="danger"
                                    size="small"
                                    link
                                    @click="handleLocalDelete(model.id)"
                                >
                                    <el-icon><Delete /></el-icon>
                                </el-button>
                            </template>
                        </div>
                    </div>
                </div>

                <div v-if="hiddenLocalModels.length > 0" class="section-divider">
                    <span>{{ t('settings.localModel.hiddenModels') }}</span>
                </div>

                <div v-if="hiddenLocalModels.length > 0" class="text-[12px] text-[var(--color-text-tertiary)] mb-3">
                    {{ t('settings.localModel.hiddenModelsHint') }}
                </div>

                <div v-if="hiddenLocalModels.length > 0" class="grid grid-cols-2 gap-3">
                    <div
                        v-for="model in hiddenLocalModels"
                        :key="model.id"
                        class="local-model-card hidden"
                    >
                        <div class="flex items-center gap-2">
                            <span class="text-[14px] text-[var(--color-text-primary)] font-600">{{ model.name }}</span>
                            <span class="hidden-tag">{{ t('settings.localModel.hidden') }}</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <el-button
                                type="primary"
                                size="small"
                                plain
                                @click="handleRestoreModel(model.id)"
                            >
                                {{ t('settings.localModel.restore') }}
                            </el-button>
                        </div>
                    </div>
                </div>

                <div v-if="localModels.length === 0 && hiddenLocalModels.length === 0" class="llm-empty">
                    {{ t('settings.localModel.noModels') }}
                </div>
            </el-tab-pane>
        </el-tabs>

        <el-dialog
            v-model="ggufSelectVisible"
            :title="t('settings.localModel.selectQuant')"
            width="480px"
            :close-on-click-modal="false"
            append-to-body>
            <div class="gguf-select-list">
                <div
                    v-for="g in ggufFileOptions"
                    :key="g.filename"
                    class="gguf-option"
                    :class="{ active: selectedGgufFile === g.filename }"
                    @click="selectedGgufFile = g.filename"
                >
                    <div class="gguf-option-main">
                        <el-radio :value="g.filename" v-model="selectedGgufFile" />
                        <div class="gguf-option-info">
                            <span class="gguf-quant-label">{{ g.quantLabel }}</span>
                            <span class="gguf-filename">{{ g.filename }}</span>
                        </div>
                    </div>
                    <span class="gguf-size">~{{ g.estimatedSizeMb >= 1000 ? (g.estimatedSizeMb / 1000).toFixed(1) + 'GB' : Math.round(g.estimatedSizeMb) + 'MB' }}</span>
                </div>
            </div>
            <template #footer>
                <el-button @click="ggufSelectVisible = false">{{ t('app.cancel') }}</el-button>
                <el-button type="primary" @click="confirmGgufSelect" :disabled="!selectedGgufFile">
                    {{ t('settings.localModel.hfAddAndDownload') }}
                </el-button>
            </template>
        </el-dialog>

        <el-dialog
            v-model="dialogVisible"
            :title="isEditing ? t('llm.editModel') : t('llm.addModel')"
            width="560px"
            :close-on-click-modal="false"
            append-to-body>
            <div class="add-model-flow">
                <div class="step-section" v-if="!isEditing">
                    <div class="step-label">
                        <span class="step-num">1</span>
                        {{ t('llm.selectProvider') }}
                    </div>
                    <div class="provider-grid">
                        <div
                            v-for="p in providers"
                            :key="p.code"
                            class="provider-card"
                            :class="{ active: formData.provider === p.code }"
                            @click="selectProvider(p)">
                            <span class="provider-name">{{ p.name }}</span>
                        </div>
                    </div>
                </div>

                <div class="step-section" v-if="formData.provider || isEditing">
                    <div class="step-label" v-if="!isEditing">
                        <span class="step-num">2</span>
                        {{ t('llm.inputApiKey') }}
                    </div>
                    <el-form :model="formData" :rules="formRules" ref="formRef" label-width="90px">
                        <el-form-item :label="t('llm.provider')" v-if="isEditing">
                            <el-select v-model="formData.provider" :placeholder="t('llm.selectProvider')" style="width: 100%"
                                @change="onProviderChange">
                                <el-option v-for="p in providers" :key="p.code" :value="p.code" :label="p.name" />
                            </el-select>
                        </el-form-item>
                        <el-form-item :label="t('llm.baseUrl')" prop="base_url">
                            <el-input v-model="formData.base_url" :placeholder="t('llm.baseUrlPlaceholder')" />
                        </el-form-item>
                        <el-form-item :label="t('llm.apiKey')" prop="api_key">
                            <el-input v-model="formData.api_key" :placeholder="t('llm.apiKeyPlaceholder')" type="password" show-password />
                        </el-form-item>
                        <el-button
                            v-if="!isEditing && formData.provider && formData.api_key"
                            class="fetch-btn"
                            :loading="fetchingModels"
                            @click="fetchModels">
                            {{ t('llm.fetchModels') }}
                        </el-button>
                    </el-form>
                </div>

                <div class="step-section" v-if="fetchedRemoteModels.length > 0 || isEditing">
                    <div class="step-label" v-if="!isEditing">
                        <span class="step-num">3</span>
                        {{ t('llm.selectModel') }}
                    </div>

                    <div class="remote-model-list" v-if="fetchedRemoteModels.length > 0 && !isEditing">
                        <div class="remote-model-type-select">
                            <span class="rm-type-label">{{ t('llm.modelType') }}:</span>
                            <el-radio-group v-model="formData.model_type" size="small">
                                <el-radio-button value="chat">{{ t('llm.chat') }}</el-radio-button>
                                <el-radio-button value="embedding">{{ t('llm.embedding') }}</el-radio-button>
                            </el-radio-group>
                        </div>
                        <el-checkbox-group v-model="selectedModelIds">
                            <div v-for="rm in fetchedRemoteModels" :key="rm.id" class="remote-model-item">
                                <el-checkbox :value="rm.id">
                                    <span class="rm-id">{{ rm.id }}</span>
                                    <span class="rm-owned" v-if="rm.owned_by">({{ rm.owned_by }})</span>
                                </el-checkbox>
                            </div>
                        </el-checkbox-group>
                    </div>

                    <el-form v-if="isEditing" :model="formData" ref="editFormRef" label-width="90px">
                        <el-form-item :label="t('llm.modelType')">
                            <el-radio-group v-model="formData.model_type">
                                <el-radio-button value="chat">{{ t('llm.chat') }}</el-radio-button>
                                <el-radio-button value="embedding">{{ t('llm.embedding') }}</el-radio-button>
                            </el-radio-group>
                        </el-form-item>
                        <el-form-item :label="t('llm.modelName')" prop="name">
                            <el-input v-model="formData.name" :placeholder="t('llm.modelNamePlaceholder')" />
                        </el-form-item>
                        <el-form-item :label="t('llm.modelId')" prop="model_id">
                            <el-input v-model="formData.model_id" :placeholder="t('llm.modelIdPlaceholder')" />
                        </el-form-item>
                        <el-form-item :label="t('llm.temperature')">
                            <el-slider v-model="formData.temperature" :min="0" :max="2" :step="0.1" show-input />
                        </el-form-item>
                        <el-form-item :label="t('llm.maxTokens')">
                            <el-input-number v-model="formData.max_tokens" :min="1" :max="128000" style="width: 100%" />
                        </el-form-item>
                    </el-form>
                </div>
            </div>

            <template #footer>
                <el-button @click="dialogVisible = false">{{ t('app.cancel') }}</el-button>
                <el-button type="primary" @click="saveModel" :loading="saving" :disabled="!canSave">
                    {{ t('app.save') }}
                </el-button>
            </template>
        </el-dialog>
    </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { ElMessage, ElMessageBox, type FormRules } from 'element-plus';
import { Plus, Edit, Delete, Search } from '@element-plus/icons-vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

const { t } = useI18n();

// ── Shared ──
const activeTab = ref('remote');

// ── Remote Models ──
interface LLMModel {
    id: string;
    name: string;
    provider: string;
    model_id: string;
    base_url: string;
    auth_type: string;
    api_key: string;
    token_plan: string;
    temperature: number;
    max_tokens: number;
    description: string;
    enabled: boolean;
    model_type: string;
    statusLoading?: boolean;
}

interface ProviderOption {
    code: string;
    name: string;
    base_url: string;
}

interface RemoteModel {
    id: string;
    name: string;
    owned_by: string;
}

const remoteModels = ref<LLMModel[]>([]);
const providers = ref<ProviderOption[]>([]);
const fetchedRemoteModels = ref<RemoteModel[]>([]);
const selectedModelIds = ref<string[]>([]);

const dialogVisible = ref(false);
const isEditing = ref(false);
const saving = ref(false);
const fetchingModels = ref(false);
const editingModelId = ref<string | undefined>();

const formData = reactive<LLMModel>({
    id: '',
    name: '',
    provider: '',
    model_id: '',
    base_url: '',
    auth_type: 'api_key',
    api_key: '',
    token_plan: '',
    temperature: 0.7,
    max_tokens: 4096,
    description: '',
    enabled: true,
    model_type: 'chat',
});

const formRules: FormRules = {
    base_url: [{ required: true, message: '请输入基础URL', trigger: 'blur' }],
    api_key: [{ required: true, message: '请输入API密钥', trigger: 'blur' }],
    name: [{ required: true, message: '请输入模型名称', trigger: 'blur' }],
    model_id: [{ required: true, message: '请输入模型ID', trigger: 'blur' }],
};

const canSave = computed(() => {
    if (isEditing.value) {
        return !!formData.name && !!formData.model_id;
    }
    return selectedModelIds.value.length > 0;
});

const providerLabels = computed(() => {
    const map: Record<string, string> = {};
    for (const p of providers.value) {
        map[p.code] = p.name;
    }
    return map;
});

const getProviderLabel = (provider: string): string => {
    return providerLabels.value[provider] || provider;
};

const selectProvider = (p: ProviderOption) => {
    formData.provider = p.code;
    formData.base_url = p.base_url;
    fetchedRemoteModels.value = [];
    selectedModelIds.value = [];
};

const onProviderChange = (code: string) => {
    const p = providers.value.find(p => p.code === code);
    if (p) {
        formData.base_url = p.base_url;
    }
};

const resetForm = () => {
    Object.assign(formData, {
        id: '',
        name: '',
        provider: '',
        model_id: '',
        base_url: '',
        auth_type: 'api_key',
        api_key: '',
        token_plan: '',
        temperature: 0.7,
        max_tokens: 4096,
        description: '',
        enabled: true,
        model_type: 'chat',
    });
    fetchedRemoteModels.value = [];
    selectedModelIds.value = [];
};

const loadRemoteModels = async () => {
    try {
        const all = await invoke<LLMModel[]>('get_llm_models');
        remoteModels.value = all;
    } catch (e) {
        console.error('Failed to load LLM models:', e);
    }
};

const loadProviders = async () => {
    try {
        providers.value = await invoke<ProviderOption[]>('get_llm_providers');
    } catch (e) {
        console.error('Failed to load providers:', e);
    }
};

const openAddDialog = () => {
    isEditing.value = false;
    editingModelId.value = undefined;
    resetForm();
    dialogVisible.value = true;
};

const openEditDialog = (model: LLMModel) => {
    isEditing.value = true;
    editingModelId.value = model.id;
    Object.assign(formData, { ...model });
    fetchedRemoteModels.value = [];
    selectedModelIds.value = [];
    dialogVisible.value = true;
};

const fetchModels = async () => {
    if (!formData.provider || !formData.api_key) {
        ElMessage.warning(t('llm.providerKeyRequired'));
        return;
    }
    fetchingModels.value = true;
    fetchedRemoteModels.value = [];
    selectedModelIds.value = [];
    try {
        const list = await invoke<RemoteModel[]>('fetch_provider_models', {
            provider: formData.provider,
            baseUrl: formData.base_url,
            apiKey: formData.api_key,
        });
        if (list.length === 0) {
            ElMessage.info(t('llm.noModelsFound'));
        }
        fetchedRemoteModels.value = list;
    } catch (e: any) {
        ElMessage.error(e?.toString() || t('llm.fetchModelsFailed'));
    } finally {
        fetchingModels.value = false;
    }
};

const saveModel = async () => {
    saving.value = true;
    try {
        if (isEditing.value) {
            const modelToSave = { ...formData };
            const saved = await invoke<LLMModel>('save_llm_model', { model: modelToSave });
            const index = remoteModels.value.findIndex((m) => m.id === editingModelId.value);
            if (index !== -1) remoteModels.value[index] = saved;
            ElMessage.success(t('llm.updateSuccess'));
            dialogVisible.value = false;
        } else {
            const promises = selectedModelIds.value.map((modelId) => {
                const remote = fetchedRemoteModels.value.find((r) => r.id === modelId);
                const model: LLMModel = {
                    id: crypto.randomUUID(),
                    name: remote?.name || modelId,
                    provider: formData.provider,
                    model_id: modelId,
                    base_url: formData.base_url,
                    auth_type: 'api_key',
                    api_key: formData.api_key,
                    token_plan: '',
                    temperature: 0.7,
                    max_tokens: 4096,
                    description: '',
                    enabled: true,
                    model_type: formData.model_type,
                };
                return invoke<LLMModel>('save_llm_model', { model });
            });
            const results = await Promise.all(promises);
            remoteModels.value.push(...results);
            ElMessage.success(t('llm.addSuccess'));
            dialogVisible.value = false;
        }
    } catch (e: any) {
        ElMessage.error(e?.toString() || 'Save failed');
    } finally {
        saving.value = false;
    }
};

const deleteModel = async (model: LLMModel) => {
    try {
        await ElMessageBox.confirm(t('llm.deleteConfirm'), t('llm.deleteModel'), {
            type: 'warning',
        });
        await invoke('delete_llm_model', { id: model.id });
        remoteModels.value = remoteModels.value.filter((m) => m.id !== model.id);
        ElMessage.success(t('llm.deleteSuccess'));
    } catch (e: any) {
        if (e !== 'cancel') {
            ElMessage.error(e?.toString() || 'Delete failed');
        }
    }
};

const toggleModelStatus = async (model: LLMModel) => {
    model.statusLoading = true;
    try {
        await invoke('toggle_llm_model', { id: model.id, enabled: model.enabled });
        ElMessage.success(model.enabled ? t('llm.enabledSuccess') : t('llm.disabledSuccess'));
    } catch (e: any) {
        model.enabled = !model.enabled;
        ElMessage.error(e?.toString() || 'Operation failed');
    } finally {
        model.statusLoading = false;
    }
};

// ── Local Models ──
interface LocalChatModel {
    id: string
    name: string
    paramsBillions: number
    sizeMb: number
    license: string
    description: string
    downloaded: boolean
    isActive: boolean
    isLoaded: boolean
    isCustom: boolean
    ggufFile: string
}

interface DownloadProgressPayload {
    modelId: string
    fileName: string
    current: number
    total: number
    percentage: number
    status: string
}

interface ModelLoadState {
    modelId: string
    status: string
    message: string | null
}

interface HfModelSearchResult {
    modelId: string
    author: string
    downloads: number
    likes: number
    pipelineTag: string | null
    tags: string[]
    libraryName: string | null
    createdAt: string | null
}

interface HfModelFileInfo {
    rfilename: string
}

interface HfModelInfo {
    modelId: string
    siblings: HfModelFileInfo[]
    tags: string[]
    pipelineTag: string | null
    libraryName: string | null
    downloads: number | null
    likes: number | null
}

interface GgufFileOption {
    filename: string
    quantLabel: string
    estimatedSizeMb: number
}

const localModels = ref<LocalChatModel[]>([])
const hiddenLocalModels = ref<LocalChatModel[]>([])
const downloadingId = ref<string | null>(null)
const activatingId = ref<string | null>(null)
const loadingModel = ref(false)
const downloadProgress = reactive<Record<string, DownloadProgressPayload>>({})

const hfSearchQuery = ref('')
const hfSearching = ref(false)
const hfSearchResults = ref<HfModelSearchResult[]>([])
const addingFromHf = ref<string | null>(null)
const ggufSelectVisible = ref(false)
const ggufSelectModelId = ref('')
const ggufFileOptions = ref<GgufFileOption[]>([])
const selectedGgufFile = ref('')

const sortedLocalModels = computed(() => {
    return [...localModels.value].sort((a, b) => {
        if (a.isActive && !b.isActive) return -1
        if (!a.isActive && b.isActive) return 1
        if (a.isLoaded && !b.isLoaded) return -1
        if (!a.isLoaded && b.isLoaded) return 1
        if (a.downloaded && !b.downloaded) return -1
        if (!a.downloaded && b.downloaded) return 1
        return a.paramsBillions - b.paramsBillions
    })
})

const localKnownIds = computed(() => new Set(localModels.value.map(m => m.id)))
const localDownloadedIds = computed(() => new Set(localModels.value.filter(m => m.downloaded).map(m => m.id)))
const localCustomIds = computed(() => new Set(localModels.value.filter(m => m.isCustom).map(m => m.id)))

function isLocalModelKnown(modelId: string): boolean {
    return localKnownIds.value.has(modelId)
}

function isLocalModelDownloaded(modelId: string): boolean {
    return localDownloadedIds.value.has(modelId)
}

function isLocalModelCustom(modelId: string): boolean {
    return localCustomIds.value.has(modelId)
}

let unlistenProgress: UnlistenFn | null = null
let unlistenLoad: UnlistenFn | null = null

async function loadLocalModels() {
    try {
        localModels.value = await invoke<LocalChatModel[]>('list_local_chat_models')
    } catch (e) {
        console.error('Failed to load local chat models:', e)
    }
}

async function loadHiddenLocalModels() {
    try {
        const cfg = await invoke<{ hiddenIds: string[], customModels: any[] }>('get_local_chat_config')
        if (cfg.hiddenIds && cfg.hiddenIds.length > 0) {
            const catalog: { id: string; name: string; paramsBillions: number }[] = [
                { id: 'HuggingFaceTB/SmolLM2-135M-Instruct', name: 'SmolLM2 135M', paramsBillions: 0.135 },
                { id: 'HuggingFaceTB/SmolLM2-360M-Instruct', name: 'SmolLM2 360M', paramsBillions: 0.36 },
                { id: 'HuggingFaceTB/SmolLM2-1.7B-Instruct', name: 'SmolLM2 1.7B', paramsBillions: 1.7 },
            ]
            hiddenLocalModels.value = cfg.hiddenIds
                .filter(id => catalog.some(c => c.id === id))
                .map(id => {
                    const entry = catalog.find(c => c.id === id)!
                    return {
                        id: entry.id,
                        name: entry.name,
                        paramsBillions: entry.paramsBillions,
                        sizeMb: 0,
                        license: '',
                        description: '',
                        downloaded: false,
                        isActive: false,
                        isLoaded: false,
                        isCustom: false,
                        ggufFile: '',
                    }
                })
        } else {
            hiddenLocalModels.value = []
        }
    } catch (e) {
        console.error('Failed to load hidden models:', e)
    }
}

async function handleLocalDownload(modelId: string) {
    downloadingId.value = modelId
    try {
        await invoke('download_local_chat_model', { modelId, autoActivate: true })
        ElMessage.success(t('settings.localModel.downloadSuccess'))
        await loadLocalModels()
    } catch (e: unknown) {
        const msg = `${e}`
        if (msg.includes('Manual download:')) {
            ElMessageBox.alert(msg, 'Download Failed', { confirmButtonText: 'OK', customClass: 'download-error-msg' })
        } else {
            ElMessage.error(msg)
        }
    } finally {
        downloadingId.value = null
        delete downloadProgress[modelId]
    }
}

async function handleLocalCancelDownload() {
    try {
        await invoke('cancel_local_chat_download')
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    }
}

async function handleLocalActivate(modelId: string) {
    activatingId.value = modelId
    loadingModel.value = true
    try {
        await invoke('set_active_local_chat_model', { modelId })
        ElMessage.success(t('settings.localModel.activateSuccess'))
        await loadLocalModels()
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    } finally {
        activatingId.value = null
        loadingModel.value = false
    }
}

async function handleLocalUnload() {
    try {
        await invoke('unload_local_chat_model')
        ElMessage.success(t('settings.localModel.unloadSuccess'))
        await loadLocalModels()
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    }
}

async function handleLocalDelete(modelId: string) {
    try {
        await ElMessageBox.confirm(
            t('settings.localModel.deleteConfirm'),
            t('settings.localModel.deleteModel'),
            { type: 'warning' }
        )
        await invoke('delete_local_chat_model', { modelId })
        ElMessage.success(t('settings.localModel.deleteSuccess'))
        await loadLocalModels()
    } catch {
        // cancelled
    }
}

async function handleRemoveCustom(modelId: string) {
    try {
        await ElMessageBox.confirm(
            t('settings.localModel.removeCustomConfirm'),
            t('settings.localModel.removeCustom'),
            { type: 'warning' }
        )
        await invoke('remove_custom_local_chat_model', { modelId })
        ElMessage.success(t('settings.localModel.removeCustomSuccess'))
        await loadLocalModels()
    } catch {
        // cancelled
    }
}

async function handleRestoreModel(modelId: string) {
    try {
        await invoke('restore_local_chat_model', { modelId })
        ElMessage.success(t('settings.localModel.restoreSuccess'))
        await Promise.all([loadLocalModels(), loadHiddenLocalModels()])
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    }
}

async function handleHfSearch() {
    hfSearching.value = true
    try {
        const query = hfSearchQuery.value.trim() || undefined
        hfSearchResults.value = await invoke<HfModelSearchResult[]>('search_hf_chat_models', {
            query,
            limit: 30,
        })
        if (hfSearchResults.value.length === 0) {
            ElMessage.info(t('settings.localModel.hfNoResults'))
        }
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    } finally {
        hfSearching.value = false
    }
}

function findGgufFile(siblings: HfModelFileInfo[]): string {
    const ggufFiles = siblings
        .map(f => f.rfilename)
        .filter(n => n.endsWith('.gguf'))

    const q4km = ggufFiles.find(n =>
        n.includes('Q4_K_M') || n.includes('q4_k_m') || n.includes('Q4_KM')
    )
    if (q4km) return q4km

    const q4 = ggufFiles.find(n =>
        n.includes('Q4_0') || n.includes('Q4_1') || n.includes('q4')
    )
    if (q4) return q4

    const q5 = ggufFiles.find(n =>
        n.includes('Q5_K_M') || n.includes('Q5_0') || n.includes('q5')
    )
    if (q5) return q5

    const q8 = ggufFiles.find(n =>
        n.includes('Q8_0') || n.includes('q8')
    )
    if (q8) return q8

    return ggufFiles[0] || ''
}

function estimateSizeMb(ggufFile: string, paramsBillions: number): number {
    if (paramsBillions > 0) {
        return Math.round(paramsBillions * 600)
    }
    const name = ggufFile.toLowerCase()
    if (name.includes('1.7b') || name.includes('1_7b')) return 1000
    if (name.includes('3b')) return 2000
    if (name.includes('7b')) return 4500
    if (name.includes('8b')) return 5000
    if (name.includes('13b')) return 8000
    if (name.includes('14b')) return 8500
    if (name.includes('70b')) return 40000
    return 1000
}

function estimateParams(ggufFile: string, modelId: string): number {
    const combined = (ggufFile + ' ' + modelId).toLowerCase()
    if (combined.includes('135m')) return 0.135
    if (combined.includes('360m')) return 0.36
    if (combined.includes('0.5b') || combined.includes('500m')) return 0.5
    if (combined.includes('1.7b') || combined.includes('1_7b')) return 1.7
    if (combined.includes('1.8b') || combined.includes('1_8b')) return 1.8
    if (combined.includes('2b')) return 2.0
    if (combined.includes('3b')) return 3.0
    if (combined.includes('7b')) return 7.0
    if (combined.includes('8b')) return 8.0
    if (combined.includes('13b')) return 13.0
    if (combined.includes('14b')) return 14.0
    if (combined.includes('70b')) return 70.0
    return 1.0
}

async function handleAddFromHf(modelId: string) {
    addingFromHf.value = modelId
    try {
        const ggufFiles = await invoke<GgufFileOption[]>('list_gguf_files', { modelId })

        if (ggufFiles.length === 0) {
            const info = await invoke<HfModelInfo>('get_hf_chat_model_info', { modelId })
            const ggufFile = findGgufFile(info.siblings)
            if (!ggufFile) {
                ElMessage.warning(t('settings.localModel.hfNoGguf'))
                return
            }
            await addAndDownloadHfModel(modelId, ggufFile)
            return
        }

        if (ggufFiles.length === 1) {
            await addAndDownloadHfModel(modelId, ggufFiles[0].filename)
            return
        }

        ggufSelectModelId.value = modelId
        ggufFileOptions.value = ggufFiles
        const q4km = ggufFiles.find(g => g.quantLabel === 'Q4_K_M')
        selectedGgufFile.value = q4km?.filename || ggufFiles[0].filename
        ggufSelectVisible.value = true
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    } finally {
        addingFromHf.value = null
    }
}

async function addAndDownloadHfModel(modelId: string, ggufFile: string) {
    try {
        const info = await invoke<HfModelInfo>('get_hf_chat_model_info', { modelId })
        const hasGguf = info.siblings.some(f => f.rfilename.endsWith('.gguf'))
        const displayName = modelId.replace(/-GGUF$/i, '').split('/').pop() || modelId
        const paramsBillions = estimateParams(ggufFile, modelId)
        const sizeMb = estimateSizeMb(ggufFile, paramsBillions)
        const license = info.tags.find(tag => tag.startsWith('license:'))?.replace('license:', '') || 'unknown'
        const hfRepo = modelId
        const tokModelId = modelId.replace(/-GGUF$/i, '')

        await invoke('add_custom_local_chat_model', {
            model: {
                modelId,
                name: displayName,
                paramsBillions,
                sizeMb,
                license,
                description: hasGguf
                    ? `${info.pipelineTag || 'text-generation'} GGUF model from HuggingFace`
                    : `⚠️ No GGUF file detected — may not work with local inference`,
                hfRepo,
                ggufFile,
                tokModelId,
            },
        })

        ElMessage.success(t('settings.localModel.hfAdded'))

        await loadLocalModels()

        if (hasGguf) {
            await handleLocalDownload(modelId)
        }
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    }
}

async function confirmGgufSelect() {
    ggufSelectVisible.value = false
    if (selectedGgufFile.value && ggufSelectModelId.value) {
        await addAndDownloadHfModel(ggufSelectModelId.value, selectedGgufFile.value)
    }
}

function formatNumber(n: number): string {
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M'
    if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K'
    return String(n)
}

function extractQuantLabel(model: LocalChatModel): string {
    const name = model.ggufFile.toUpperCase()
    if (name.includes('Q8_0')) return 'Q8_0'
    if (name.includes('Q6_K_L')) return 'Q6_K_L'
    if (name.includes('Q6_K_M')) return 'Q6_K_M'
    if (name.includes('Q6_K')) return 'Q6_K'
    if (name.includes('Q5_K_M')) return 'Q5_K_M'
    if (name.includes('Q5_K_S')) return 'Q5_K_S'
    if (name.includes('Q5_1')) return 'Q5_1'
    if (name.includes('Q5_0')) return 'Q5_0'
    if (name.includes('Q4_K_M')) return 'Q4_K_M'
    if (name.includes('Q4_K_S')) return 'Q4_K_S'
    if (name.includes('Q4_1')) return 'Q4_1'
    if (name.includes('Q4_0')) return 'Q4_0'
    if (name.includes('Q3_K_M')) return 'Q3_K_M'
    if (name.includes('Q3_K_S')) return 'Q3_K_S'
    if (name.includes('Q2_K')) return 'Q2_K'
    if (name.includes('FP16') || name.includes('F16')) return 'FP16'
    if (name.includes('BF16')) return 'BF16'
    return 'Q4_K_M'
}

function handleProgressEvent(event: { payload: DownloadProgressPayload }) {
    const p = event.payload
    downloadProgress[p.modelId] = p
    if (p.status === 'complete' || p.status === 'cancelled') {
        setTimeout(() => {
            delete downloadProgress[p.modelId]
        }, 3000)
    }
}

function handleLoadEvent(event: { payload: ModelLoadState }) {
    const p = event.payload
    loadingModel.value = p.status === 'loading'
    if (p.status === 'ready' || p.status === 'error') {
        loadLocalModels()
    }
}

// ── Lifecycle ──
onMounted(async () => {
    unlistenProgress = await listen<DownloadProgressPayload>('local-llm-download-progress', handleProgressEvent)
    unlistenLoad = await listen<ModelLoadState>('local-llm-load-progress', handleLoadEvent)
    await Promise.all([loadRemoteModels(), loadProviders(), loadLocalModels(), loadHiddenLocalModels()])
})

onUnmounted(() => {
    unlistenProgress?.()
    unlistenLoad?.()
})
</script>

<style scoped>
.llm-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
}

.llm-header .section-heading {
    margin-bottom: 0;
}

.llm-tabs :deep(.el-tabs__header) {
    margin-bottom: 16px;
}

.llm-tabs :deep(.el-tabs__item) {
    font-size: var(--app-font-13);
    font-weight: 500;
}

.tab-toolbar {
    display: flex;
    justify-content: flex-end;
    margin-bottom: 12px;
}

.llm-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.llm-row {
    padding: 16px 18px;
    border-radius: 10px;
    background: var(--color-card-bg);
    border: 1px solid var(--color-card-border);
    transition: box-shadow 0.15s;
}

.llm-row:hover {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.llm-row-main {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
}

.llm-row-info {
    flex: 1;
    min-width: 0;
}

.llm-row-title {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 8px;
}

.llm-row-name {
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text-primary);
}

.llm-provider-tag {
    font-size: 12px;
    color: var(--color-provider-tag-text, #42b883);
    background: var(--color-provider-tag-bg, rgba(66, 184, 131, 0.1));
    padding: 2px 8px;
    border-radius: 4px;
    font-weight: 500;
    white-space: nowrap;
}

.llm-type-tag {
    font-size: 11px;
    padding: 1px 6px;
    border-radius: 3px;
    font-weight: 500;
    white-space: nowrap;
}

.type-chat {
    color: #409eff;
    background: rgba(64, 158, 255, 0.1);
}

.type-embedding {
    color: #e6a23c;
    background: rgba(230, 162, 60, 0.1);
}

.llm-row-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 20px;
    font-size: 13px;
    color: var(--color-text-tertiary);
}

.llm-meta-item {
    display: flex;
    align-items: center;
    gap: 4px;
}

.llm-meta-label {
    color: var(--color-grey);
    flex-shrink: 0;
}

.llm-meta-value {
    color: var(--color-text-secondary);
}

.llm-base-url {
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: block;
}

.llm-row-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
}

.llm-empty {
    padding: 48px 0;
    text-align: center;
    font-size: 14px;
    color: var(--color-text-tertiary);
}

/* ── HF Search Results ── */
.hf-result-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 12px;
    border-bottom: 1px solid var(--color-card-border);
    transition: background 0.1s;
}

.hf-result-item:last-child {
    border-bottom: none;
}

.hf-result-item:hover {
    background: rgba(0, 0, 0, 0.02);
}

.hf-tag {
    font-size: 10px;
    padding: 1px 5px;
    border-radius: 3px;
    background: rgba(64, 158, 255, 0.08);
    color: #409eff;
    font-weight: 500;
}

.hf-tag.lib {
    background: rgba(103, 194, 58, 0.08);
    color: #67c23a;
}

.section-divider {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
    color: var(--color-text-tertiary);
    font-size: 12px;
}

.section-divider::before,
.section-divider::after {
    content: '';
    flex: 1;
    height: 1px;
    background: var(--color-card-border);
}

/* ── Local Model Cards ── */
.local-model-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.local-model-card {
    padding: 14px 16px;
    border-radius: 10px;
    background: var(--color-card-bg);
    border: 1.5px solid var(--color-card-border);
    transition: all 0.15s;
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.local-model-card:hover {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.local-model-card.active {
    border-color: #42b883;
    background: rgba(66, 184, 131, 0.04);
    box-shadow: 0 0 0 1px rgba(66, 184, 131, 0.15);
}

.local-model-card.loaded {
    border-color: rgba(64, 158, 255, 0.35);
    background: rgba(64, 158, 255, 0.02);
}

.local-model-card.downloaded {
    border-color: rgba(66, 184, 131, 0.35);
    background: rgba(66, 184, 131, 0.02);
}

.local-model-card.pending {
    border-color: rgba(0, 0, 0, 0.06);
    background: rgba(0, 0, 0, 0.01);
}

.local-model-card.downloading {
    border-color: #e6a23c;
    background: rgba(230, 162, 60, 0.04);
}

.local-model-card.hidden {
    opacity: 0.55;
}

.local-quant-tag {
    font-size: 11px;
    font-weight: 600;
    color: #e6a23c;
    background: rgba(230, 162, 60, 0.12);
    padding: 1px 6px;
    border-radius: 3px;
}

.custom-tag {
    font-size: 11px;
    font-weight: 600;
    color: #409eff;
    background: rgba(64, 158, 255, 0.1);
    padding: 1px 6px;
    border-radius: 3px;
}

.hidden-tag {
    font-size: 11px;
    font-weight: 600;
    color: #909399;
    background: rgba(144, 147, 153, 0.1);
    padding: 1px 6px;
    border-radius: 3px;
    margin-left: auto;
}

.local-status-tag {
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 10px;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
}

.local-status-active {
    color: #42b883;
    background: rgba(66, 184, 131, 0.12);
}

.local-status-loaded {
    color: #409eff;
    background: rgba(64, 158, 255, 0.1);
}

.local-status-downloaded {
    color: #67c23a;
    background: rgba(103, 194, 58, 0.08);
}

.local-status-pending {
    color: #909399;
    background: rgba(144, 147, 153, 0.08);
}

.running-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #42b883;
    animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
}

/* ── Add Model Dialog ── */
.add-model-flow {
    display: flex;
    flex-direction: column;
    gap: 20px;
}

.step-section {
    border-top: 1px solid var(--color-card-border);
    padding-top: 16px;
}

.step-section:first-child {
    border-top: none;
    padding-top: 0;
}

.step-label {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: 12px;
    display: flex;
    align-items: center;
    gap: 8px;
}

.step-num {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: #42b883;
    color: #fff;
    font-size: 12px;
    font-weight: 600;
    flex-shrink: 0;
}

.provider-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
}

.provider-card {
    padding: 10px 12px;
    border: 1.5px solid var(--color-input-border);
    border-radius: 8px;
    cursor: pointer;
    text-align: center;
    transition: all 0.15s;
    background: var(--color-input-bg);
}

.provider-card:hover {
    border-color: #42b883;
}

.provider-card.active {
    border-color: #42b883;
    background: rgba(66, 184, 131, 0.08);
    box-shadow: 0 0 0 1px rgba(66, 184, 131, 0.2);
}

.provider-name {
    font-size: 14px;
    color: var(--color-text-primary);
    font-weight: 500;
}

.fetch-btn {
    width: 100%;
    margin-top: 4px;
}

.remote-model-list {
    max-height: 240px;
    overflow-y: auto;
    border: 1px solid var(--color-input-border);
    border-radius: 8px;
    padding: 8px;
    background: var(--color-input-bg);
}

.remote-model-type-select {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--color-card-border);
}

.rm-type-label {
    font-size: 13px;
    color: var(--color-text-secondary);
    flex-shrink: 0;
}

.remote-model-item {
    padding: 4px 0;
}

.rm-id {
    font-size: 14px;
    color: var(--color-text-primary);
}

.rm-owned {
    font-size: 13px;
    color: var(--color-text-tertiary);
    margin-left: 6px;
}

/* ── GGUF Select Dialog ── */
.gguf-select-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 360px;
    overflow-y: auto;
}

.gguf-option {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border: 1.5px solid var(--color-card-border);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s;
}

.gguf-option:hover {
    border-color: #42b883;
    background: rgba(66, 184, 131, 0.03);
}

.gguf-option.active {
    border-color: #42b883;
    background: rgba(66, 184, 131, 0.06);
    box-shadow: 0 0 0 1px rgba(66, 184, 131, 0.15);
}

.gguf-option-main {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
}

.gguf-option-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
}

.gguf-quant-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-primary);
}

.gguf-filename {
    font-size: 11px;
    color: var(--color-text-tertiary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.gguf-size {
    font-size: 12px;
    font-weight: 500;
    color: #e6a23c;
    background: rgba(230, 162, 60, 0.1);
    padding: 2px 8px;
    border-radius: 4px;
    white-space: nowrap;
    flex-shrink: 0;
}
</style>

<style>
.download-error-msg .el-message-box__message p {
    white-space: pre-wrap;
    word-break: break-all;
    font-family: monospace;
    font-size: 12px;
    user-select: text;
}
</style>
