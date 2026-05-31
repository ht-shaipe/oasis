<template>
    <el-dialog
        :model-value="modelValue"
        @update:model-value="$emit('update:modelValue', $event)"
        :title="dialogTitle"
        width="800"
        append-to-body
        destroy-on-close
        class="credential-form-dialog">
        <el-form ref="credFormRef" :model="credForm" @submit.prevent="handleSaveCredential" class="credential-form">
            <!-- Basic info -->
            <h4
                class="text-sm font-600 text-[var(--color-text-primary)] mb-3 mt-0 first:mt-0 pb-1.5 border-b border-solid border-0 border-[var(--color-window-titlebar-border)]">
                {{ t('credential.detail.basicInfo') }}
            </h4>

            <div class="grid grid-cols-2 gap-4">
                <el-form-item :label="t('credential.detail.credentialType')" class="col-span-2">
                    <div class="credential-type-group">
                        <el-select
                            v-model="credForm.credential_type"
                            class="credential-type-select"
                            popper-class="credential-type-popper"
                            @change="handleCredentialTypeChange">
                            <el-option
                                v-for="option in allTemplates"
                                :key="option.value"
                                :label="option.label"
                                :value="option.value">
                                <div class="flex flex-col py-1">
                                    <span class="text-sm font-500 text-[var(--color-text-primary)]">{{
                                        option.label
                                    }}</span>
                                    <span class="text-xs text-[var(--color-text-secondary)]">{{
                                        option.description
                                    }}</span>
                                </div>
                            </el-option>
                        </el-select>
                        <el-button class="credential-type-action" @click="showTemplateManager = true" title="管理模板">
                            <el-icon><Setting /></el-icon>
                        </el-button>
                    </div>
                </el-form-item>

                <el-form-item :label="t('credential.list.title')" required>
                    <el-input v-model="credForm.title" />
                </el-form-item>

                <el-form-item :label="t('credential.list.category')" required>
                    <el-select
                        v-model="credForm.category_id"
                        :placeholder="t('credential.list.category')"
                        class="w-full">
                        <el-option v-for="cat in categories" :key="cat.id" :label="cat.name" :value="cat.id">
                            <span :style="{ paddingLeft: cat.level * 20 + 'px' }">{{ cat.name }}</span>
                        </el-option>
                    </el-select>
                </el-form-item>

                <el-form-item :label="t('credential.list.username')">
                    <el-input v-model="credForm.username" :placeholder="t('credential.form.usernameHint')" />
                </el-form-item>

                <el-form-item :label="t('credential.list.url')">
                    <el-input v-model="credForm.url" :placeholder="t('credential.form.urlHint')" />
                </el-form-item>

                <el-form-item :label="t('credential.detail.tags')">
                    <el-input v-model="credForm.tags" :placeholder="t('credential.detail.tags')" />
                </el-form-item>

                <el-form-item :label="t('credential.detail.notes')" class="col-span-2">
                    <el-input v-model="credForm.notes" type="textarea" :rows="2" />
                </el-form-item>
            </div>

            <!-- Sensitive info -->
            <h4
                class="text-sm font-600 text-[var(--color-text-primary)] mb-3 mt-5 pb-1.5 border-b border-solid border-0 border-[var(--color-window-titlebar-border)]">
                {{ t('credential.detail.sensitiveInfo') }}
            </h4>

            <div class="grid grid-cols-2 gap-4">
                <template v-for="field in sensitiveFieldDefs" :key="field.key">
                    <el-form-item
                        v-if="shouldShowField(credForm.credential_type, field.key)"
                        :label="t(field.labelKey)">
                        <el-date-picker
                            v-if="field.isDatePicker"
                            v-model="credForm.sensitive[field.key]"
                            class="w-full"
                            type="datetime"
                            :placeholder="t(field.hintKey)"
                            value-format="YYYY-MM-DD HH:mm:ss" />
                        <div v-else class="flex items-center gap-1 w-full">
                            <el-input
                                v-model="credForm.sensitive[field.key]"
                                :placeholder="t(field.hintKey)"
                                :type="visibleFields[field.key] ? 'text' : 'password'"
                                class="flex-1" />
                            <el-button link @click="visibleFields[field.key] = !visibleFields[field.key]">
                                <el-icon>
                                    <component :is="visibleFields[field.key] ? Hide : View" />
                                </el-icon>
                            </el-button>
                            <el-button link @click="copyToClipboard(credForm.sensitive[field.key])">
                                <el-icon><CopyDocument /></el-icon>
                            </el-button>
                        </div>
                    </el-form-item>
                </template>
            </div>

            <!-- Custom fields -->
            <h4
                class="text-sm font-600 text-[var(--color-text-primary)] mb-3 mt-5 pb-1.5 border-b border-solid border-0 border-[var(--color-window-titlebar-border)]">
                {{ t('credential.detail.customFields') }}
            </h4>

            <div v-for="(field, index) in customFields" :key="index" class="flex items-center gap-2 mb-2.5">
                <el-input v-model="field.key" placeholder="Key" class="w-35 flex-shrink-0" />
                <div class="flex items-center gap-1 flex-1">
                    <el-input
                        v-model="field.value"
                        :type="field.visible ? 'text' : 'password'"
                        placeholder="Value"
                        class="flex-1" />
                    <el-button link @click="field.visible = !field.visible">
                        <el-icon><component :is="field.visible ? Hide : View" /></el-icon>
                    </el-button>
                </div>
                <el-button link type="danger" @click="customFields.splice(index, 1)">
                    <el-icon><Delete /></el-icon>
                </el-button>
            </div>
            <el-button text @click="customFields.push({ key: '', value: '', visible: false })">
                <el-icon><Plus /></el-icon>
                {{ t('credential.detail.addField') }}
            </el-button>
            <!-- Hidden submit button to enable Enter key submission -->
            <button type="submit" class="hidden" />
        </el-form>

        <template #footer>
            <el-button @click="$emit('update:modelValue', false)">
                {{ t('credential.detail.cancel') }}
            </el-button>
            <el-button type="primary" :loading="credSaving" @click="handleSaveCredential">
                {{ t('credential.detail.save') }}
            </el-button>
        </template>
    </el-dialog>

    <!-- Template Manager -->
    <TemplateManager
        v-model="showTemplateManager"
        :templates="allTemplates"
        @update:templates="handleTemplatesUpdate" />
