<template>
    <AppDialog
        v-model="visible"
        title="从浏览器导入网站密码"
        width="750px"
        append-to-body
        destroy-on-close
        @closed="handleClosed">
        <!-- ═══ Step 1: Browser Selection ═══ -->
        <template v-if="!importedItems.length">
            <div v-if="scanningBrowsers" class="browser-loading">
                <el-icon class="is-loading" :size="20"><Loading /></el-icon>
                <span>正在检测已安装的浏览器...</span>
            </div>
            <template v-else>
                <p v-if="browsers.length === 0" class="browser-empty">未检测到已安装的浏览器。</p>
                <el-radio-group v-else v-model="selectedBrowser" class="browser-list">
                    <el-radio v-for="b in browsers" :key="b" :value="b" class="browser-radio">
                        {{ b }}
                    </el-radio>
                </el-radio-group>
            </template>
            <div class="browser-action">
                <el-button
                    type="primary"
                    :disabled="!selectedBrowser || scanningBrowsers"
                    :loading="scanningPasswords"
                    @click="handleScanPasswords">
                    扫描密码
                </el-button>
            </div>
        </template>

        <!-- ═══ Step 2: Results ═══ -->
        <template v-else>
            <el-table :data="importedItems" ref="tableRef" max-height="400" @selection-change="handleSelectionChange">
                <el-table-column type="selection" width="50" />
                <el-table-column type="index" label="序号" width="60" />
                <el-table-column prop="url" label="URL" min-width="200" show-overflow-tooltip />
                <el-table-column prop="username" label="用户名" width="140" show-overflow-tooltip />
                <el-table-column label="密码" width="180">
                    <template #default="{ row, $index }">
                        <span class="password-cell">
                            <span class="password-text">{{ passwordVisible[$index] ? row.password : '••••••••' }}</span>
                            <el-button link size="small" @click="togglePassword($index)">
                                <el-icon :size="16">
                                    <View v-if="!passwordVisible[$index]" />
                                    <Hide v-else />
                                </el-icon>
                            </el-button>
                        </span>
                    </template>
                </el-table-column>
                <el-table-column label="状态" width="90">
                    <template #default="{ row }">
                        <el-tag v-if="importedIds.has(row.id)" type="success" size="small">已导入</el-tag>
                        <span v-else class="text-gray-400">待导入</span>
                    </template>
                </el-table-column>
            </el-table>

            <div class="import-controls">
                <el-checkbox v-model="selectAll" :indeterminate="isIndeterminate" @change="handleSelectAllChange">
                    全选 / 取消全选
                </el-checkbox>
                <el-button
                    type="primary"
                    :disabled="selectedRows.length === 0"
                    :loading="importing"
                    @click="handleImportSelected">
                    导入选中 ({{ selectedRows.length }})
                </el-button>
            </div>

            <el-progress
                v-if="importing"
                :percentage="importProgress"
                :status="importProgress === 100 ? 'success' : undefined"
                style="margin-top: 12px" />
        </template>
    </AppDialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { ElMessage } from 'element-plus';
import { View, Hide, Loading } from '@element-plus/icons-vue';
import { useCredential, type CreateCredentialRequest } from '@/composables/useCredential';
import AppDialog from '@/components/common/AppDialog.vue';

const props = defineProps<{
    modelValue: boolean;
    dek: string | null;
}>();

const emit = defineEmits<{
    (e: 'update:modelValue', value: boolean): void;
    (e: 'imported'): void;
}>();

const visible = computed({
    get: () => props.modelValue,
    set: (v) => emit('update:modelValue', v),
});

const { scanBrowsers, importFromBrowser, createCredential, listCategories, createCategory } = useCredential();

// ── Browser scanning ──

const scanningBrowsers = ref(false);
const browsers = ref<string[]>([]);
const selectedBrowser = ref('');
const scanningPasswords = ref(false);

// ── Imported items ──

interface BrowserCredential {
    id: number;
    url: string;
    username: string;
    password: string;
    browser: string;
}

const importedItems = ref<BrowserCredential[]>([]);
const passwordVisible = ref<Record<number, boolean>>({});
const importedIds = ref<Set<number>>(new Set());
const tableRef = ref();

// ── Selection ──

const selectedRows = ref<BrowserCredential[]>([]);
const selectAll = ref(false);

const isIndeterminate = computed(() => {
    const unimported = importedItems.value.filter((item) => !importedIds.value.has(item.id));
    return selectedRows.value.length > 0 && selectedRows.value.length < unimported.length;
});

const handleSelectionChange = (rows: BrowserCredential[]) => {
    selectedRows.value = rows;
    selectAll.value = rows.length === importedItems.value.filter((item) => !importedIds.value.has(item.id)).length;
};

const handleSelectAllChange = (checked: boolean) => {
    if (!tableRef.value) return;
    if (checked) {
        importedItems.value.forEach((row) => {
            if (!importedIds.value.has(row.id)) {
                tableRef.value.toggleRowSelection(row, true);
            }
        });
    } else {
        tableRef.value.clearSelection();
    }
};

// ── Password toggle ──

