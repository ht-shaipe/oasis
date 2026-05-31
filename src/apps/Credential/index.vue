<template>
    <MacWindow
        :title="t('credential.title')"
        :isMinimized="isMinimized"
        @close="handleClose"
        @minimize="emit('minimize')"
        width="1000"
        height="600">
        <div class="credential-container">
            <!-- ═══ Setup View ═══ -->
            <CredentialAuthCard
                v-if="viewState === 'setup'"
                :title="t('credential.setup.title')"
                :loading="setupLoading"
                :loading-text="setupLoadingText">
                <template #icon>
                    <el-icon :size="56"><Lock /></el-icon>
                </template>
                <el-form
                    ref="setupFormRef"
                    :model="setupForm"
                    :rules="setupRules"
                    label-position="top"
                    @submit.prevent="handleSetup">
                    <el-form-item :label="t('credential.setup.password')" prop="password">
                        <el-input
                            v-model="setupForm.password"
                            type="password"
                            show-password
                            :disabled="setupLoading"
                            :placeholder="t('credential.setup.passwordHint')" />
                    </el-form-item>
                    <el-form-item :label="t('credential.setup.confirmPassword')" prop="confirmPassword">
                        <el-input
                            v-model="setupForm.confirmPassword"
                            type="password"
                            show-password
                            :disabled="setupLoading"
                            :placeholder="t('credential.setup.confirmPassword')" />
                    </el-form-item>
                    <el-form-item>
                        <el-button
                            type="primary"
                            native-type="submit"
                            :loading="setupLoading"
                            :disabled="setupLoading"
                            size="large"
                            style="width: 100%">
                            {{ setupLoading ? t('credential.setup.submitting') : t('credential.setup.submit') }}
                        </el-button>
                    </el-form-item>
                </el-form>
            </CredentialAuthCard>

            <!-- ═══ Unlock View ═══ -->
            <CredentialAuthCard
                v-else-if="viewState === 'unlock'"
                :title="t('credential.unlock.title')"
                :loading="unlockLoading"
                :loading-text="unlockLoadingText">
                <template #icon>
                    <el-icon :size="56"><Unlock /></el-icon>
                </template>
                <el-form
                    ref="unlockFormRef"
                    :model="unlockForm"
                    :rules="unlockRules"
                    label-position="top"
                    @submit.prevent="handleUnlock">
                    <el-form-item :label="t('credential.unlock.password')" prop="password">
                        <el-input
                            v-model="unlockForm.password"
                            type="password"
                            show-password
                            :disabled="unlockLoading"
                            @keyup.enter="handleUnlock" />
                    </el-form-item>
                    <p v-if="unlockError" class="unlock-error">{{ unlockError }}</p>
                    <el-form-item>
                        <el-button
                            type="primary"
                            native-type="submit"
                            :loading="unlockLoading"
                            :disabled="unlockLoading"
                            style="width: 100%">
                            {{ unlockLoading ? t('credential.unlock.submitting') : t('credential.unlock.submit') }}
                        </el-button>
                    </el-form-item>
                </el-form>
            </CredentialAuthCard>

            <!-- ═══ Main View ═══ -->
            <el-splitter v-else class="credential-main">
                <el-splitter-panel v-model:size="sidebarSize" :min="SIDEBAR_MIN" :max="SIDEBAR_MAX">
                    <!-- Sidebar -->
                    <CredentialSidebar
                        :title="t('credential.title')"
                        :all-label="t('credential.category.all')"
                        :category-tree="categoryTree"
                        :selected-category-id="selectedCategoryId"
                        @add-category="showAddCategoryDialog = true"
                        @select-category="selectCategory"
                        @quick-add-sub-category="quickAddSubCategory"
                        @delete-category="handleDeleteCategory" />
                </el-splitter-panel>
                <el-splitter-panel>
                    <!-- Content area -->
                    <div class="credential-content">
                        <!-- Toolbar -->
                        <CredentialToolbar
                            v-model="searchQuery"
                            :search-placeholder="t('credential.list.search')"
                            :add-label="t('credential.list.add')"
                            :lock-label="t('credential.lock')"
                            @add="openCreateDialog"
                            @lock="handleLock" />

                        <!-- Credential table -->
                        <div class="credential-table-wrapper">
                            <el-empty
                                v-if="displayCredentials.length === 0 && !tableLoading"
                                :description="t('credential.list.empty')" />
                            <el-table
                                v-else
                                v-loading="tableLoading"
                                :data="displayCredentials"
                                style="width: 100%"
                                @row-dblclick="handleViewCredential">
                                <el-table-column :label="t('credential.list.title')" min-width="160">
                                    <template #default="{ row }">
                                        <div class="cred-title">
                                            <el-icon><Key /></el-icon>
                                            <span>{{ row.title }}</span>
                                        </div>
                                    </template>
                                </el-table-column>
                                <el-table-column :label="t('credential.list.username')" min-width="120">
                                    <template #default="{ row }">
                                        {{ row.username || '-' }}
                                    </template>
                                </el-table-column>
                                <el-table-column :label="t('credential.list.url')" min-width="140">
                                    <template #default="{ row }">
                                        {{ row.url || '-' }}
                                    </template>
                                </el-table-column>
                                <el-table-column :label="t('credential.list.category')" width="120">
                                    <template #default="{ row }">
                                        {{ row.category_name || '-' }}
                                    </template>
                                </el-table-column>
                                <el-table-column :label="t('credential.list.updatedAt')" width="160">
                                    <template #default="{ row }">
                                        {{ formatDate(row.updated_at) }}
                                    </template>
                                </el-table-column>
                                <el-table-column :label="t('credential.list.actions')" width="110" fixed="right">
                                    <template #default="{ row }">
                                        <el-button link size="small" @click="handleViewCredential(row)">
                                            <el-icon><View /></el-icon>
                                        </el-button>
                                        <el-button link size="small" @click="openEditDialog(row)">
                                            <el-icon><Edit /></el-icon>
                                        </el-button>
                                        <el-button link size="small" type="danger" @click="handleDeleteCredential(row)">
                                            <el-icon><Delete /></el-icon>
                                        </el-button>
                                    </template>
                                </el-table-column>
                            </el-table>
                        </div>
                    </div>
                </el-splitter-panel>
            </el-splitter>
        </div>

        <!-- ═══ Add Category Dialog ═══ -->
        <el-dialog v-model="showAddCategoryDialog" :title="t('credential.category.add')" width="600" append-to-body>
            <el-form @submit.prevent="handleAddCategory">
                <el-form-item :label="t('credential.category.name')">
                    <el-input v-model="newCategoryName" @keyup.enter="handleAddCategory" />
                </el-form-item>
                <el-form-item :label="t('credential.category.parent') || 'Parent Category'">
                    <el-select v-model="newCategoryParentId" placeholder="None" clearable style="width: 100%">
                        <el-option v-for="cat in flattenedCategories" :key="cat.id" :label="cat.name" :value="cat.id">
                            <span :style="{ paddingLeft: cat.level * 20 + 'px' }">{{ cat.name }}</span>
                        </el-option>
                    </el-select>
                </el-form-item>
            </el-form>
            <template #footer>
                <el-button
                    @click="
                        showAddCategoryDialog = false;
                        newCategoryParentId = null;
                    "
                    >{{ t('credential.detail.cancel') }}</el-button
                >
                <el-button type="primary" @click="handleAddCategory">{{ t('credential.detail.save') }}</el-button>
            </template>
        </el-dialog>

        <!-- ═══ Credential Form Dialog (standalone component) ═══ -->
        <CredentialFormDialog
            v-model="showCredDialog"
            :categories="flattenedCategories"
            :dek="dek"
            :editing-credential="editingRow"
            @saved="loadMainData" />

        <!-- ═══ Credential Detail Dialog ═══ -->
        <el-dialog
            v-model="showDetailDialog"
            :title="t('credential.detail.title')"
            width="600"
            append-to-body
            destroy-on-close>
            <template v-if="credentialDetail">
                <h4 class="section-heading">{{ t('credential.detail.basicInfo') }}</h4>
                <el-descriptions :column="1" border size="small">
                    <el-descriptions-item :label="t('credential.list.title')">{{
                        credentialDetail.title
                    }}</el-descriptions-item>
                    <el-descriptions-item :label="t('credential.list.username')">{{
                        credentialDetail.username || '-'
                    }}</el-descriptions-item>
                    <el-descriptions-item :label="t('credential.list.url')">{{
                        credentialDetail.url || '-'
                    }}</el-descriptions-item>
                    <el-descriptions-item :label="t('credential.list.category')">{{
                        credentialDetail.category_name || '-'
                    }}</el-descriptions-item>
                    <el-descriptions-item :label="t('credential.detail.credentialType')">{{
                        credentialDetail.sensitive_data?.credential_type
                            ? getCredentialTemplateLabel(
                                  credentialDetail.sensitive_data.credential_type as CredentialTemplateKey,
                              )
                            : '-'
                    }}</el-descriptions-item>
                    <el-descriptions-item :label="t('credential.detail.tags')">{{
                        credentialDetail.tags || '-'
                    }}</el-descriptions-item>
                    <el-descriptions-item :label="t('credential.detail.notes')">{{
                        credentialDetail.notes || '-'
                    }}</el-descriptions-item>
                </el-descriptions>

                <h4 class="section-heading" style="margin-top: 16px">{{ t('credential.detail.sensitiveInfo') }}</h4>
                <el-descriptions :column="1" border size="small">
                    <el-descriptions-item :label="t('credential.detail.password')">
                        <div class="detail-sensitive-value">
                            <span>{{
                                detailVisible.password ? credentialDetail.sensitive_data?.password || '-' : '••••••••'
                            }}</span>
                            <el-button link size="small" @click="detailVisible.password = !detailVisible.password">
                                <el-icon><component :is="detailVisible.password ? Hide : View" /></el-icon>
                            </el-button>
                            <el-button
                                link
                                size="small"
                                @click="copyToClipboard(credentialDetail.sensitive_data?.password)">
                                <el-icon><CopyDocument /></el-icon>
                            </el-button>
                        </div>
                    </el-descriptions-item>
                    <el-descriptions-item :label="t('credential.detail.apiKey')">
                        <div class="detail-sensitive-value">
                            <span>{{
                                detailVisible.apiKey ? credentialDetail.sensitive_data?.api_key || '-' : '••••••••'
                            }}</span>
                            <el-button link size="small" @click="detailVisible.apiKey = !detailVisible.apiKey">
                                <el-icon><component :is="detailVisible.apiKey ? Hide : View" /></el-icon>
                            </el-button>
                            <el-button
                                link
                                size="small"
                                @click="copyToClipboard(credentialDetail.sensitive_data?.api_key)">
                                <el-icon><CopyDocument /></el-icon>
                            </el-button>
                        </div>
                    </el-descriptions-item>
                    <el-descriptions-item :label="t('credential.detail.secretKey')">
                        <div class="detail-sensitive-value">
                            <span>{{
                                detailVisible.secretKey
                                    ? credentialDetail.sensitive_data?.secret_key || '-'
                                    : '••••••••'
                            }}</span>
                            <el-button link size="small" @click="detailVisible.secretKey = !detailVisible.secretKey">
                                <el-icon><component :is="detailVisible.secretKey ? Hide : View" /></el-icon>
                            </el-button>
                            <el-button
                                link
                                size="small"
                                @click="copyToClipboard(credentialDetail.sensitive_data?.secret_key)">
                                <el-icon><CopyDocument /></el-icon>
                            </el-button>
                        </div>
                    </el-descriptions-item>
                    <el-descriptions-item :label="t('credential.detail.accessToken')">
                        <div class="detail-sensitive-value">
                            <span>{{
                                detailVisible.accessToken
                                    ? credentialDetail.sensitive_data?.access_token || '-'
                                    : '••••••••'
                            }}</span>
                            <el-button
                                link
                                size="small"
                                @click="detailVisible.accessToken = !detailVisible.accessToken">
                                <el-icon><component :is="detailVisible.accessToken ? Hide : View" /></el-icon>
                            </el-button>
                            <el-button
                                link
                                size="small"
                                @click="copyToClipboard(credentialDetail.sensitive_data?.access_token)">
                                <el-icon><CopyDocument /></el-icon>
                            </el-button>
                        </div>
                    </el-descriptions-item>
                    <el-descriptions-item :label="t('credential.detail.refreshToken')">
                        <div class="detail-sensitive-value">
                            <span>{{
                                detailVisible.refreshToken
                                    ? credentialDetail.sensitive_data?.refresh_token || '-'
                                    : '••••••••'
                            }}</span>
                            <el-button
                                link
                                size="small"
                                @click="detailVisible.refreshToken = !detailVisible.refreshToken">
                                <el-icon><component :is="detailVisible.refreshToken ? Hide : View" /></el-icon>
                            </el-button>
                            <el-button
                                link
                                size="small"
                                @click="copyToClipboard(credentialDetail.sensitive_data?.refresh_token)">
                                <el-icon><CopyDocument /></el-icon>
                            </el-button>
                        </div>
                    </el-descriptions-item>
                    <el-descriptions-item
                        v-if="credentialDetail.sensitive_data?.expires_at"
                        :label="t('credential.form.expiresAtLabel')">
                        {{ credentialDetail.sensitive_data.expires_at }}
                    </el-descriptions-item>
                </el-descriptions>

                <!-- Custom fields in detail -->
                <template
                    v-if="
                        credentialDetail.sensitive_data?.custom_fields &&
                        Object.keys(credentialDetail.sensitive_data.custom_fields).length
                    ">
                    <h4 class="section-heading" style="margin-top: 16px">{{ t('credential.detail.customFields') }}</h4>
                    <el-descriptions :column="1" border size="small">
                        <el-descriptions-item
                            v-for="(val, key) in credentialDetail.sensitive_data.custom_fields"
                            :key="key"
                            :label="String(key)">
                            <div class="detail-sensitive-value">
                                <span>{{ detailCustomVisible[String(key)] ? val : '••••••••' }}</span>
                                <el-button
                                    link
                                    size="small"
                                    @click="detailCustomVisible[String(key)] = !detailCustomVisible[String(key)]">
                                    <el-icon
                                        ><component :is="detailCustomVisible[String(key)] ? Hide : View"
                                    /></el-icon>
                                </el-button>
                                <el-button link size="small" @click="copyToClipboard(val)">
                                    <el-icon><CopyDocument /></el-icon>
                                </el-button>
                            </div>
                        </el-descriptions-item>
                    </el-descriptions>
                </template>
            </template>

            <template #footer>
                <el-button @click="showDetailDialog = false">{{ t('credential.detail.cancel') }}</el-button>
                <el-button
                    type="primary"
                    @click="
                        openEditDialog(credentialDetail!);
                        showDetailDialog = false;
                    ">
                    <el-icon><Edit /></el-icon>
                    {{ t('credential.list.edit') }}
                </el-button>
            </template>
        </el-dialog>
    </MacWindow>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, reactive, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { ElMessage, ElMessageBox } from 'element-plus';