</template>

<script setup lang="ts">
import { ref, reactive, watch, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { ElMessage } from 'element-plus';
import { View, Hide, Delete, CopyDocument, Plus, Setting } from '@element-plus/icons-vue';
import {
    buildSensitiveData,
    defaultCredentialForm,
    credentialTemplateOptions,
    defaultCredentialTemplateOptions,
    saveCredentialTemplates,
    inferCredentialType,
    normalizeSensitiveFields,
    shouldShowField,
    type CredentialFormModel,
    type CredentialTemplateKey,
    type CredentialTemplateOption,
} from './credentialForm';
import TemplateManager from './TemplateManager.vue';
import {
    useCredential,
    type CredentialView,
    type CredentialDetail,
    type SensitiveData,
} from '@/composables/useCredential';

const { t } = useI18n();
const { lock, getCredential, createCredential, updateCredential } = useCredential();

// ── Computed Properties ──

const dialogTitle = computed(() =>
    isEditMode.value ? t('credential.detail.editTitle') : t('credential.detail.createTitle'),
);

const allTemplates = computed(() => {
    return [...credentialTemplateOptions, ...customTemplates.value];
});

// ── Props / Emits ──

const props = defineProps<{
    modelValue: boolean;
    categories: Array<{ id: number; name: string; level: number }>;
    dek?: string | null;
    editingCredential?: CredentialView | CredentialDetail | null;
}>();

const emit = defineEmits<{
    (e: 'update:modelValue', val: boolean): void;
    (e: 'saved'): void;
}>();

// ── State ──

const isEditMode = ref(false);
const editingCredId = ref<number | null>(null);
const credSaving = ref(false);

// 模板管理
const showTemplateManager = ref(false);
const customTemplates = ref<Array<CredentialTemplateOption>>([]);

const credForm = reactive<CredentialFormModel>(defaultCredentialForm(null));

const customFields = ref<Array<{ key: string; value: string; visible: boolean }>>([]);

const visibleFields = reactive<Record<string, boolean>>({
    password: false,
    api_key: false,
    secret_key: false,
    access_token: false,
    refresh_token: false,
});

interface SensitiveFieldDef {
    key: keyof CredentialFormModel['sensitive'];
    labelKey: string;
    hintKey: string;
    isDatePicker?: boolean;
}

const sensitiveFieldDefs: SensitiveFieldDef[] = [
    { key: 'password', labelKey: 'credential.form.passwordLabel', hintKey: 'credential.form.passwordHint' },
    { key: 'api_key', labelKey: 'credential.form.keyLabel', hintKey: 'credential.form.keyHint' },
    { key: 'secret_key', labelKey: 'credential.form.secretLabel', hintKey: 'credential.form.secretHint' },
    {
        key: 'expires_at',
        labelKey: 'credential.form.expiresAtLabel',
        hintKey: 'credential.form.expiresAtHint',
        isDatePicker: true,
    },
    { key: 'access_token', labelKey: 'credential.detail.accessToken', hintKey: 'credential.form.accessTokenHint' },
    { key: 'refresh_token', labelKey: 'credential.detail.refreshToken', hintKey: 'credential.form.refreshTokenHint' },
];

// ── Methods ──

const handleCredentialTypeChange = () => {
    normalizeSensitiveFields(credForm);
};

const resetCredForm = () => {
    const defaultCategoryId = props.categories.length > 0 ? props.categories[0].id : null;
    Object.assign(credForm, defaultCredentialForm(defaultCategoryId));
    customFields.value = [];
    for (const key of Object.keys(visibleFields)) {
        visibleFields[key] = false;
    }
};

const populateFromCredential = async (row: CredentialView | CredentialDetail) => {
    credForm.title = row.title;
    credForm.category_id = row.category_id;
    credForm.username = row.username || '';
    credForm.url = row.url || '';
    credForm.tags = row.tags || '';
    credForm.notes = row.notes || '';

    const detail = row as CredentialDetail;
    if (detail.sensitive_data) {
        fillSensitiveData(detail.sensitive_data);
    } else {
        try {
            const d = await getCredential(row.id);
            fillSensitiveData(d.sensitive_data);
        } catch (err: unknown) {
            const errorMsg = err instanceof Error ? err.message : String(err);
            if (errorMsg === 'Vault is locked') {
                ElMessage.warning('保险库已锁定，请重新输入密码');
                lock();
                emit('update:modelValue', false);
                return;
            }
            ElMessage.error('获取凭证详情失败，敏感字段将留空');
        }
    }
};

const fillSensitiveData = (sensitiveData: Partial<SensitiveData>) => {
    credForm.credential_type =
        (sensitiveData.credential_type as CredentialTemplateKey) ?? inferCredentialType(sensitiveData);
    credForm.sensitive.password = sensitiveData.password || '';
    credForm.sensitive.api_key = sensitiveData.api_key || '';
    credForm.sensitive.secret_key = sensitiveData.secret_key || '';
    credForm.sensitive.expires_at = sensitiveData.expires_at || '';
    credForm.sensitive.access_token = sensitiveData.access_token || '';
    credForm.sensitive.refresh_token = sensitiveData.refresh_token || '';
    if (sensitiveData.custom_fields) {
        customFields.value = Object.entries(sensitiveData.custom_fields).map(([key, value]) => ({
            key,
            value,
            visible: false,
        }));
    }
};

const handleSaveCredential = async () => {
    if (!credForm.title.trim()) {
        ElMessage.warning(t('credential.list.title'));
        return;
    }

    if (!credForm.category_id || credForm.category_id <= 0) {
        ElMessage.warning('请选择一个分类');
        return;
    }

    normalizeSensitiveFields(credForm);
    const sensitiveData: SensitiveData = buildSensitiveData(credForm);
    const sensitiveDataJson = JSON.stringify(sensitiveData);

    const customObj: Record<string, string> = {};
    for (const f of customFields.value) {
        if (f.key.trim()) customObj[f.key.trim()] = f.value;
    }
    if (Object.keys(customObj).length > 0) {
        sensitiveData.custom_fields = customObj;
    }

    credSaving.value = true;
    try {
        if (isEditMode.value && editingCredId.value !== null) {
            await updateCredential({
                id: editingCredId.value,
                category_id: credForm.category_id ?? undefined,
                title: credForm.title,
                username: credForm.username || undefined,
                url: credForm.url || undefined,
                sensitive_data_json: sensitiveDataJson,
                dekBase64: props.dek ?? '',
                nonceBase64: '',
                tags: credForm.tags || undefined,
                notes: credForm.notes || undefined,
            });
        } else {
            await createCredential({
                category_id: credForm.category_id!,
                title: credForm.title,
                username: credForm.username || undefined,
                url: credForm.url || undefined,
                sensitive_data_json: sensitiveDataJson,
                dekBase64: props.dek ?? '',
                nonceBase64: '',
                tags: credForm.tags || undefined,
                notes: credForm.notes || undefined,
            });
        }
        emit('update:modelValue', false);
        emit('saved');
    } catch (err: unknown) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        if (errorMsg === 'Vault is locked') {
            ElMessage.warning('保险库已锁定，请重新输入密码');
            lock();
            emit('update:modelValue', false);
        } else {
            ElMessage.error(errorMsg);
        }
    } finally {
        credSaving.value = false;
    }
};

