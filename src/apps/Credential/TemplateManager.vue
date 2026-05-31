<template>
    <el-dialog
        v-model="visible"
        :title="t('credential.detail.templateManager')"
        width="800"
        append-to-body
        destroy-on-close
        class="template-manager-dialog">
        <div class="template-list">
            <div v-for="(template, index) in templates" :key="template.value" class="template-item">
                <div class="template-info">
                    <div class="template-header">
                        <span class="template-label">{{ template.label }}</span>
                        <el-button-group size="small">
                            <el-button @click="editTemplate(index)">
                                <el-icon><Edit /></el-icon>
                            </el-button>
                            <el-button type="danger" @click="deleteTemplate(index)">
                                <el-icon><Delete /></el-icon>
                            </el-button>
                        </el-button-group>
                    </div>
                    <div class="template-desc">{{ template.description }}</div>
                    <div class="template-fields">
                        <span class="field-label">{{ t('credential.detail.fieldsIncluded') }}</span>
                        <el-tag
                            v-for="field in getTemplateFields(template.value)"
                            :key="field"
                            size="small"
                            class="mr-1">
                            {{ getFieldLabel(field) }}
                        </el-tag>
                    </div>
                </div>
            </div>
        </div>

        <el-button type="primary" @click="addTemplate" class="add-template-btn">
            <el-icon><Plus /></el-icon>
            {{ t('credential.detail.addTemplate') }}
        </el-button>

        <!-- Edit/Add Template Dialog -->
        <el-dialog
            v-model="showEditDialog"
            :title="isEditMode ? t('credential.detail.editTemplate') : t('credential.detail.addTemplate')"
            width="600"
            append-to-body
            destroy-on-close>
            <el-form ref="formRef" :model="editingTemplate" label-width="100px">
                <el-form-item :label="t('credential.detail.templateKey')">
                    <el-input
                        v-model="editingTemplate.value"
                        :disabled="isEditMode"
                        placeholder="例如: my_custom_type" />
                </el-form-item>
                <el-form-item :label="t('credential.detail.templateName')">
                    <el-input v-model="editingTemplate.label" :placeholder="t('credential.detail.templateName')" />
                </el-form-item>
                <el-form-item :label="t('credential.detail.templateDescription')">
                    <el-input
                        v-model="editingTemplate.description"
                        type="textarea"
                        :rows="2"
                        :placeholder="t('credential.detail.templateDescription')" />
                </el-form-item>
                <el-form-item :label="t('credential.detail.templateFields')">
                    <el-checkbox-group v-model="editingTemplate.fields">
                        <el-checkbox label="password">{{ t('credential.form.passwordLabel') }}</el-checkbox>
                        <el-checkbox label="api_key">{{ t('credential.form.keyLabel') }}</el-checkbox>
                        <el-checkbox label="secret_key">{{ t('credential.form.secretLabel') }}</el-checkbox>
                        <el-checkbox label="expires_at">{{ t('credential.form.expiresAtLabel') }}</el-checkbox>
                        <el-checkbox label="access_token">{{ t('credential.detail.accessToken') }}</el-checkbox>
                        <el-checkbox label="refresh_token">{{ t('credential.detail.refreshToken') }}</el-checkbox>
                    </el-checkbox-group>
                </el-form-item>
            </el-form>

            <template #footer>
                <el-button @click="showEditDialog = false">{{ t('app.cancel') }}</el-button>
                <el-button type="primary" @click="saveTemplate">{{ t('app.save') }}</el-button>
            </template>
        </el-dialog>
    </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { Edit, Delete, Plus } from '@element-plus/icons-vue';
import { ElMessage, ElMessageBox } from 'element-plus';

const { t } = useI18n();

// 定义模板类型
export interface CredentialTemplateOption {
    value: string;
    label: string;
    description: string;
    fields?: string[];
}

// Props
const props = defineProps<{
    modelValue: boolean;
    templates: CredentialTemplateOption[];
}>();

// Emits
const emit = defineEmits<{
    (e: 'update:modelValue', val: boolean): void;
    (e: 'update:templates', templates: CredentialTemplateOption[]): void;
}>();

// State
const visible = ref(props.modelValue);
const showEditDialog = ref(false);
const isEditMode = ref(false);
const editIndex = ref(-1);