import type { FormInstance, FormRules } from 'element-plus';
import { View, Hide, Key, Edit, Delete, CopyDocument } from '@element-plus/icons-vue';
import MacWindow from '@/components/common/MacWindow.vue';
import CredentialAuthCard from './AuthCard.vue';
import CredentialSidebar from './Sidebar.vue';
import CredentialToolbar from './Toolbar.vue';
import { getCredentialTemplateLabel, type CredentialTemplateKey } from './credentialForm.ts';
import { useCredential, type Category, type CredentialView, type CredentialDetail } from '@/composables/useCredential';
import CredentialFormDialog from './CredentialFormDialog.vue';

const { t } = useI18n();
const {
    isMasterKeySet,
    setupMasterKey,
    unlock,
    lock,
    listCategories,
    createCategory,
    deleteCategory,
    listCredentials,
    getCredential,
    deleteCredential,
    dek,
} = useCredential();

// ── Sidebar Size Management ──

const SIDEBAR_MIN = 180;
const SIDEBAR_MAX = 400;
const SIDEBAR_DEFAULT = 200;
const SIDEBAR_STORAGE_KEY = 'credential-sidebar-width';

function loadSidebarWidth(): number {
    try {
        const saved = localStorage.getItem(SIDEBAR_STORAGE_KEY);
        let width = saved ? parseInt(saved, 10) : SIDEBAR_DEFAULT;
        // Ensure the width is within the valid range
        width = Math.max(SIDEBAR_MIN, Math.min(SIDEBAR_MAX, width));
        return width;
    } catch {
        return SIDEBAR_DEFAULT;
    }
}

