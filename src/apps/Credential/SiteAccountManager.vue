<template>
    <AppDialog
        v-model="visible"
        :title="dialogTitle"
        width="900"
        append-to-body
        destroy-on-close
        class="site-account-dialog">
        <el-form ref="formRef" :model="siteForm" label-width="100px">
            <!-- 網站信息 -->
            <h4 class="section-title">{{ t('siteManager.siteInfo') }}</h4>

            <div class="grid grid-cols-2 gap-4">
                <el-form-item :label="t('siteManager.siteName')" required>
                    <el-input v-model="siteForm.name" :placeholder="t('siteManager.siteNamePlaceholder')" />
                </el-form-item>

                <el-form-item :label="t('siteManager.siteUrl')">
                    <el-input v-model="siteForm.url" :placeholder="t('siteManager.siteUrlPlaceholder')" />
                </el-form-item>

                <el-form-item :label="t('credential.list.category')" required>
                    <el-select
                        v-model="siteForm.category_id"
                        :placeholder="t('credential.list.category')"
                        class="w-full">
                        <el-option v-for="cat in categories" :key="cat.id" :label="cat.name" :value="cat.id">
                            <span :style="{ paddingLeft: cat.level * 20 + 'px' }">{{ cat.name }}</span>
                        </el-option>
                    </el-select>
                </el-form-item>

                <el-form-item :label="t('credential.detail.tags')">
                    <el-input v-model="siteForm.tags" :placeholder="t('credential.detail.tags')" />
                </el-form-item>

                <el-form-item :label="t('credential.detail.notes')" class="col-span-2">
                    <el-input v-model="siteForm.notes" type="textarea" :rows="2" />
                </el-form-item>
            </div>

            <!-- 賬號列表 -->
            <h4 class="section-title">{{ t('siteManager.accountList') }}</h4>

            <div class="account-list">
                <div v-for="(account, index) in siteForm.accounts" :key="index" class="account-item">
                    <div class="account-header">
                        <span class="account-index">{{ t('siteManager.account') }} {{ index + 1 }}</span>
                        <el-button-group size="small">
                            <el-button @click="editAccount(index)" :disabled="isEditingAccount">
                                <el-icon><Edit /></el-icon>
                            </el-button>
                            <el-button
                                type="danger"
                                @click="deleteAccount(index)"
                                :disabled="siteForm.accounts.length <= 1">
                                <el-icon><Delete /></el-icon>
                            </el-button>
                        </el-button-group>
                    </div>

                    <div class="account-info">
                        <div class="info-row">
                            <span class="label">{{ t('credential.list.username') }}:</span>
                            <span class="value">{{ account.username || '-' }}</span>
                        </div>
                        <div class="info-row">
                            <span class="label">{{ t('credential.form.passwordLabel') }}:</span>
                            <span class="value">••••••••</span>
                        </div>
                    </div>
                </div>

                <el-button type="primary" @click="addAccount" :disabled="isEditingAccount" class="add-account-btn">
                    <el-icon><Plus /></el-icon>
                    {{ t('siteManager.addAccount') }}
                </el-button>
            </div>

            <!-- 編輯賬號表單 -->
            <div v-if="isEditingAccount" class="account-edit-form">
                <h4 class="section-title">{{ t('siteManager.editAccountInfo') }}</h4>

                <div class="grid grid-cols-2 gap-4">
                    <el-form-item :label="t('credential.list.username')" required>
                        <el-input v-model="editingAccount.username" />
                    </el-form-item>

                    <el-form-item :label="t('credential.form.passwordLabel')" required>
                        <el-input v-model="editingAccount.password" type="password" show-password />
                    </el-form-item>

                    <el-form-item label="API Key">
                        <el-input v-model="editingAccount.api_key" />
                    </el-form-item>

                    <el-form-item :label="t('credential.form.secretLabel')">
                        <el-input v-model="editingAccount.secret_key" type="password" show-password />
                    </el-form-item>
                </div>

                <div class="edit-actions">
                    <el-button @click="cancelEditAccount">{{ t('app.cancel') }}</el-button>
                    <el-button type="primary" @click="saveAccount">{{ t('app.save') }}</el-button>
                </div>
            </div>
        </el-form>

        <template #footer>
            <el-button @click="visible = false">{{ t('app.cancel') }}</el-button>
            <el-button type="primary" :loading="saving" @click="handleSave">{{ t('app.save') }}</el-button>
        </template>
    </AppDialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { ElMessage } from 'element-plus';
import { Edit, Delete, Plus } from '@element-plus/icons-vue';
import AppDialog from '@/components/common/AppDialog.vue';

const { t } = useI18n();

// Props
const props = defineProps<{
    modelValue: boolean;
    categories: Array<{ id: number; name: string; level: number }>;
    editingSite?: any;
    dek?: string | null;
}>();

// Emits
const emit = defineEmits<{
    (e: 'update:modelValue', val: boolean): void;
    (e: 'saved', data: any): void;
}>();

// State
const visible = ref(props.modelValue);
const saving = ref(false);
const isEditingAccount = ref(false);
const editAccountIndex = ref(-1);

