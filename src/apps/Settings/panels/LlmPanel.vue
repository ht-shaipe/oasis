<template>
    <div class="section-panel">
        <div class="llm-header">
            <h2 class="section-heading">{{ t('settings.llm.title') }}</h2>
            <el-button type="primary" @click="openAddDialog">
                <el-icon><Plus /></el-icon>
                {{ t('llm.addModel') }}
            </el-button>
        </div>

        <div v-if="models.length === 0" class="llm-empty">
            {{ t('settings.llm.empty') }}
        </div>
        <div v-else class="llm-list">
            <div v-for="model in models" :key="model.id" class="llm-row">
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

                <div class="step-section" v-if="remoteModels.length > 0 || isEditing">
                    <div class="step-label" v-if="!isEditing">
                        <span class="step-num">3</span>
                        {{ t('llm.selectModel') }}
                    </div>

                    <div class="remote-model-list" v-if="remoteModels.length > 0 && !isEditing">
                        <div class="remote-model-type-select">
                            <span class="rm-type-label">{{ t('llm.modelType') }}:</span>
                            <el-radio-group v-model="formData.model_type" size="small">
                                <el-radio-button value="chat">{{ t('llm.chat') }}</el-radio-button>
                                <el-radio-button value="embedding">{{ t('llm.embedding') }}</el-radio-button>
                            </el-radio-group>
                        </div>
                        <el-checkbox-group v-model="selectedModelIds">
                            <div v-for="rm in remoteModels" :key="rm.id" class="remote-model-item">
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
import { ref, reactive, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from 'element-plus';
import { Plus, Edit, Delete } from '@element-plus/icons-vue';
import { invoke } from '@tauri-apps/api/core';

const { t } = useI18n();

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

const models = ref<LLMModel[]>([]);
const providers = ref<ProviderOption[]>([]);
const remoteModels = ref<RemoteModel[]>([]);
const selectedModelIds = ref<string[]>([]);

const dialogVisible = ref(false);
const isEditing = ref(false);
const saving = ref(false);
const fetchingModels = ref(false);
const formRef = ref<FormInstance>();
const editFormRef = ref<FormInstance>();
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
    remoteModels.value = [];
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
    remoteModels.value = [];
    selectedModelIds.value = [];
};

const loadModels = async () => {
    try {
        models.value = await invoke<LLMModel[]>('get_llm_models');
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

onMounted(() => {
    loadModels();
    loadProviders();
});

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
    remoteModels.value = [];
    selectedModelIds.value = [];
    dialogVisible.value = true;
};

const fetchModels = async () => {
    if (!formData.provider || !formData.api_key) {
        ElMessage.warning(t('llm.providerKeyRequired'));
        return;
    }
    fetchingModels.value = true;
    remoteModels.value = [];
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
        remoteModels.value = list;
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
            const index = models.value.findIndex((m) => m.id === editingModelId.value);
            if (index !== -1) models.value[index] = saved;
            ElMessage.success(t('llm.updateSuccess'));
            dialogVisible.value = false;
        } else {
            const promises = selectedModelIds.value.map((modelId) => {
                const remote = remoteModels.value.find((r) => r.id === modelId);
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
            models.value.push(...results);
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
        models.value = models.value.filter((m) => m.id !== model.id);
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
</script>

<style scoped>
.llm-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
}

.llm-header .section-heading {
    margin-bottom: 0;
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
</style>