const sidebarSize = ref(loadSidebarWidth());

// 监听尺寸变化，保存到 localStorage
watch(sidebarSize, (newSize) => {
    try {
        localStorage.setItem(SIDEBAR_STORAGE_KEY, String(newSize));
    } catch (error) {
        console.warn('Failed to save sidebar width:', error);
    }
});

// ── Props / Emits ──

const props = defineProps<{ isMinimized: boolean }>();
const emit = defineEmits<{
    (e: 'close'): void;
    (e: 'minimize'): void;
}>();

// ── View state machine ──

type ViewState = 'setup' | 'unlock' | 'main';
const viewState = ref<ViewState>('unlock');

// ── Setup form ──

const setupFormRef = ref<FormInstance>();
const setupForm = reactive({ password: '', confirmPassword: '' });
const setupLoading = ref(false);
const setupLoadingText = ref(t('credential.setup.loadingVerifying'));

const setupRules = computed<FormRules>(() => ({
    password: [
        { required: true, message: t('credential.setup.tooShort'), trigger: 'blur' },
        { min: 8, message: t('credential.setup.tooShort'), trigger: 'blur' },
    ],
    confirmPassword: [
        { required: true, message: t('credential.setup.mismatch'), trigger: 'blur' },
        {
            validator: (_rule: unknown, value: string, callback: (err?: Error) => void) => {
                if (value !== setupForm.password) {
                    callback(new Error(t('credential.setup.mismatch')));
                } else {
                    callback();
                }
            },
            trigger: 'blur',
        },
    ],
}));

