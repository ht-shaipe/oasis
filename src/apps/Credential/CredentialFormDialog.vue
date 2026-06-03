<template>
    <AppDialog
        :model-value="modelValue"
        @update:model-value="$emit('update:modelValue', $event)"
        :title="dialogTitle"
        width="800"
        append-to-body
        destroy-on-close
        class="credential-form-dialog [&_.el-dialog]:rounded-3 [&_.el-dialog]:overflow-hidden [&_.el-dialog]:h-[82vh] [&_.el-dialog]:flex [&_.el-dialog]:flex-col [&_.el-dialog__header]:px-6 [&_.el-dialog__header]:pt-5 [&_.el-dialog__header]:pb-4 [&_.el-dialog__header]:border-b [&_.el-dialog__header]:border-[var(--color-window-titlebar-border)] [&_.el-dialog__header]:bg-[var(--color-bg-glass)] [&_.el-dialog__title]:text-[var(--app-font-16)] [&_.el-dialog__title]:font-600 [&_.el-dialog__title]:text-[var(--color-text-primary)] [&_.el-dialog__body]:p-6 [&_.el-dialog__body]:flex-1 [&_.el-dialog__body]:min-h-0 [&_.el-dialog__body]:overflow-y-auto [&_.el-dialog__footer]:px-6 [&_.el-dialog__footer]:py-4 [&_.el-dialog__footer]:border-t [&_.el-dialog__footer]:border-[var(--color-window-titlebar-border)] [&_.el-dialog__footer]:bg-[var(--color-bg-glass)]">
        <el-form
            :model="credForm"
            @submit.prevent="handleSaveCredential"
            class="m-0 flex h-full flex-col"
            label-width="80px">
            <h4
                class="mb-3 mt-0 pb-1.5 text-sm font-600 text-[var(--color-text-primary)] border-b border-solid border-0 border-[var(--color-window-titlebar-border)]">
                {{ t('credential.detail.basicInfo') }}
            </h4>

            <div
                class="grid grid-cols-2 gap-2 border-[var(--color-window-titlebar-border)] border-1 border-solid p-2 rounded-2">
                <el-form-item :label="t('credential.detail.credentialType')" class="col-span-2">
                    <div
                        class="flex w-full min-w-0 items-stretch overflow-hidden rounded-1 border border-solid border-[var(--color-input-border)] bg-[var(--color-bg-page)]">
                        <el-select
                            v-model="credForm.credential_type"
                            class="flex-1 credential-type-select"
                            popper-class="credential-form-dialog"
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
                        <el-button
                            class="h-[32px] w-[32px] min-w-[42px] rounded-none border-0 border-l border-[var(--color-input-border)] bg-[var(--color-bg-page)] p-0 hover:bg-[color-mix(in_srgb,var(--color-primary)_8%,var(--color-bg-page))]"
                            @click="showTemplateManager = true"
                            title="管理模板">
                            <el-icon><Setting /></el-icon>
                        </el-button>
                    </div>
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

                <el-form-item :label="t('credential.list.name')" required>
                    <el-input v-model="credForm.title" />
                </el-form-item>

                <el-form-item :label="t('credential.list.url')" class="col-span-2">
                    <el-input v-model="credForm.url" :placeholder="t('credential.form.urlHint')" />
                </el-form-item>

                <!-- 密钥凭证 -->
                <template v-if="isKeyCredentialType">
                    <el-form-item label="接口地址">
                        <el-input v-model="credForm.api_url" placeholder="https://api.example.com/v1" />
                    </el-form-item>

                    <el-form-item label="文档地址">
                        <el-input v-model="credForm.doc_url" placeholder="https://docs.example.com" />
                    </el-form-item>
                </template>

                <el-form-item :label="t('credential.detail.tags')">
                    <el-input v-model="credForm.tags" :placeholder="t('credential.detail.tags')" />
                </el-form-item>

                <el-form-item :label="t('credential.detail.notes')" class="col-span-2">
                    <el-input v-model="credForm.notes" type="textarea" :rows="2" />
                </el-form-item>
            </div>

            <!-- 账号凭证：用户名/密码直接跟在网址后面 -->
            <template v-if="isAccountLikeCredential">
                <div
                    class="flex items-center justify-between m-1 py-1 border-b border-solid border-0 border-[var(--color-window-titlebar-border)]">
                    <h4 class="m-1 pb-1.5 text-sm font-600 text-[var(--color-text-primary)]">
                        {{ t('credential.detail.sensitiveInfo') }}
                    </h4>
                    <el-button text size="small" class="mt-0.5" @click="addAccountSet">
                        <el-icon><Plus /></el-icon>
                        添加一套账号
                    </el-button>
                </div>
                <div class="col-span-2 space-y-1.5 mt-1">
                    <div v-for="(account, index) in credForm.accounts" :key="index" class="flex items-start gap-1.5">
                        <div class="grid grid-cols-1 sm:grid-cols-3 gap-2 items-start flex-1 min-w-0">
                            <el-form-item
                                :label="t('credential.list.username')"
                                label-width="54px"
                                class="mb-0 min-w-0">
                                <el-input v-model="account.username" :placeholder="t('credential.list.username')" />
                            </el-form-item>
                            <el-form-item
                                :label="t('credential.detail.password')"
                                label-width="54px"
                                class="mb-0 min-w-0">
                                <el-input
                                    v-model="account.password"
                                    type="password"
                                    show-password
                                    :placeholder="t('credential.detail.password')" />
                            </el-form-item>
                            <el-form-item :label="t('credential.detail.notes')" label-width="54px" class="mb-0 min-w-0">
                                <el-input v-model="account.notes" :placeholder="t('credential.detail.notes')" />
                            </el-form-item>
                        </div>
                        <el-button
                            link
                            type="danger"
                            :disabled="credForm.accounts.length === 1"
                            class="self-start mt-4.5 shrink-0"
                            @click="removeAccountSet(index)">
                            <el-icon><Delete /></el-icon>
                        </el-button>
                    </div>
                </div>
            </template>

            <!-- 非账号凭证：敏感信息独立区块 -->
            <template v-if="!isAccountLikeCredential">
                <div
                    class="flex items-center justify-between m-1 border-b border-solid border-0 border-[var(--color-window-titlebar-border)]">
                    <h4 class="m-1 pb-1.5 text-sm font-600 text-[var(--color-text-primary)]">
                        {{ t('credential.detail.sensitiveInfo') }}
                    </h4>
                </div>

                <div class="space-y-2 border border-solid border-[var(--color-window-titlebar-border)] rounded-2 p-2">
                    <div
                        v-for="(account, index) in credForm.accounts"
                        :key="index"
                        class="bg-[var(--color-bg-page)] p-1">
                        <div class="grid grid-cols-1 gap-2 lg:grid-cols-25 items-start">
                            <template v-for="field in sensitiveFieldDefs" :key="field.key">
                                <el-form-item
                                    v-if="shouldShowField(credForm.credential_type, field.key)"
                                    label-width="68px"
                                    :class="[
                                        'mb-0 min-w-0 col-span-12',
                                        templateFieldsLength(credForm.credential_type) < 2
                                            ? 'lg:col-span-16'
                                            : 'lg:col-span-8',
                                    ]"
                                    :label="t(field.labelKey)">
                                    <el-date-picker
                                        v-if="field.isDatePicker"
                                        v-model="account[field.key]"
                                        class="w-full"
                                        type="datetime"
                                        :placeholder="t(field.hintKey)"
                                        value-format="YYYY-MM-DD HH:mm:ss" />
                                    <div v-else class="flex items-center gap-1 w-full">
                                        <el-input
                                            v-model="account[field.key]"
                                            :placeholder="t(field.hintKey)"
                                            :type="visibleFields[field.key] ? 'text' : 'password'"
                                            :show-password="!visibleFields[field.key]"
                                            class="flex-1" />
                                        <el-button link @click="copyToClipboard(account[field.key])">
                                            <el-icon><CopyDocument /></el-icon>
                                        </el-button>
                                    </div>
                                </el-form-item>
                            </template>

                            <el-form-item
                                :label="t('credential.detail.notes')"
                                label-width="64px"
                                class="mb-0 min-w-0 col-span-12 lg:col-span-8">
                                <el-input v-model="account.notes" :placeholder="t('credential.detail.notes')" />
                            </el-form-item>

                            <el-button
                                link
                                type="danger"
                                :disabled="credForm.accounts.length === 1"
                                class="col-span-12 lg:col-span-1 self-start justify-self-end mt-1"
                                @click="removeAccountSet(index)">
                                <el-icon><Delete /></el-icon>
                            </el-button>
                        </div>
                    </div>
                </div>
            </template>

            <div
                class="flex items-center justify-between mt-5 mb-3 border-b border-solid border-0 border-[var(--color-window-titlebar-border)]">
                <h4 class="mb-3 mt-5 pb-1.5 text-sm font-600 text-[var(--color-text-primary)]">
                    {{ t('credential.detail.customFields') }}
                </h4>
                <el-button text @click="customFields.push({ key: '', value: '', visible: false })">
                    <el-icon><Plus /></el-icon>
                    {{ t('credential.detail.addField') }}
                </el-button>
            </div>
            <div v-for="(field, index) in customFields" :key="index" class="flex items-center gap-2 mb-2.5">
                <el-input v-model="field.key" placeholder="Key" class="w-35 flex-shrink-0" />
                <div class="flex items-center gap-1 flex-1">
                    <el-input
                        v-model="field.value"
                        :type="field.visible ? 'text' : 'password'"
                        placeholder="Value"
                        :show-password="!field.visible"
                        class="flex-1" />
                </div>
                <el-button link type="danger" @click="customFields.splice(index, 1)">
                    <el-icon><Delete /></el-icon>
                </el-button>
            </div>

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
    </AppDialog>

    <TemplateManager
        v-model="showTemplateManager"
        :templates="allTemplates"
        @update:templates="handleTemplatesUpdate" />
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { ElMessage } from 'element-plus';
import { Delete, CopyDocument, Plus, Setting } from '@element-plus/icons-vue';
import AppDialog from '@/components/common/AppDialog.vue';
import {
    buildSensitiveData,
    createEmptyCredentialAccount,
    defaultCredentialForm,
    defaultCredentialTemplateOptions,
    saveCredentialTemplates,
    inferCredentialType,
    normalizeSensitiveFields,
    shouldShowField,
    type CredentialAccountForm,
    type CredentialFormModel,
    type CredentialTemplateKey,
    type CredentialTemplateOption,
    templateFieldsLength,
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

const dialogTitle = computed(() =>
    isEditMode.value ? t('credential.detail.editTitle') : t('credential.detail.createTitle'),
);

const isAccountLikeCredential = computed(
    () => credForm.credential_type === 'account' || credForm.credential_type === 'custom',
);

const isKeyCredentialType = computed(() =>
    ['api_key', 'key_secret', 'expiring_key'].includes(credForm.credential_type),
);

const allTemplates = computed(() => [...defaultCredentialTemplateOptions, ...customTemplates.value]);

const isEditMode = ref(false);
const editingCredId = ref<number | null>(null);
const credSaving = ref(false);
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

const handleCredentialTypeChange = () => {
    if (credForm.accounts.length === 0) {
        credForm.accounts = [createEmptyCredentialAccount()];
    }
    normalizeSensitiveFields(credForm);
};

const addAccountSet = () => {
    credForm.accounts.push(createEmptyCredentialAccount());
};

const removeAccountSet = (index: number) => {
    if (credForm.accounts.length <= 1) return;
    credForm.accounts.splice(index, 1);
};

const resetCredForm = () => {
    const defaultCategoryId = props.categories.length > 0 ? props.categories[0].id : null;
    Object.assign(credForm, defaultCredentialForm(defaultCategoryId));
    customFields.value = [];
    for (const key of Object.keys(visibleFields)) {
        visibleFields[key] = false;
    }
};

const fillSensitiveData = (sensitiveData: Partial<SensitiveData>, rowUsername: string | null) => {
    credForm.credential_type =
        (sensitiveData.credential_type as CredentialTemplateKey) ?? inferCredentialType(sensitiveData);

    credForm.api_url = sensitiveData.api_url || '';
    credForm.doc_url = sensitiveData.doc_url || '';

    credForm.sensitive.password = sensitiveData.password || '';
    credForm.sensitive.api_key = sensitiveData.api_key || '';
    credForm.sensitive.secret_key = sensitiveData.secret_key || '';
    credForm.sensitive.expires_at = sensitiveData.expires_at || '';
    credForm.sensitive.access_token = sensitiveData.access_token || '';
    credForm.sensitive.refresh_token = sensitiveData.refresh_token || '';

    const sensitiveSets =
        sensitiveData.sensitive_sets?.map((set) => ({
            username: set.username || '',
            notes: set.notes || '',
            password: set.password || '',
            api_key: set.api_key || '',
            secret_key: set.secret_key || '',
            access_token: set.access_token || '',
            refresh_token: set.refresh_token || '',
            expires_at: set.expires_at || '',
        })) ?? [];

    const accountSets =
        sensitiveData.account_sets?.map((account) => ({
            username: account.username || '',
            notes: account.notes || '',
            password: account.password || '',
            api_key: '',
            secret_key: '',
            access_token: '',
            refresh_token: '',
            expires_at: '',
        })) ?? [];

    if (sensitiveSets.length > 0) {
        if (rowUsername && !sensitiveSets[0].username) {
            sensitiveSets[0].username = rowUsername;
        }
        credForm.accounts = sensitiveSets;
        credForm.username = sensitiveSets.find((account) => account.username.trim())?.username || rowUsername || '';
    } else if (accountSets.length > 0) {
        if (rowUsername && !accountSets[0].username) {
            accountSets[0].username = rowUsername;
        }
        credForm.accounts = accountSets;
        credForm.username = accountSets.find((account) => account.username.trim())?.username || rowUsername || '';
    } else {
        credForm.accounts = [
            {
                username: rowUsername || '',
                notes: '',
                password: sensitiveData.password || '',
                api_key: sensitiveData.api_key || '',
                secret_key: sensitiveData.secret_key || '',
                access_token: sensitiveData.access_token || '',
                refresh_token: sensitiveData.refresh_token || '',
                expires_at: sensitiveData.expires_at || '',
            },
        ];
        credForm.username = rowUsername || '';
    }

    if (sensitiveData.custom_fields) {
        customFields.value = Object.entries(sensitiveData.custom_fields).map(([key, value]) => ({
            key,
            value,
            visible: false,
        }));
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
        fillSensitiveData(detail.sensitive_data, row.username);
    } else {
        try {
            const d = await getCredential(row.id);
            fillSensitiveData(d.sensitive_data, row.username);
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

const handleSaveCredential = async () => {
    if (!credForm.title.trim()) {
        ElMessage.warning(t('credential.list.title'));
        return;
    }

    if (!credForm.category_id || credForm.category_id <= 0) {
        ElMessage.warning('请选择一个分类');
        return;
    }

    const normalizedAccounts = credForm.accounts
        .map((account: CredentialAccountForm) => ({
            username: account.username.trim(),
            notes: account.notes.trim(),
            password: account.password,
            api_key: account.api_key,
            secret_key: account.secret_key,
            access_token: account.access_token,
            refresh_token: account.refresh_token,
            expires_at: account.expires_at,
        }))
        .filter((account) => {
            const hasTypeSensitiveValue = sensitiveFieldDefs.some(
                (field) =>
                    shouldShowField(credForm.credential_type, field.key) &&
                    String(account[field.key] || '').trim().length > 0,
            );
            const hasAccountMetaValue =
                (credForm.credential_type === 'account' || credForm.credential_type === 'custom') &&
                (account.username || account.notes);

            return hasTypeSensitiveValue || hasAccountMetaValue;
        });

    if (normalizedAccounts.length === 0) {
        ElMessage.warning('请至少填写一套敏感信息');
        return;
    }

    credForm.accounts =
        normalizedAccounts.length > 0
            ? normalizedAccounts.map((account) => ({ ...account }))
            : [createEmptyCredentialAccount()];

    const primaryUsername = normalizedAccounts.find((account) => account.username.trim())?.username || '';
    credForm.username = primaryUsername;

    normalizeSensitiveFields(credForm);
    const sensitiveData: SensitiveData = buildSensitiveData(credForm);

    const customObj: Record<string, string> = {};
    for (const field of customFields.value) {
        if (field.key.trim()) customObj[field.key.trim()] = field.value;
    }
    if (Object.keys(customObj).length > 0) {
        sensitiveData.custom_fields = customObj;
    }

    const sensitiveDataJson = JSON.stringify(sensitiveData);

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
    const defaultValues = new Set(defaultCredentialTemplateOptions.map((template) => template.value));
    customTemplates.value = templates.filter((template) => !defaultValues.has(template.value));
    saveCredentialTemplates(customTemplates.value);

    if (!templates.find((template) => template.value === credForm.credential_type)) {
        credForm.credential_type = 'account';
        normalizeSensitiveFields(credForm);
    }

    ElMessage.success('模板已更新');
};

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

onMounted(() => {
    loadCustomTemplates();
});
</script>

<style scoped>
.credential-form-dialog :deep(.el-overlay) {
    z-index: var(--z-index-modal);
}

.credential-form-dialog :deep(.el-select .el-input__wrapper),
.credential-form-dialog :deep(.credential-type-select .el-input__wrapper) {
    border-radius: 0 !important;
    border: none !important;
    box-shadow: none !important;
    background-color: transparent !important;
}

:global(.credential-form-dialog .el-form-item--default) {
    margin-bottom: 1px !important;
}

:global(.credential-form-dialog .credential-type-select .el-select__wrapper) {
    border: none !important;
    box-shadow: none !important;
}

:global(.credential-form-dialog .el-select-dropdown__item) {
    min-height: 56px !important;
    height: auto !important;
    padding: 6px 16px !important;
    display: flex;
    align-items: flex-start;
}

:global(.credential-form-dialog .el-select-dropdown__item .flex) {
    width: 100%;
}

:global(.credential-form-dialog .el-select-dropdown__item .text-sm) {
    line-height: 1.5;
}

:global(.credential-form-dialog .el-select-dropdown__item .text-xs) {
    line-height: 1.4;
    margin-top: 4px;
}
</style>
