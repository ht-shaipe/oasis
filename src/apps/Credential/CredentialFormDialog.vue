<template>
    <el-dialog
        :model-value="modelValue"
        @update:model-value="$emit('update:modelValue', $event)"
        :title="isEditMode ? t('credential.detail.editTitle') : t('credential.detail.createTitle')"
        width="600"
        append-to-body
        destroy-on-close>
        <el-form ref="credFormRef" :model="credForm" label-position="top" @submit.prevent="handleSaveCredential">
            <!-- Basic info -->
            <h4 class="section-heading">{{ t('credential.detail.basicInfo') }}</h4>

            <div class="grid gap-4 md:grid-cols-2">
                <el-form-item :label="t('credential.detail.credentialType')">
                    <el-select
                        v-model="credForm.credential_type"
                        style="width: 100%"
                        @change="handleCredentialTypeChange">
                        <el-option
                            v-for="option in credentialTemplateOptions"
                            :key="option.value"
                            :label="option.label"
                            :value="option.value">
                            <div class="flex flex-col py-1">
                                <span class="text-sm font-600 text-[var(--color-text-primary)]">{{
                                    option.label
                                }}</span>
                                <span class="text-xs text-[var(--color-text-secondary)]">{{
                                    option.description
                                }}</span>
                            </div>
                        </el-option>
                    </el-select>
                </el-form-item>

                <el-form-item :label="t('credential.list.title')" required>
                    <el-input v-model="credForm.title" />
                </el-form-item>

                <el-form-item :label="t('credential.list.category')" required>
                    <el-select
                        v-model="credForm.category_id"
                        :placeholder="t('credential.list.category')"
                        style="width: 100%">
                        <el-option
                            v-for="cat in categories"
                            :key="cat.id"
                            :label="cat.name"
                            :value="cat.id">
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

                <el-form-item :label="t('credential.detail.notes')" class="md:col-span-2">
                    <el-input v-model="credForm.notes" type="textarea" :rows="2" />
                </el-form-item>
            </div>

            <div
                class="mb-4 rounded-xl border border-[rgba(64,158,255,0.12)] bg-[rgba(64,158,255,0.06)] px-4 py-3 text-sm text-[var(--color-text-secondary)]">
                {{ t('credential.form.typeDescription') }}
                <span class="font-600 text-[var(--color-text-primary)]">{{
                    getCredentialTemplateLabel(credForm.credential_type)
                }}</span>
            </div>

            <!-- Sensitive info -->
            <h4 class="section-heading">{{ t('credential.detail.sensitiveInfo') }}</h4>

            <div class="grid gap-4 md:grid-cols-2">
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
                        <div v-else class="sensitive-field">
                            <el-input
                                v-model="credForm.sensitive[field.key]"
                                :placeholder="t(field.hintKey)"
                                :type="visibleFields[field.key] ? 'text' : 'password'" />
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
            <h4 class="section-heading">{{ t('credential.detail.customFields') }}</h4>

            <div v-for="(field, index) in customFields" :key="index" class="custom-field-row">
                <el-input v-model="field.key" placeholder="Key" class="custom-key" />
                <div class="sensitive-field custom-value">
                    <el-input
                        v-model="field.value"
                        :type="field.visible ? 'text' : 'password'"
                        placeholder="Value" />
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
            <button type="submit" style="display: none" />
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
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { ElMessage } from 'element-plus';
import type { FormInstance } from 'element-plus';
import { View, Hide, Delete, CopyDocument, Plus } from '@element-plus/icons-vue';
import {
    buildSensitiveData,
    defaultCredentialForm,
    credentialTemplateOptions,
    getCredentialTemplateLabel,
    inferCredentialType,
    normalizeSensitiveFields,
    shouldShowField,
    type CredentialFormModel,
    type CredentialTemplateKey,
} from './credentialForm';
import {
    useCredential,
    type CredentialView,
    type CredentialDetail,
    type SensitiveData,
} from '@/composables/useCredential';

const { t } = useI18n();
const { lock, getCredential, createCredential, updateCredential } = useCredential();

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

const credFormRef = ref<FormInstance>();
const isEditMode = ref(false);
const editingCredId = ref<number | null>(null);
const credSaving = ref(false);

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
    { key: 'expires_at', labelKey: 'credential.form.expiresAtLabel', hintKey: 'credential.form.expiresAtHint', isDatePicker: true },
    { key: 'access_token', labelKey: 'credential.detail.accessToken', hintKey: 'credential.form.accessTokenHint' },
    { key: 'refresh_token', labelKey: 'credential.detail.refreshToken', hintKey: 'credential.form.refreshTokenHint' },
];

// ── Methods ──

const handleCredentialTypeChange = () => {
    normalizeSensitiveFields(credForm);
};

const resetCredForm = () => {
    const defaultCategoryId =
        props.categories.length > 0 ? props.categories[0].id : null;
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
</script>

<style scoped>
.sensitive-field {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
}

.sensitive-field .el-input {
    flex: 1;
}

.custom-field-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
}

.custom-key {
    width: 140px;
    flex-shrink: 0;
}

.custom-value {
    flex: 1;
}

.section-heading {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-secondary);
    margin: 12px 0 8px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--color-window-titlebar-border);
}
</style>