const handleSetup = async () => {
    const form = setupFormRef.value;
    if (!form) return;
    await form.validate();
    setupLoading.value = true;
    setupLoadingText.value = t('credential.setup.loadingVerifying');
    try {
        await setupMasterKey(setupForm.password);
        setupLoadingText.value = t('credential.setup.loadingSyncing');
        await loadMainData();

        viewState.value = 'main';
    } catch (err: unknown) {
        ElMessage.error(err instanceof Error ? err.message : String(err));
    } finally {
        setupLoading.value = false;
    }
};

// ── Unlock form ──

const unlockFormRef = ref<FormInstance>();
const unlockForm = reactive({ password: '' });
const unlockLoading = ref(false);
const unlockLoadingText = ref(t('credential.unlock.loadingVerifying'));
const unlockError = ref('');
const unlockAttempts = ref(0);

const unlockRules = computed<FormRules>(() => ({
    password: [{ required: true, message: t('credential.unlock.wrongPassword'), trigger: 'blur' }],
}));

const handleUnlock = async () => {
    if (unlockLoading.value) return;

    const form = unlockFormRef.value;
    if (!form) return;
    await form.validate();
    unlockError.value = '';

    unlockLoading.value = true;
    unlockLoadingText.value = t('credential.unlock.loadingVerifying');

    // Show a warning after 5 failed attempts
    if (unlockAttempts.value >= 5) {
        unlockError.value = t('credential.unlock.tooManyAttempts');
    }

    try {
        await unlock(unlockForm.password);
        unlockAttempts.value = 0;
        unlockLoadingText.value = t('credential.unlock.loadingSyncing');
        await loadMainData();

        viewState.value = 'main';
    } catch {
        unlockAttempts.value++;
        unlockError.value = t('credential.unlock.wrongPassword');
    } finally {
        unlockLoading.value = false;
    }
};