const togglePassword = (index: number) => {
    passwordVisible.value[index] = !passwordVisible.value[index];
};

// ── Dialog open → scan browsers ──

watch(
    () => props.modelValue,
    (val) => {
        if (val) {
            resetState();
            doScanBrowsers();
        }
    },
);

const resetState = () => {
    browsers.value = [];
    selectedBrowser.value = '';
    importedItems.value = [];
    passwordVisible.value = {};
    importedIds.value = new Set();
    selectedRows.value = [];
    selectAll.value = false;
    scanningBrowsers.value = false;
    scanningPasswords.value = false;
    importing.value = false;
    importProgress.value = 0;
};

const handleClosed = () => {
    resetState();
};

const doScanBrowsers = async () => {
    scanningBrowsers.value = true;
    try {
        browsers.value = await scanBrowsers();
    } catch (err: unknown) {
        ElMessage.error(err instanceof Error ? err.message : '检测浏览器失败');
    } finally {
        scanningBrowsers.value = false;
    }
};

// ── Scan passwords ──

const handleScanPasswords = async () => {
    if (!selectedBrowser.value) return;
    scanningPasswords.value = true;
    try {
        const items = await importFromBrowser(selectedBrowser.value);
        importedItems.value = items;
        passwordVisible.value = {};
    } catch (err: unknown) {
        ElMessage.error(err instanceof Error ? err.message : '扫描密码失败');
    } finally {
        scanningPasswords.value = false;
    }
};

// ── Import ──

const importing = ref(false);
const importProgress = ref(0);

const generateNonce = (): string => {
    const nonce = new Uint8Array(12);
    crypto.getRandomValues(nonce);
    return btoa(String.fromCharCode(...nonce));
};

const resolveDefaultCategory = async (): Promise<number> => {
    const categories = await listCategories();
    const uncategorized = categories.find((cat) => cat.name === '未分类');
    if (uncategorized) return uncategorized.id;
    const created = await createCategory('未分类');
    return created.id;
};

const handleImportSelected = async () => {
    if (!props.dek) {
        ElMessage.warning('保险库未解锁');
        return;
    }

    const toImport = selectedRows.value.filter((row) => !importedIds.value.has(row.id));
    if (toImport.length === 0) {
        ElMessage.info('没有需要导入的凭证');
        return;
    }

    importing.value = true;
    importProgress.value = 0;

    let categoryId: number;
    try {
        categoryId = await resolveDefaultCategory();
    } catch (err: unknown) {
        ElMessage.error(err instanceof Error ? err.message : '获取分类失败');
        importing.value = false;
        return;
    }

    let successCount = 0;
    let failCount = 0;

    for (let i = 0; i < toImport.length; i++) {
        const item = toImport[i];
        try {
            const nonceBase64 = generateNonce();
            const sensitiveDataJson = JSON.stringify({
                credential_type: 'password',
                password: item.password,
            });

            const request: CreateCredentialRequest = {
                category_id: categoryId,
                title: item.url || item.username || '未命名',
                username: item.username || undefined,
                url: item.url || undefined,
                sensitive_data_json: sensitiveDataJson,
                dekBase64: props.dek,
                nonceBase64,
            };

            await createCredential(request);
            importedIds.value = new Set([...importedIds.value, item.id]);
            successCount++;
        } catch (err: unknown) {
            const msg = err instanceof Error ? err.message : String(err);
            console.error(`导入失败 [${item.url || item.username}]:`, msg);
            failCount++;
        }

        importProgress.value = Math.round(((i + 1) / toImport.length) * 100);
    }

    importing.value = false;

    if (successCount > 0) {
        ElMessage.success(`成功导入 ${successCount} 条凭证${failCount > 0 ? `，${failCount} 条失败` : ''}`);
    } else {
        ElMessage.error(`导入失败，${failCount} 条均未成功。请查看控制台获取详细错误。`);
    }

    // Check if all unimported items are done
    const allDone = importedItems.value.every((item) => importedIds.value.has(item.id));
    if (allDone) {
        setTimeout(() => {
            emit('imported');
        }, 800);
    }
};

// ── Clear selection when import completes for some items ──

watch(
    importedIds,
    () => {
        selectAll.value = false;
        selectedRows.value = [];
    },
    { deep: true },
);
</script>

<style scoped>
.browser-loading {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 24px 0;
    color: #6b7280;
    font-size: 14px;
}

.browser-empty {
    padding: 24px 0;
    color: #9ca3af;
    font-size: 14px;
    text-align: center;
}

.browser-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 8px 0;
}

.browser-radio {
    margin-right: 0;
    height: 36px;
    display: flex;
    align-items: center;
}

.browser-action {
    margin-top: 16px;
    padding-top: 12px;
    border-top: 1px solid var(--color-window-titlebar-border, #e5e7eb);
}

.password-cell {
    display: flex;
    align-items: center;
    gap: 4px;
}

.password-text {
    font-family: monospace;
    min-width: 80px;
}

.import-controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 16px;
    padding-top: 12px;
    border-top: 1px solid var(--color-window-titlebar-border, #e5e7eb);
}

.text-gray-400 {
    color: #9ca3af;
}
</style>
