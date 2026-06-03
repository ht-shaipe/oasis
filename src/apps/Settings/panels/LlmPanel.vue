<template>
    <div class="section-panel">
        <div class="llm-header">
            <h2 class="section-heading">{{ t('settings.llm.title') }}</h2>
            <el-button type="primary" size="small" @click="openAddDialog">
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
                            <span class="llm-provider-label">
                                {{ getProviderLabel(model.provider) }}
                            </span>
                        </div>
                        <div class="llm-row-meta">
                            <span class="llm-meta-item">
                                <span class="llm-meta-label">{{ t('llm.modelId') }}:</span>
                                <span>{{ model.model_id }}</span>
                            </span>
                            <span class="llm-meta-item">
                                <span class="llm-meta-label">{{ t('llm.baseUrl') }}:</span>
                                <span class="llm-base-url">{{ model.base_url }}</span>
                            </span>
                        </div>
                    </div>
                    <div class="llm-row-actions">
                        <el-switch
                            v-model="model.enabled"
                            @change="toggleModelStatus(model)"
                            :loading="model.statusLoading"
                            size="small" />
                        <el-button link size="small" @click="openEditDialog(model)">
                            <el-icon><Edit /></el-icon>
                        </el-button>
                        <el-button link size="small" type="danger" @click="deleteModel(model)">
                            <el-icon><Delete /></el-icon>
                        </el-button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Add/Edit Dialog -->
        <el-dialog
            v-model="dialogVisible"
            :title="isEditing ? t('llm.editModel') : t('llm.addModel')"
            width="540px"
            :close-on-click-modal="false"
            append-to-body>
            <el-form :model="formData" :rules="formRules" ref="formRef" label-width="100px">
                <el-form-item :label="t('llm.modelName')" prop="name">
                    <el-input v-model="formData.name" :placeholder="t('llm.modelNamePlaceholder')" />
                </el-form-item>
                <el-form-item :label="t('llm.provider')" prop="provider">
                    <el-select v-model="formData.provider" :placeholder="t('llm.selectProvider')" style="width: 100%"
                        @change="onProviderChange">
                        <el-option value="openai" label="OpenAI" />
                        <el-option value="anthropic" label="Anthropic" />
                        <el-option value="deepseek" label="DeepSeek" />
                        <el-option value="qwen" label="通义千问" />
                        <el-option value="moonshot" label="Moonshot（月之暗面）" />
                        <el-option value="zhipu" label="智谱 GLM" />
                        <el-option value="baidu" label="百度文心" />
                        <el-option value="azure" label="Azure OpenAI" />
                        <el-option value="custom" label="Custom" />
                    </el-select>
                </el-form-item>
                <el-form-item :label="t('llm.modelId')" prop="model_id">
                    <el-input v-model="formData.model_id" :placeholder="t('llm.modelIdPlaceholder')" />
                </el-form-item>
                <el-form-item :label="t('llm.baseUrl')" prop="base_url">
                    <el-input v-model="formData.base_url" :placeholder="t('llm.baseUrlPlaceholder')" />
                </el-form-item>
                <el-form-item :label="t('llm.authType')" prop="auth_type">
                    <el-radio-group v-model="formData.auth_type">
                        <el-radio value="api_key">{{ t('llm.authApiKey') }}</el-radio>
                        <el-radio value="token_plan">{{ t('llm.authTokenPlan') }}</el-radio>
                    </el-radio-group>
                </el-form-item>
                <el-form-item v-if="formData.auth_type === 'api_key'" :label="t('llm.apiKey')" prop="api_key">
                    <el-input v-model="formData.api_key" :placeholder="t('llm.apiKeyPlaceholder')" type="password" show-password />
                </el-form-item>
                <el-form-item v-else :label="t('llm.tokenPlan')" prop="token_plan">
                    <el-input v-model="formData.token_plan" :placeholder="t('llm.tokenPlanPlaceholder')" />
                </el-form-item>
                <el-form-item :label="t('llm.temperature')">
                    <el-slider v-model="formData.temperature" :min="0" :max="2" :step="0.1" show-input />
                </el-form-item>
                <el-form-item :label="t('llm.maxTokens')">
                    <el-input-number v-model="formData.max_tokens" :min="1" :max="128000" style="width: 100%" />
                </el-form-item>
                <el-form-item :label="t('llm.description')">
                    <el-input v-model="formData.description" type="textarea" :rows="2" :placeholder="t('llm.descriptionPlaceholder')" />
                </el-form-item>
            </el-form>
            <template #footer>
                <el-button @click="dialogVisible = false">{{ t('app.cancel') }}</el-button>
                <el-button type="primary" @click="saveModel" :loading="saving">
                    {{ t('app.save') }}
                </el-button>
            </template>
        </el-dialog>
    </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
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
    statusLoading?: boolean;
}

const models = ref<LLMModel[]>([]);

const dialogVisible = ref(false);
const isEditing = ref(false);
const saving = ref(false);
const formRef = ref<FormInstance>();
const editingModelId = ref<string | undefined>();