// ── Lock ──

const handleLock = () => {
    lock();
    viewState.value = 'unlock';
    unlockForm.password = '';
    resetAutoLockTimer();
};

const handleClose = () => {
    lock();
    emit('close');
};

// ── Auto-lock (30 min) ──

let autoLockTimer: ReturnType<typeof setTimeout> | null = null;

const resetAutoLockTimer = () => {
    if (autoLockTimer) clearTimeout(autoLockTimer);
    if (viewState.value === 'main') {
        autoLockTimer = setTimeout(
            () => {
                lock();
                viewState.value = 'unlock';
                ElMessage.warning(t('credential.autoLockWarning'));
            },
            30 * 60 * 1000,
        );
    }
};

const onUserActivity = () => {
    if (viewState.value === 'main') resetAutoLockTimer();
};

// ── Main view data ──

const categories = ref<Category[]>([]);
const credentials = ref<CredentialView[]>([]);
const selectedCategoryId = ref<number | null>(null);
const searchQuery = ref('');
const tableLoading = ref(false);

// ── Tree Logic ──

interface CategoryNode extends Category {
    parent_id?: number | null;
    children: CategoryNode[];
}

const categoryTree = computed(() => {
    const map = new Map<number, CategoryNode>();
    const roots: CategoryNode[] = [];
    categories.value.forEach((cat) => map.set(cat.id, { ...cat, children: [] }));
    categories.value.forEach((cat) => {
        const node = map.get(cat.id)!;
        if (cat.parent_id && map.has(cat.parent_id)) {
            map.get(cat.parent_id)!.children.push(node);
        } else {
            roots.push(node);
        }
    });
    return roots;
});