const editingTemplate = reactive<{
    value: string;
    label: string;
    description: string;
    fields: string[];
}>({
    value: '',
    label: '',
    description: '',
    fields: [],
});

// 字段标签映射
const fieldLabels: Record<string, string> = {
    password: '密码',
    api_key: 'API Key',
    secret_key: 'Secret',
    expires_at: '到期时间',
    access_token: '访问令牌',
    refresh_token: '刷新令牌',
};

// Methods
const getFieldLabel = (field: string): string => {
    return fieldLabels[field] || field;
};

const getTemplateFields = (value: string): string[] => {
    const template = props.templates.find((t) => t.value === value);
    if (!template) return [];

    // 如果模板已定义字段，返回它们
    if (template.fields && template.fields.length > 0) {
        return template.fields;
    }

    // 否则根据模板类型返回默认字段
    switch (value) {
        case 'account':
            return ['password'];
        case 'api_key':
            return ['api_key'];
        case 'key_secret':
            return ['api_key', 'secret_key'];
        case 'expiring_key':
            return ['api_key', 'expires_at'];
        case 'custom':
            return ['password', 'api_key', 'secret_key', 'expires_at', 'access_token', 'refresh_token'];
        default:
            return [];
    }
};

const addTemplate = () => {
    isEditMode.value = false;
    Object.assign(editingTemplate, {
        value: '',
        label: '',
        description: '',
        fields: [],
    });
    showEditDialog.value = true;
};

const editTemplate = (index: number) => {
    isEditMode.value = true;
    editIndex.value = index;
    const template = props.templates[index];
    Object.assign(editingTemplate, {
        value: template.value,
        label: template.label,
        description: template.description,
        fields: template.fields || getTemplateFields(template.value),
    });
    showEditDialog.value = true;
};

const saveTemplate = () => {
    if (!editingTemplate.value.trim()) {
        ElMessage.warning(t('credential.detail.enterTemplateKey'));
        return;
    }
    if (!editingTemplate.label.trim()) {
        ElMessage.warning(t('credential.detail.enterTemplateName'));
        return;
    }

    const newTemplate: CredentialTemplateOption = {
        value: editingTemplate.value,
        label: editingTemplate.label,
        description: editingTemplate.description,
        fields: editingTemplate.fields,
    };

    const newTemplates = [...props.templates];
    if (isEditMode.value) {
        newTemplates[editIndex.value] = newTemplate;
    } else {
        // 检查是否已存在
        if (newTemplates.some((t) => t.value === newTemplate.value)) {
            ElMessage.warning(t('credential.detail.templateKeyExists'));
            return;
        }
        newTemplates.push(newTemplate);
    }

    emit('update:templates', newTemplates);
    showEditDialog.value = false;
    ElMessage.success(isEditMode.value ? t('credential.detail.templateUpdated') : t('credential.detail.templateAdded'));
};

const deleteTemplate = async (index: number) => {
    const template = props.templates[index];

    try {
        await ElMessageBox.confirm(
            t('credential.detail.confirmDeleteTemplate', { name: template.label }),
            t('credential.detail.deleteTemplate'),
            {
                confirmButtonText: t('app.confirm'),
                cancelButtonText: t('app.cancel'),
                type: 'warning',
            },
        );

        const newTemplates = props.templates.filter((_, i) => i !== index);
        emit('update:templates', newTemplates);
        ElMessage.success(t('credential.detail.templateDeleted'));
    } catch {
        // 用户取消
    }
};

// Watch props变化
watch(
    () => props.modelValue,
    (val) => {
        visible.value = val;
    },
);

watch(visible, (val) => {
    emit('update:modelValue', val);
});
</script>

<style scoped>
.template-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-height: 500px;
    overflow-y: auto;
}

.template-item {
    padding: 16px;
    border: 1px solid var(--color-input-border);
    border-radius: 8px;
    background-color: var(--color-input-bg);
    transition: all 0.2s;
}

.template-item:hover {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.template-info {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.template-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.template-label {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary);
}

.template-desc {
    font-size: 13px;
    color: var(--color-text-secondary);
    line-height: 1.5;
}

.template-fields {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
}

.field-label {
    font-size: 12px;
    color: var(--color-text-tertiary);
}

.add-template-btn {
    width: 100%;
    margin-top: 16px;
}
</style>
