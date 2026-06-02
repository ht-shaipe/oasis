<template>
    <MacWindow
        :title="t('llm.title')"
        :isMinimized="isMinimized"
        @close="handleClose"
        @minimize="emit('minimize')"
        width="1200"
        height="700">
        <div class="flex flex-col h-full min-h-0 bg-[var(--color-sidebar-bg)] font-sans">
            <!-- Header -->
            <div class="flex items-center justify-between px-4 py-3 border-b border-[var(--color-window-titlebar-border)]">
                <h3 class="text-lg font-semibold">{{ t('llm.modelConfig') }}</h3>
                <div class="flex gap-2">
                    <el-button type="primary" size="small" @click="openAddDialog">
                        <el-icon><Plus /></el-icon>
                        {{ t('llm.addModel') }}
                    </el-button>
                </div>
            </div>

            <!-- Model List -->
            <div class="flex-1 overflow-hidden p-4">
                <el-table :data="models" height="100%" style="width: 100%" v-loading="loading">
                    <el-table-column :label="t('llm.modelName')" prop="name" min-width="150" />
                    <el-table-column :label="t('llm.provider')" prop="provider" width="120">
                        <template #default="{ row }">
                            <el-tag :type="getProviderTagType(row.provider)" size="small">
                                {{ getProviderLabel(row.provider) }}
                            </el-tag>
                        </template>
                    </el-table-column>
                    <el-table-column :label="t('llm.modelId')" prop="model_id" min-width="200" />
                    <el-table-column :label="t('llm.baseUrl')" prop="base_url" min-width="200" show-overflow-tooltip />
                    <el-table-column :label="t('llm.status')" width="80">
                        <template #default="{ row }">
                            <el-switch
                                v-model="row.enabled"
                                @change="toggleModelStatus(row)"
                                :loading="row.statusLoading" />
                        </template>
                    </el-table-column>
                    <el-table-column :label="t('llm.actions')" width="120" fixed="right">
                        <template #default="{ row }">
                            <el-button link size="small" @click="openEditDialog(row)">
                                <el-icon><Edit /></el-icon>
                            </el-button>
                            <el-button link size="small" type="danger" @click="deleteModel(row)">
                                <el-icon><Delete /></el-icon>
                            </el-button>
                        </template>
                    </el-table-column>
                </el-table>
            </div>
        </div>

        <!-- Add/Edit Dialog -->
        <el-dialog
            v-model="showDialog"
            :title="isEditing ? t('llm.editModel') : t('llm.addModel')"
            width="600px"
            :close-on-click-modal="false">
            <el-form :model="formData" :rules="formRules" ref="formRef" label-width="120px">
                <el-form-item :label="t('llm.modelName')" prop="name">
                    <el-input v-model="formData.name" :placeholder="t('llm.modelNamePlaceholder')" />
                </el-form-item>
                <el-form-item :label="t('llm.provider')" prop="provider">
                    <el-select v-model="formData.provider" :placeholder="t('llm.selectProvider')" style="width: 100%">
                        <el-option value="openai" label="OpenAI" />
                        <el-option value="anthropic" label="Anthropic" />
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
                <el-form-item :label="t('llm.apiKey')" prop="api_key">
                    <el-input
                        v-model="formData.api_key"
                        type="password"
                        show-password
                        :placeholder="t('llm.apiKeyPlaceholder')" />
                </el-form-item>
                <el-form-item :label="t('llm.temperature')">
                    <el-slider v-model="formData.temperature" :min="0" :max="2" :step="0.1" show-input />
                </el-form-item>
                <el-form-item :label="t('llm.maxTokens')">
                    <el-input-number v-model="formData.max_tokens" :min="1" :max="128000" />
                </el-form-item>
                <el-form-item :label="t('llm.description')">
                    <el-input
                        v-model="formData.description"
                        type="textarea"
                        :rows="3"
                        :placeholder="t('llm.descriptionPlaceholder')" />
                </el-form-item>
            </el-form>
            <template #footer>
                <el-button @click="showDialog = false">{{ t('app.cancel') }}</el-button>
                <el-button type="primary" @click="saveModel" :loading="saving">
                    {{ t('app.save') }}
                </el-button>
            </template>
        </el-dialog>
    </MacWindow>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';
import { useI18n } from 'vue-i18n';
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from 'element-plus';
import { Plus, Edit, Delete } from '@element-plus/icons-vue';
import MacWindow from '@/components/common/MacWindow.vue';

const { t } = useI18n();