// Flattened list for sidebar and selects
const flattenedCategories = computed(() => {
    const result: Array<{ id: number; name: string; level: number }> = [];
    const traverse = (nodes: CategoryNode[], level: number) => {
        nodes.forEach((node) => {
            result.push({ id: node.id, name: node.name, level });
            traverse(node.children, level + 1);
        });
    };
    traverse(categoryTree.value, 0);
    return result;
});

const getCategoryChildrenIds = (catId: number): number[] => {
    const ids: number[] = [catId];
    const findChildren = (parentId: number) => {
        categories.value.forEach((cat) => {
            if (cat.parent_id === parentId) {
                ids.push(cat.id);
                findChildren(cat.id);
            }
        });
    };
    findChildren(catId);
    return ids;
};

const filteredCredentials = computed(() => {
    let list = credentials.value;
    // If we filter by category, it's already filtered by loadMainData/selectCategory calling backend
    // But wait, if selectedCategoryId is set, the backend currently only returns that category.
    // The requirement says "display current category and its children".
    // Since we load credentials for a specific category from backend,
    // we might need to change how we fetch if the backend doesn't support recursive fetch.
    // For now, let's assume we fetch all and filter locally OR update the fetch logic.
    // Actually, listCredentials(catId) is called.
    // Let's assume we filter locally if we want recursive behavior, or keep backend fetch if it's preferred.
    // Given the "better experience" suggestion, I'll filter locally if I fetch all credentials,
    // OR call listCredentials with null and filter locally.

    if (!searchQuery.value) return list;
    const q = searchQuery.value.toLowerCase();
    return list.filter(
        (c) =>
            c.title.toLowerCase().includes(q) ||
            (c.username && c.username.toLowerCase().includes(q)) ||
            (c.url && c.url.toLowerCase().includes(q)),
    );
});