const copyToClipboard = async (text: string | undefined) => {
    if (!text) return;
    try {
        await navigator.clipboard.writeText(text);
        ElMessage.success(t('credential.detail.copied'));
    } catch {
        ElMessage.error(t('credential.detail.copy'));
    }
};

// ── Template Management ──

const loadCustomTemplates = () => {
    const saved = localStorage.getItem('credential_templates');
    if (saved) {
        try {
            customTemplates.value = JSON.parse(saved);
        } catch (error) {
            console.error('Failed to load custom templates:', error);
            customTemplates.value = [];
        }
    }
};

const handleTemplatesUpdate = (templates: Array<CredentialTemplateOption>) => {
    // 分离默认模板和自定义模板
    const defaultValues = new Set(defaultCredentialTemplateOptions.map((t) => t.value));
    customTemplates.value = templates.filter((t) => !defaultValues.has(t.value));

    // 保存到localStorage
    saveCredentialTemplates(customTemplates.value);

    // 如果当前模板被删除了，重置为默认
    if (!templates.find((t) => t.value === credForm.credential_type)) {
        credForm.credential_type = 'account';
        normalizeSensitiveFields(credForm);
    }

    ElMessage.success('模板已更新');
};

// ── Open dialog logic ──

const open = () => {
    if (props.editingCredential) {
        isEditMode.value = true;
        editingCredId.value = props.editingCredential.id;
        resetCredForm();
        populateFromCredential(props.editingCredential);
    } else {
        isEditMode.value = false;
        editingCredId.value = null;
        resetCredForm();
        if (props.categories.length > 0) {
            credForm.category_id = props.categories[0].id;
        }
    }
};

