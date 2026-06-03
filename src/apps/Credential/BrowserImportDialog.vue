<template>
    <AppDialog
        v-model="visible"
        title="从浏览器导入网站密码（CSV）"
        width="750px"
        append-to-body
        destroy-on-close
        @closed="handleClosed">
        <!-- ═══ Step 1: CSV Upload ═══ -->
        <template v-if="!importedItems.length">
            <!-- 使用提示 -->
            <el-collapse v-model="activeNames" class="usage-tips">
                <el-collapse-item title="使用提示：如何从浏览器导出密码 CSV" name="1">
                    <ul class="tip-list">
                        <li>
                            <strong>Chrome</strong>：打开 <code>chrome://password-manager/settings</code> &rarr;
                            下载文件 &rarr; 选择导出的 CSV
                        </li>
                        <li>
                            <strong>Edge</strong>：打开 <code>edge://wallet/passwords</code> &rarr; 设置 &rarr; 导出密码
                            &rarr; 选择导出的 CSV
                        </li>
                        <li>
                            <strong>Firefox</strong>：打开 <code>about:logins</code> &rarr; 三点菜单 &rarr; 导出登录信息
                            &rarr; 选择导出的 CSV
                        </li>
                        <li>
                            <strong>Brave</strong>：打开 <code>brave://password-manager/settings</code> &rarr; 下载文件
                            &rarr; 选择导出的 CSV
                        </li>
                        <li>
                            <strong>Safari</strong>：打开 Safari &rarr; 设置 &rarr; 密码 &rarr; 三点菜单 &rarr;
                            导出所有密码 &rarr; 选择导出的 CSV
                        </li>
                    </ul>
                </el-collapse-item>
            </el-collapse>

            <!-- 文件上传区域 -->
            <div class="csv-upload-area" @click="handleClickUpload">
                <el-icon class="upload-icon" :size="40"><UploadFilled /></el-icon>
                <div class="upload-text">点击选择 CSV 文件</div>
                <div class="upload-tip">支持 Chrome / Edge / Firefox / Brave / Safari 导出的 CSV 文件</div>
            </div>

            <div v-if="parsing" class="browser-loading">
                <el-icon class="is-loading" :size="20"><Loading /></el-icon>
                <span>正在解析 CSV 文件...</span>
            </div>
        </template>

        <!-- ═══ Step 2: Results ═══ -->
        <template v-else>
            <!-- 导入配置 -->
            <div class="import-config">
                <div class="config-row">
                    <div class="config-item">
                        <span class="config-label">凭证类型：</span>
                        <el-select
                            v-model="selectedCredentialType"
                            placeholder="选择凭证类型"
                            size="small"
                            class="type-select">
                            <el-option
                                v-for="opt in credentialTemplateOptions"
                                :key="opt.value"
                                :label="opt.label"
                                :value="opt.value"
                                :title="opt.description">
                            </el-option>
                        </el-select>
                    </div>
                    <div class="config-item">
                        <span class="config-label">分类：</span>
                        <el-tree-select
                            v-model="selectedCategoryId"
                            :data="categoryTree"
                            placeholder="选择分类（可选）"
                            size="small"
                            class="category-select"
                            clearable
                            filterable
                            :props="{ label: 'name', children: 'children' }"
                            node-key="id"
                            default-expand-all
                            check-strictly />
                    </div>
                </div>
            </div>

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
import { ref, computed, watch, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import { View, Hide, Loading, UploadFilled } from '@element-plus/icons-vue';
import { Category, useCredential, type CreateCredentialRequest } from '@/composables/useCredential';
import { useFileDialog } from '@/composables/useFileDialog';
import { credentialTemplateOptions, type CredentialTemplateKey } from './credentialForm';
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

// ── Preload categories on mount ──

onMounted(() => {
    fetchCategories();
});

const { importCsvPasswords, createCredential, listCategories, listCredentials, createCategory } = useCredential();
const { selectFile } = useFileDialog();

// ── Usage tips collapse ──

const activeNames = ref(['1']);

// ── CSV upload ──

const parsing = ref(false);

// ── Type & category selection ──

const selectedCredentialType = ref<CredentialTemplateKey>('account');
const selectedCategoryId = ref<number | null>(null);

const categoryTree = ref<Category[]>([]);

const fetchCategories = async () => {
    try {
        const categories: Category[] = await listCategories();
        const roots = categories.filter((c) => c.parent_id == null);
        const map = new Map<number, Category>();
        categories.forEach((c) => map.set(c.id, { ...c, children: [] }));

        const tree: Category[] = [];
        categories.forEach((c) => {
            const node = map.get(c.id)!;
            if (c.parent_id != null) {
                const parent = map.get(c.parent_id);
                if (parent) parent.children!.push(node);
            }
        });
        roots.forEach((r) => tree.push(map.get(r.id)!));

        categoryTree.value = tree;

        // 异步加载凭证数量统计（不阻塞树展示）
        loadCounts();
    } catch (err) {
        console.error('获取分类列表失败:', err);
    }
};

const loadCounts = async () => {
    try {
        const credentials = await listCredentials();
        const countMap = new Map<number, number>();
        credentials.forEach((cred) => {
            if (cred.category_id != null) {
                countMap.set(cred.category_id, (countMap.get(cred.category_id) || 0) + 1);
            }
        });

        // 递归重建节点名称（含数量），返回新数组触发响应式更新
        const attachCount = (nodes: Category[]): Category[] => {
            return nodes.map((n) => {
                const cnt = countMap.get(n.id) || 0;
                return {
                    ...n,
                    name: `${n.name} (${cnt})`,
                    children: n.children ? attachCount(n.children) : undefined,
                };
            });
        };
        categoryTree.value = attachCount(categoryTree.value);
    } catch (err) {
        console.error('加载凭证数量失败:', err);
    }
};

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

// ── File selection → parse CSV ──

const handleClickUpload = async () => {
    const filePath = await selectFile({
        title: '选择浏览器导出的 CSV 文件',
        extensions: ['csv'],
    });
    if (!filePath) return;

    parsing.value = true;
    try {
        const items = await importCsvPasswords(filePath);
        importedItems.value = items;
        passwordVisible.value = {};
    } catch (err: unknown) {
        ElMessage.error(err instanceof Error ? err.message : '解析 CSV 失败');
    } finally {
        parsing.value = false;
    }
};

// ── Dialog open → reset ──

watch(
    () => props.modelValue,
    (val) => {
        if (val) {
            resetState();
        }
    },
);

const resetState = () => {
    importedItems.value = [];
    passwordVisible.value = {};
    importedIds.value = new Set();
    selectedRows.value = [];
    selectAll.value = false;
    parsing.value = false;
    importing.value = false;
    importProgress.value = 0;
    activeNames.value = ['1'];
    selectedCredentialType.value = 'account';
    selectedCategoryId.value = null;
    fetchCategories();
};

const handleClosed = () => {
    resetState();
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
        if (selectedCategoryId.value !== null) {
            categoryId = selectedCategoryId.value;
        } else {
            categoryId = await resolveDefaultCategory();
        }
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
                credential_type: selectedCredentialType.value,
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
.usage-tips {
    margin-bottom: 16px;
}

.tip-list {
    margin: 0;
    padding-left: 20px;
    font-size: var(--app-font-13);
    line-height: 2;
    color: #4b5563;
}

.tip-list code {
    background: #f3f4f6;
    padding: 1px 4px;
    border-radius: 3px;
    font-size: var(--app-font-12);
    color: #1f2937;
}

.csv-upload-area {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 160px;
    border: 2px dashed #d1d5db;
    border-radius: 8px;
    cursor: pointer;
    transition:
        border-color 0.2s,
        background-color 0.2s;
    margin-bottom: 8px;
}

.csv-upload-area:hover {
    border-color: var(--el-color-primary, #409eff);
    background-color: rgba(64, 158, 255, 0.04);
}

.upload-icon {
    color: #9ca3af;
    margin-bottom: 8px;
}

.upload-text {
    font-size: var(--app-font-15);
    color: #374151;
    margin-bottom: 6px;
}

.upload-tip {
    font-size: var(--app-font-12);
    color: #9ca3af;
}

.import-config {
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--color-window-titlebar-border, #e5e7eb);
}

.config-row {
    display: flex;
    align-items: center;
    gap: 24px;
    flex-wrap: wrap;
}

.config-item {
    display: flex;
    align-items: center;
    gap: 8px;
}

.type-select {
    width: 140px;
}

.category-select {
    width: 220px;
}

.config-label {
    font-size: var(--app-font-13);
    color: #4b5563;
    white-space: nowrap;
}

.browser-loading {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 24px 0;
    color: #6b7280;
    font-size: var(--app-font-14);
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