const loadMainData = async () => {
    try {
        // Fetch all categories and all credentials (to allow local filtering for recursive tree)
        const [cats, creds] = await Promise.all([listCategories(), listCredentials()]);
        categories.value = cats;
        credentials.value = creds;
    } catch (err: unknown) {
        ElMessage.error(err instanceof Error ? err.message : String(err));
    }
};

const displayCredentials = computed(() => {
    if (selectedCategoryId.value === null) return filteredCredentials.value;
    const targetIds = getCategoryChildrenIds(selectedCategoryId.value);
    return filteredCredentials.value.filter((c) => targetIds.includes(c.category_id));
});

const selectCategory = (catId: number | null) => {
    selectedCategoryId.value = catId;
};

// ── Add category ──

const showAddCategoryDialog = ref(false);
const newCategoryName = ref('');
const newCategoryParentId = ref<number | null>(null);

const handleAddCategory = async () => {
    if (!newCategoryName.value.trim()) return;
    try {
        const cat = await createCategory(
            newCategoryName.value.trim(),
            undefined,
            newCategoryParentId.value ?? undefined,
        );
        categories.value.push(cat);
        newCategoryName.value = '';
        newCategoryParentId.value = null;
        showAddCategoryDialog.value = false;
    } catch (err: unknown) {
        ElMessage.error(err instanceof Error ? err.message : String(err));
    }
};

const quickAddSubCategory = (parentId: number) => {
    newCategoryParentId.value = parentId;
    showAddCategoryDialog.value = true;
};

const handleDeleteCategory = async (data: Pick<Category, 'id' | 'name'>) => {
    try {
        await ElMessageBox.confirm(
            t('credential.category.deleteConfirm', { name: data.name }),
            t('credential.category.deleteTitle'),
            {
                confirmButtonText: t('credential.list.delete'),
                cancelButtonText: t('credential.detail.cancel'),
                type: 'warning',
            },
        );

        await deleteCategory(data.id);
        ElMessage.success(t('credential.category.deleteSuccess'));
        await loadMainData();
        if (selectedCategoryId.value === data.id) {
            selectedCategoryId.value = null;
        }
    } catch (err: unknown) {
        if (err !== 'cancel') {
            ElMessage.error(err instanceof Error ? err.message : String(err));
        }
    }
};

// ── Credential edit/create dialog ──

const showCredDialog = ref(false);
const editingRow = ref<CredentialView | CredentialDetail | null>(null);

const openCreateDialog = () => {
    editingRow.value = null;
    showCredDialog.value = true;
};

const openEditDialog = (row: CredentialView | CredentialDetail) => {
    editingRow.value = row;
    showCredDialog.value = true;
};

// ── Delete credential ──

const handleDeleteCredential = async (row: CredentialView) => {
    try {
        await ElMessageBox.confirm(t('credential.list.deleteConfirm'), {
            type: 'warning',
        });
        await deleteCredential(row.id);
        await loadMainData();
    } catch {
        // User cancelled or deletion failed silently
    }
};

// ── View credential detail ──

const showDetailDialog = ref(false);
const credentialDetail = ref<CredentialDetail | null>(null);
const detailVisible = reactive({
    password: false,
    apiKey: false,
    secretKey: false,
    accessToken: false,
    refreshToken: false,
});
const detailCustomVisible = reactive<Record<string, boolean>>({});