watch(
    () => props.modelValue,
    (val) => {
        if (val) open();
    },
);

// 加载自定义模板
onMounted(() => {
    loadCustomTemplates();
});
</script>

<style scoped>
/* === Dialog Styling === */
.credential-form-dialog :deep(.el-overlay) {
    z-index: var(--z-index-modal);
}

.credential-form-dialog :deep(.el-dialog) {
    border-radius: 12px;
    overflow: hidden;
}

.credential-form-dialog :deep(.el-dialog__header) {
    padding: 20px 24px 16px;
    border-bottom: 1px solid var(--color-window-titlebar-border);
    background-color: var(--color-bg-glass);
}

.credential-form-dialog :deep(.el-dialog__title) {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary);
}

.credential-form-dialog :deep(.el-dialog__body) {
    padding: 24px;
    max-height: 70vh;
    overflow-y: auto;
}

.credential-form-dialog :deep(.el-dialog__footer) {
    padding: 16px 24px;
    border-top: 1px solid var(--color-window-titlebar-border);
    background-color: var(--color-bg-glass);
}

/* === Form Styling === */
.credential-form {
    margin: 0;
}

.credential-form :deep(.el-form-item) {
    margin-bottom: 0px;
}

.credential-form :deep(.el-form-item__label) {
    font-weight: 500;
    color: var(--color-text-primary);
    font-size: 13px;
    line-height: 1.5;
    width: 80px;
}