// 網站表單
const siteForm = reactive({
    id: null as number | null,
    name: '',
    url: '',
    category_id: null as number | null,
    tags: '',
    notes: '',
    accounts: [] as Array<{
        username: string;
        password: string;
        api_key?: string;
        secret_key?: string;
    }>,
});

// 編輯中的賬號
const editingAccount = reactive({
    username: '',
    password: '',
    api_key: '',
    secret_key: '',
});

// 計算屬性
const dialogTitle = computed(() => (props.editingSite ? t('siteManager.editSite') : t('siteManager.addSite')));

// 方法
const resetForm = () => {
    siteForm.id = null;
    siteForm.name = '';
    siteForm.url = '';
    siteForm.category_id = props.categories.length > 0 ? props.categories[0].id : null;
    siteForm.tags = '';
    siteForm.notes = '';
    siteForm.accounts = [{ username: '', password: '' }];
    isEditingAccount.value = false;
    editAccountIndex.value = -1;
};

const addAccount = () => {
    isEditingAccount.value = true;
    editAccountIndex.value = -1;
    Object.assign(editingAccount, {
        username: '',
        password: '',
        api_key: '',
        secret_key: '',
    });
};

const editAccount = (index: number) => {
    isEditingAccount.value = true;
    editAccountIndex.value = index;
    const account = siteForm.accounts[index];
    Object.assign(editingAccount, {
        username: account.username,
        password: account.password,
        api_key: account.api_key || '',
        secret_key: account.secret_key || '',
    });
};

const saveAccount = () => {
    if (!editingAccount.username.trim()) {
        ElMessage.warning(t('siteManager.enterUsername'));
        return;
    }
    if (!editingAccount.password.trim()) {
        ElMessage.warning(t('siteManager.enterPassword'));
        return;
    }

    const accountData = {
        username: editingAccount.username,
        password: editingAccount.password,
    };

    // 可選字段
    if (editingAccount.api_key.trim()) {
        (accountData as any).api_key = editingAccount.api_key;
    }
    if (editingAccount.secret_key.trim()) {
        (accountData as any).secret_key = editingAccount.secret_key;
    }

    if (editAccountIndex.value === -1) {
        // 新增
        siteForm.accounts.push(accountData);
    } else {
        // 編輯
        siteForm.accounts[editAccountIndex.value] = accountData;
    }

    isEditingAccount.value = false;
    editAccountIndex.value = -1;
    ElMessage.success(t('siteManager.accountSaved'));
};

const cancelEditAccount = () => {
    isEditingAccount.value = false;
    editAccountIndex.value = -1;
};

const deleteAccount = (index: number) => {
    if (siteForm.accounts.length <= 1) {
        ElMessage.warning(t('siteManager.cannotDeleteLastAccount'));
        return;
    }
    siteForm.accounts.splice(index, 1);
    ElMessage.success(t('siteManager.accountDeleted'));
};

const handleSave = async () => {
    if (!siteForm.name.trim()) {
        ElMessage.warning(t('siteManager.enterSiteName'));
        return;
    }

    if (!siteForm.category_id) {
        ElMessage.warning(t('siteManager.selectCategory'));
        return;
    }

    if (siteForm.accounts.length === 0) {
        ElMessage.warning(t('siteManager.atLeastOneAccount'));
        return;
    }

    // 驗證所有賬號
    for (const account of siteForm.accounts) {
        if (!account.username || !account.password) {
            ElMessage.warning(t('siteManager.completeAccountInfo'));
            return;
        }
    }

    saving.value = true;

    try {
        // 這裡需要調用API保存數據
        emit('saved', {
            id: siteForm.id,
            name: siteForm.name,
            url: siteForm.url,
            category_id: siteForm.category_id,
            tags: siteForm.tags,
            notes: siteForm.notes,
            accounts: siteForm.accounts,
        });

        visible.value = false;
    } finally {
        saving.value = false;
    }
};

// Watch
watch(
    () => props.modelValue,
    (val) => {
        visible.value = val;
        if (val) {
            if (props.editingSite) {
                // 編輯模式 - 加載數據
                Object.assign(siteForm, props.editingSite);
            } else {
                // 新增模式 - 重置表單
                resetForm();
            }
        }
    },
);

watch(visible, (val) => {
    emit('update:modelValue', val);
});
</script>

<style scoped>
.section-title {
    font-size: var(--app-font-14);
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 20px 0 12px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--color-window-titlebar-border);
}

.account-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.account-item {
    padding: 16px;
    border: 1px solid var(--color-input-border);
    border-radius: 8px;
    background-color: var(--color-input-bg);
}

.account-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
}

.account-index {
    font-size: var(--app-font-14);
    font-weight: 600;
    color: var(--color-text-primary);
}

.account-info {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.info-row {
    display: flex;
    align-items: center;
    gap: 8px;
}

.info-row .label {
    font-size: var(--app-font-12);
    color: var(--color-text-secondary);
    width: 80px;
}

.info-row .value {
    font-size: var(--app-font-13);
    color: var(--color-text-primary);
    font-weight: 500;
}

.add-account-btn {
    width: 100%;
}

.account-edit-form {
    margin-top: 20px;
    padding: 16px;
    background-color: var(--color-card-bg);
    border-radius: 8px;
}

.edit-actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 16px;
}
</style>