const handleViewCredential = async (row: CredentialView) => {
    try {
        const detail = await getCredential(row.id);
        credentialDetail.value = detail;
        detailVisible.password = false;
        detailVisible.apiKey = false;
        detailVisible.secretKey = false;
        detailVisible.accessToken = false;
        detailVisible.refreshToken = false;
        // Reset custom field visibility
        for (const k of Object.keys(detailCustomVisible)) delete detailCustomVisible[k];
        if (detail.sensitive_data?.custom_fields) {
            for (const k of Object.keys(detail.sensitive_data.custom_fields)) {
                detailCustomVisible[k] = false;
            }
        }
        showDetailDialog.value = true;
    } catch (err: unknown) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        if (errorMsg === 'Vault is locked') {
            ElMessage.warning('保险库已锁定，请重新输入密码');
            lock();
            viewState.value = 'unlock';
        } else {
            ElMessage.error(errorMsg);
        }
    }
};

// ── Clipboard ──

const copyToClipboard = async (text: string | undefined) => {
    if (!text) return;
    try {
        await navigator.clipboard.writeText(text);
        ElMessage.success(t('credential.detail.copied'));
    } catch {
        ElMessage.error(t('credential.detail.copy'));
    }
};

// ── Date formatting ──

const formatDate = (dateStr: string | null | undefined): string => {
    if (!dateStr) return '-';
    const d = new Date(dateStr);
    return `${d.toLocaleDateString()} ${d.toLocaleTimeString()}`;
};

// ── Lifecycle ──

onMounted(async () => {
    try {
        const keySet = await isMasterKeySet();
        viewState.value = keySet ? 'unlock' : 'setup';
    } catch {
        // If backend not ready, default to setup
        viewState.value = 'setup';
    }

    // Setup diagnostic functions
    (window as any).diagnoseCredential = async (id: number) => {
        try {
            const { diagnoseCredential } = useCredential();
            const result = await diagnoseCredential(id);
            console.log('Diagnostic result:', result);
            return result;
        } catch (err: unknown) {
            console.error('Diagnostic failed:', err);
            throw err;
        }
    };

    (window as any).fixCredential = async (id: number) => {
        try {
            const { fixCredential } = useCredential();
            const result = await fixCredential(id);
            console.log('Fix result:', result);
            await loadMainData();
            return result;
        } catch (err: unknown) {
            console.error('Fix failed:', err);
            throw err;
        }
    };

    // Activity listeners for auto-lock
    document.addEventListener('mousemove', onUserActivity);
    document.addEventListener('keydown', onUserActivity);
    document.addEventListener('click', onUserActivity);
});

onUnmounted(() => {
    if (autoLockTimer) clearTimeout(autoLockTimer);
    document.removeEventListener('mousemove', onUserActivity);
    document.removeEventListener('keydown', onUserActivity);
    document.removeEventListener('click', onUserActivity);
});
</script>

<style scoped>
.credential-container {
    display: flex;
    height: 100%;
    background-color: var(--color-sidebar-bg);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
}
.unlock-error {
    color: #f56c6c;
    font-size: 13px;
    margin: -8px 0 8px;
}

/* ── Main layout ── */

.credential-main {
    display: flex;
    width: 100%;
    height: 100%;
}

/* ── Content ── */

.credential-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background-color: var(--color-input-bg);
}

.credential-table-wrapper {
    flex: 1;
    overflow-y: auto;
    padding: 12px 16px;
}

.cred-title {
    display: flex;
    align-items: center;
    gap: 6px;
}

.cred-title .el-icon {
    color: #e6a23c;
}

/* ── Section heading ── */

.section-heading {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-secondary);
    margin: 12px 0 8px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--color-window-titlebar-border);
}

/* ── Detail dialog sensitive value ── */

.detail-sensitive-value {
    display: flex;
    align-items: center;
    gap: 6px;
}

.detail-sensitive-value span {
    flex: 1;
    word-break: break-all;
}

.truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
</style>