/* === Input Styling === */
.credential-form :deep(.el-input__wrapper) {
    /* border-radius: 6px; */
    transition: all 0.2s ease;
}

.credential-type-group {
    display: flex;
    align-items: stretch;
    width: 100%;
    min-width: 0;
    border: 1px solid var(--color-input-border);
    border-radius: 10px;
    overflow: hidden;
    background: var(--color-bg-page);
    transition:
        border-color 0.2s ease,
        box-shadow 0.2s ease;
}

.credential-type-group:focus-within {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-primary) 18%, transparent);
}

.credential-type-select {
    flex: 1;
    min-width: 0;
}

.credential-type-group :deep(.el-input__wrapper) {
    border-radius: 0 !important;
    border: 0 !important;
    box-shadow: none !important;
    background: transparent !important;
}

.credential-type-group :deep(.el-select__wrapper) {
    border: 0 !important;
    box-shadow: none !important;
    background: transparent !important;
}

.credential-type-group :deep(.el-input__wrapper.is-focus),
.credential-type-group :deep(.el-select__wrapper.is-focus),
.credential-type-group :deep(.el-input__wrapper:hover),
.credential-type-group :deep(.el-select__wrapper:hover) {
    border-color: transparent !important;
    box-shadow: none !important;
}

.credential-type-group :deep(.el-input__inner) {
    padding-left: 12px;
}

.credential-type-action {
    flex: 0 0 56px;
    width: 56px;
    min-width: 56px;
    border: 0;
    border-left: 1px solid var(--color-input-border);
    border-radius: 0;
    background: var(--color-bg-page);
}

.credential-type-action:focus,
.credential-type-action:focus-visible {
    outline: none;
    border-left-color: var(--color-primary);
    box-shadow: none;
}

.credential-type-action:hover {
    background: color-mix(in srgb, var(--color-primary) 8%, var(--color-bg-page));
}

.credential-type-popper {
    min-width: 520px !important;
}

:global(.credential-type-popper .el-select-dropdown__item) {
    min-height: 66px !important;
    height: auto !important;
    padding: 6px 16px !important;
    display: flex;
    align-items: flex-start;
}

:global(.credential-type-popper .el-select-dropdown__item .flex) {
    width: 100%;
}

:global(.credential-type-popper .el-select-dropdown__item .text-sm) {
    line-height: 1.5;
}

:global(.credential-type-popper .el-select-dropdown__item .text-xs) {
    line-height: 1.4;
    margin-top: 4px;
}

/* === Buttons === */
.credential-form :deep(.el-button.is-text) {
    padding: 8px 12px;
}

.credential-form-dialog :deep(.el-dialog__footer .el-button) {
    padding: 8px 16px;
}

/* === Date Picker === */
.credential-form :deep(.el-date-editor) {
    width: 100%;
}

/* === Responsive === */
@media (max-width: 768px) {
    .credential-form-dialog :deep(.el-dialog) {
        width: 95% !important;
        margin: 0 auto;
    }
}
</style>