const formData = reactive<LLMModel>({
    id: '',
    name: '',
    provider: 'openai',
    model_id: '',
    base_url: 'https://api.openai.com/v1',
    auth_type: 'api_key',
    api_key: '',
    token_plan: '',
    temperature: 0.7,
    max_tokens: 4096,
    description: '',
    enabled: true,
});

const formRules: FormRules = {
    name: [{ required: true, message: '请输入模型名称', trigger: 'blur' }],
    provider: [{ required: true, message: '请选择提供商', trigger: 'change' }],
    model_id: [{ required: true, message: '请输入模型ID', trigger: 'blur' }],
    base_url: [{ required: true, message: '请输入基础URL', trigger: 'blur' }],
    auth_type: [{ required: true, message: '请选择认证方式', trigger: 'change' }],
    api_key: [{ required: true, message: '请输入API密钥', trigger: 'blur' }],
    token_plan: [{ required: true, message: '请输入Token Plan', trigger: 'blur' }],
};

const providerLabels: Record<string, string> = {
    openai: 'OpenAI',
    anthropic: 'Anthropic',
    deepseek: 'DeepSeek',
    qwen: '通义千问',
    moonshot: '月之暗面',
    zhipu: '智谱',
    baidu: '文心',
    azure: 'Azure',
    custom: 'Custom',
};

const providerBaseUrls: Record<string, string> = {
    openai: 'https://api.openai.com/v1',
    anthropic: 'https://api.anthropic.com',
    deepseek: 'https://api.deepseek.com/v1',
    qwen: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    moonshot: 'https://api.moonshot.cn/v1',
    zhipu: 'https://open.bigmodel.cn/api/paas/v4',
    baidu: 'https://qianfan.baidubce.com/v2',
    azure: '',
    custom: '',
};

const getProviderLabel = (provider: string): string => {
    return providerLabels[provider] || provider;
};

const onProviderChange = (provider: string) => {
    formData.base_url = providerBaseUrls[provider] || '';
};

const resetForm = () => {
    Object.assign(formData, {
        id: '',
        name: '',
        provider: 'openai',
        model_id: '',
        base_url: 'https://api.openai.com/v1',
        auth_type: 'api_key',
        api_key: '',
        token_plan: '',
        temperature: 0.7,
        max_tokens: 4096,
        description: '',
        enabled: true,
    });
};

const loadModels = async () => {
    try {
        models.value = await invoke<LLMModel[]>('get_llm_models');
    } catch (e) {
        console.error('Failed to load LLM models:', e);
    }
};

onMounted(loadModels);

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
    dialogVisible.value = true;
};

const saveModel = async () => {
    if (!formRef.value) return;

    await formRef.value.validate(async (valid) => {
        if (!valid) return;

        saving.value = true;
        try {
            const modelToSave = isEditing.value
                ? formData
                : { ...formData, id: crypto.randomUUID() };
            const saved = await invoke<LLMModel>('save_llm_model', { model: modelToSave });
            if (isEditing.value) {
                const index = models.value.findIndex((m) => m.id === editingModelId.value);
                if (index !== -1) models.value[index] = saved;
            } else {
                models.value.push(saved);
            }
            ElMessage.success(isEditing.value ? '模型更新成功' : '模型添加成功');
            dialogVisible.value = false;
        } catch (e: any) {
            ElMessage.error(e?.toString() || '保存失败');
        } finally {
            saving.value = false;
        }
    });
};

const deleteModel = async (model: LLMModel) => {
    try {
        await ElMessageBox.confirm('确定要删除此模型吗？', '删除模型', {
            type: 'warning',
        });
        await invoke('delete_llm_model', { id: model.id });
        models.value = models.value.filter((m) => m.id !== model.id);
        ElMessage.success('模型删除成功');
    } catch (e: any) {
        if (e !== 'cancel') {
            ElMessage.error(e?.toString() || '删除失败');
        }
    }
};

const toggleModelStatus = async (model: LLMModel) => {
    model.statusLoading = true;
    try {
        await invoke('toggle_llm_model', { id: model.id, enabled: model.enabled });
        ElMessage.success(model.enabled ? '模型已启用' : '模型已禁用');
    } catch (e: any) {
        // rollback on failure
        model.enabled = !model.enabled;
        ElMessage.error(e?.toString() || '操作失败');
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
}

.llm-row {
    padding: 14px 0;
    border-bottom: 1px solid var(--color-card-border);
}

.llm-row:last-child {
    border-bottom: none;
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
    align-items: baseline;
    gap: 8px;
    margin-bottom: 4px;
}

.llm-row-name {
    font-size: var(--app-font-13);
    font-weight: 600;
    color: var(--color-text-primary);
}

.llm-provider-label {
    font-size: var(--app-font-12);
    color: var(--color-grey);
    display: flex;
    align-items: center;
    gap: 4px;
}

.llm-provider-label::before {
    content: '';
    display: inline-block;
    width: 1px;
    height: 12px;
    background: var(--color-card-border);
    margin-right: 4px;
    vertical-align: middle;
}

.llm-row-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    font-size: var(--app-font-12);
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

.llm-base-url {
    max-width: 260px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: block;
}

.llm-row-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
}

.llm-empty {
    padding: 40px 0;
    text-align: center;
    font-size: var(--app-font-13);
    color: var(--color-text-tertiary);
}
</style>