const props = defineProps<{ isMinimized: boolean }>();
const emit = defineEmits<{
    (e: 'close'): void;
    (e: 'minimize'): void;
}>();

interface LLMModel {
    id?: number;
    name: string;
    provider: string;
    model_id: string;
    base_url: string;
    api_key: string;
    temperature: number;
    max_tokens: number;
    description: string;
    enabled: boolean;
    statusLoading?: boolean;
}

const loading = ref(false);
const saving = ref(false);
const showDialog = ref(false);
const isEditing = ref(false);
const formRef = ref<FormInstance>();

const models = ref<LLMModel[]>([
    {
        id: 1,
        name: 'GPT-4',
        provider: 'openai',
        model_id: 'gpt-4',
        base_url: 'https://api.openai.com/v1',
        api_key: 'sk-xxxxxxxxxxxx',
        temperature: 0.7,
        max_tokens: 4096,
        description: 'OpenAI GPT-4 model',
        enabled: true,
    },
    {
        id: 2,
        name: 'Claude 3 Opus',
        provider: 'anthropic',
        model_id: 'claude-3-opus-20240229',
        base_url: 'https://api.anthropic.com',
        api_key: 'sk-ant-xxxxxxxxxxxx',
        temperature: 0.7,
        max_tokens: 4096,
        description: 'Anthropic Claude 3 Opus',
        enabled: true,
    },
]);

const formData = reactive<LLMModel>({
    name: '',
    provider: 'openai',
    model_id: '',
    base_url: '',
    api_key: '',
    temperature: 0.7,
    max_tokens: 4096,
    description: '',
    enabled: true,
});

const formRules: FormRules = {
    name: [{ required: true, message: t('llm.modelNameRequired'), trigger: 'blur' }],
    provider: [{ required: true, message: t('llm.providerRequired'), trigger: 'change' }],
    model_id: [{ required: true, message: t('llm.modelIdRequired'), trigger: 'blur' }],
    base_url: [{ required: true, message: t('llm.baseUrlRequired'), trigger: 'blur' }],
    api_key: [{ required: true, message: t('llm.apiKeyRequired'), trigger: 'blur' }],
};

const getProviderLabel = (provider: string): string => {
    const labels: Record<string, string> = {
        openai: 'OpenAI',
        anthropic: 'Anthropic',
        azure: 'Azure',
        custom: 'Custom',
    };
    return labels[provider] || provider;
};

const getProviderTagType = (provider: string): string => {
    const types: Record<string, string> = {
        openai: 'success',
        anthropic: 'warning',
        azure: 'info',
        custom: 'info',
    };
    return types[provider] || 'info';
};

const openAddDialog = () => {
    isEditing.value = false;
    Object.assign(formData, {
        name: '',
        provider: 'openai',
        model_id: '',
        base_url: '',
        api_key: '',
        temperature: 0.7,
        max_tokens: 4096,
        description: '',
        enabled: true,
    });
    showDialog.value = true;
};

const openEditDialog = (model: LLMModel) => {
    isEditing.value = true;
    Object.assign(formData, { ...model });
    showDialog.value = true;
};

const saveModel = async () => {
    if (!formRef.value) return;

    await formRef.value.validate(async (valid) => {
        if (!valid) return;

        saving.value = true;
        try {
            if (isEditing.value) {
                const index = models.value.findIndex((m) => m.id === formData.id);
                if (index !== -1) {
                    models.value[index] = { ...formData };
                }
                ElMessage.success(t('llm.updateSuccess'));
            } else {
                models.value.push({
                    ...formData,
                    id: Date.now(),
                });
                ElMessage.success(t('llm.addSuccess'));
            }
            showDialog.value = false;
        } finally {
            saving.value = false;
        }
    });
};

const deleteModel = async (model: LLMModel) => {
    try {
        await ElMessageBox.confirm(t('llm.deleteConfirm'), t('llm.deleteModel'), {
            type: 'warning',
        });
        const index = models.value.findIndex((m) => m.id === model.id);
        if (index !== -1) {
            models.value.splice(index, 1);
            ElMessage.success(t('llm.deleteSuccess'));
        }
    } catch {
        // User cancelled
    }
};

const toggleModelStatus = (model: LLMModel) => {
    model.statusLoading = true;
    setTimeout(() => {
        model.statusLoading = false;
        ElMessage.success(model.enabled ? t('llm.enabledSuccess') : t('llm.disabledSuccess'));
    }, 500);
};

const handleClose = () => {
    emit('close');
};
</script>