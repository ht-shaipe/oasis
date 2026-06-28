<template>
    <AppDialog
        v-model="visible"
        title="从浏览器导入网站密码"
        width="750px"
        append-to-body
        destroy-on-close
        @closed="handleClosed">
        <!-- ═══ Step 1: Choose import method ═══ -->
        <template v-if="!importedItems.length && !parsing">
            <!-- Import method tabs -->
            <div class="import-method-tabs">
                <div
                    class="method-tab"
                    :class="{ active: importMethod === 'direct' }"
                    @click="importMethod = 'direct'">
                    <el-icon :size="20"><Monitor /></el-icon>
                    <span>直接提取</span>
                </div>
                <div
                    class="method-tab"
                    :class="{ active: importMethod === 'csv' }"
                    @click="importMethod = 'csv'">
                    <el-icon :size="20"><Document /></el-icon>
                    <span>CSV 文件</span>
                </div>
            </div>

            <!-- ── Direct extraction ── -->
            <template v-if="importMethod === 'direct'">
                <div v-if="scanningBrowsers" class="browser-loading">
                    <el-icon class="is-loading" :size="20"><Loading /></el-icon>
                    <span>正在扫描浏览器...</span>
                </div>

                <div v-else-if="fdaStatus && !fdaStatus.has_access" class="no-browser-tip fda-tip">
                    <el-icon :size="32" color="#f59e0b"><WarningFilled /></el-icon>
                    <p class="fda-title">{{ $t('browserImport.fdaRequired') }}</p>
                    <p class="fda-desc">{{ $t('browserImport.fdaDescription') }}</p>
                </div>

                <div v-else-if="browsers.length === 0" class="no-browser-tip">
                    <el-icon :size="32" color="#9ca3af"><WarningFilled /></el-icon>
                    <p>{{ $t('browserImport.noBrowser') }}</p>
                </div>

                <div v-else class="browser-list">
                    <div
                        v-for="browser in browsers"
                        :key="browser.key"
                        class="browser-card"
                        :class="{ selected: selectedBrowserKey === browser.key, extracting: extractingKey === browser.key }"
                        @click="handleSelectBrowser(browser)">
                        <div class="browser-icon">
                            <el-icon :size="28"><Monitor /></el-icon>
                        </div>
                        <div class="browser-info">
                            <div class="browser-name">{{ browser.name }}</div>
                            <div class="browser-profiles">
                                {{ browser.profiles.length }} 个 Profile
                            </div>
                        </div>
                        <div v-if="extractingKey === browser.key" class="browser-extracting">
                            <el-icon class="is-loading" :size="16"><Loading /></el-icon>
                        </div>
                        <el-icon v-else-if="selectedBrowserKey === browser.key" :size="18" color="var(--el-color-primary)"><Check /></el-icon>
                    </div>
                </div>
            </template>

            <!-- ── CSV upload ── -->
            <template v-if="importMethod === 'csv'">
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

                <div class="csv-upload-area" @click="handleClickUpload">
                    <el-icon class="upload-icon" :size="40"><UploadFilled /></el-icon>
                    <div class="upload-text">点击选择 CSV 文件</div>
                    <div class="upload-tip">支持 Chrome / Edge / Firefox / Brave / Safari 导出的 CSV 文件</div>
                </div>
            </template>
        </template>

        <!-- ═══ Parsing loading ═══ -->
        <div v-if="parsing" class="browser-loading">
            <el-icon class="is-loading" :size="20"><Loading /></el-icon>
            <span>{{ parsingText }}</span>
        </div>

        <!-- ═══ Step 2: Results ═══ -->
        <template v-if="importedItems.length">
            <div class="import-source-tag">
                <el-tag size="small" type="info">
                    来自 {{ sourceBrowserName || 'CSV' }} · 共 {{ importedItems.length }} 条
                    <template v-if="filterIntranet && intranetCount > 0">
                        · 已过滤内网 {{ intranetCount }} 条 · 显示 {{ filteredItems.length }} 条
                    </template>
                </el-tag>
            </div>

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

            <el-table :data="filteredItems" ref="tableRef" max-height="400" @selection-change="handleSelectionChange">
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
                <div class="flex items-center gap-4">
                    <el-checkbox v-model="selectAll" :indeterminate="isIndeterminate" @change="handleSelectAllChange">
                        全选 / 取消全选
                    </el-checkbox>
                    <el-checkbox v-model="filterIntranet">
                        过滤内网地址
                        <template v-if="intranetCount > 0">
                            ({{ intranetCount }} 条)
                        </template>
                    </el-checkbox>
                </div>
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
import { View, Hide, Loading, UploadFilled, Monitor, Document, Check, WarningFilled } from '@element-plus/icons-vue';
import { Category, useCredential } from '@/composables/useCredential';
import { useFileDialog } from '@/composables/useFileDialog';
import { credentialTemplateOptions, type CredentialTemplateKey } from './credentialForm';
import AppDialog from '@/components/common/AppDialog.vue';

interface BrowserProfile {
    name: string;
    path: string;
}

interface BrowserInfo {
    key: string;
    name: string;
    kind: string;
    user_data_dir: string;
    profiles: BrowserProfile[];
}

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

onMounted(() => {
    fetchCategories();
    scanBrowsers();
});

const {
    importCsvPasswords,
    batchImportCredentials,
    listCategories,
    listCredentials,
    createCategory,
    discoverBrowsers,
    extractBrowserPasswords,
} = useCredential();
const { selectFile } = useFileDialog();

// ── Import method ──

const importMethod = ref<'direct' | 'csv'>('direct');

// ── Browser scanning ──

const scanningBrowsers = ref(false);
const browsers = ref<BrowserInfo[]>([]);
const selectedBrowserKey = ref<string | null>(null);
const extractingKey = ref<string | null>(null);
const sourceBrowserName = ref('');
const fdaStatus = ref<{ has_access: boolean; message: string } | null>(null);

const scanBrowsers = async () => {
    scanningBrowsers.value = true;
    try {
        fdaStatus.value = await (window as any).__TAURI__.core.invoke('check_fda_status');
        browsers.value = await discoverBrowsers();
    } catch (err) {
        console.error('扫描浏览器失败:', err);
        browsers.value = [];
    } finally {
        scanningBrowsers.value = false;
    }
};

const handleSelectBrowser = async (browser: BrowserInfo) => {
    if (extractingKey.value) return;
    selectedBrowserKey.value = browser.key;
    extractingKey.value = browser.key;
    parsing.value = true;
    parsingText.value = `正在从 ${browser.name} 提取密码...`;

    try {
        const logins = await extractBrowserPasswords(browser.key);
        sourceBrowserName.value = browser.name;
        importedItems.value = logins.map((item, index) => ({
            id: index,
            url: item.url,
            username: item.username,
            password: item.password,
            browser: browser.key,
        }));
        passwordVisible.value = {};

        if (importedItems.value.length === 0) {
            ElMessage.info(`${browser.name} 中未找到保存的密码`);
            importedItems.value = [];
        }
    } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        ElMessage.error(`提取失败: ${msg}`);
        selectedBrowserKey.value = null;
    } finally {
        extractingKey.value = null;
        parsing.value = false;
    }
};

// ── Usage tips collapse ──

const activeNames = ref(['1']);

// ── CSV upload ──

const handleClickUpload = async () => {
    const filePath = await selectFile({
        title: '选择浏览器导出的 CSV 文件',
        extensions: ['csv'],
    });
    if (!filePath) return;

    parsing.value = true;
    parsingText.value = '正在解析 CSV 文件...';
    try {
        const items = await importCsvPasswords(filePath);
        sourceBrowserName.value = '';
        importedItems.value = items;
        passwordVisible.value = {};
    } catch (err: unknown) {
        ElMessage.error(err instanceof Error ? err.message : '解析 CSV 失败');
    } finally {
        parsing.value = false;
    }
};

// ── Loading state ──

const parsing = ref(false);
const parsingText = ref('');

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

const isUrlIntranet = (url: string): boolean => {
    const lower = url.toLowerCase();
    const stripped = lower
        .replace(/^https?:\/\//, '')
        .replace(/^www\./, '');
    const host = stripped.split('/')[0].split(':')[0];
    return /^localhost/i.test(host)
        || /^127\./.test(host)
        || /^0\./.test(host)
        || /^10\./.test(host)
        || /^192\.168\./.test(host)
        || /^172\.(1[6-9]|2\d|3[01])\./.test(host)
        || /\.local$/i.test(host)
        || /\.internal$/i.test(host)
        || /::1/.test(host)
        || !host.includes('.');
};

const intranetCount = computed(() => importedItems.value.filter((item) => isUrlIntranet(item.url)).length);
const filteredItems = computed(() =>
    filterIntranet.value
        ? importedItems.value.filter((item) => !isUrlIntranet(item.url))
        : importedItems.value,
);

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
    parsingText.value = '';
    importing.value = false;
    importProgress.value = 0;
    activeNames.value = ['1'];
    selectedCredentialType.value = 'account';
    selectedCategoryId.value = null;
    importMethod.value = 'direct';
    selectedBrowserKey.value = null;
    extractingKey.value = null;
    sourceBrowserName.value = '';
    fetchCategories();
    scanBrowsers();
};

const handleClosed = () => {
    resetState();
};

// ── Import ──

const importing = ref(false);
const importProgress = ref(0);
const filterIntranet = ref(true);

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

    try {
        const itemsToSend = filterIntranet.value
            ? toImport.filter((item) => !isUrlIntranet(item.url))
            : toImport;

        if (itemsToSend.length === 0) {
            ElMessage.info('选中项均为内网地址，已过滤');
            importing.value = false;
            return;
        }

        const result = await batchImportCredentials(
            itemsToSend.map((item) => ({
                url: item.url,
                username: item.username,
                password: item.password,
            })),
            categoryId,
            selectedCredentialType.value,
            false,
        );

        importProgress.value = 100;

        const intranetFilteredCount = toImport.length - itemsToSend.length;
        const parts: string[] = [];
        if (result.imported > 0) parts.push(`成功 ${result.imported} 条`);
        if (intranetFilteredCount > 0) parts.push(`内网跳过 ${intranetFilteredCount} 条`);
        if (result.skipped_empty > 0) parts.push(`空数据跳过 ${result.skipped_empty} 条`);
        if (result.failed > 0) parts.push(`失败 ${result.failed} 条`);

        if (result.imported > 0) {
            ElMessage.success(parts.join('，'));
            toImport.forEach((item) => {
                if (!filterIntranet.value || !isUrlIntranet(item.url)) {
                    importedIds.value = new Set([...importedIds.value, item.id]);
                }
            });
        } else if (intranetFilteredCount > 0 || result.skipped_empty > 0) {
            ElMessage.info(parts.join('，'));
        } else {
            ElMessage.error(parts.join('，') || '导入失败');
        }
    } catch (err: unknown) {
        ElMessage.error(err instanceof Error ? err.message : '批量导入失败');
    } finally {
        importing.value = false;
    }

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
.import-method-tabs {
    display: flex;
    gap: 12px;
    margin-bottom: 16px;
}

.method-tab {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 12px 16px;
    border: 2px solid #e5e7eb;
    border-radius: 8px;
    cursor: pointer;
    transition: border-color 0.2s, background-color 0.2s;
    font-size: var(--app-font-14);
    color: #4b5563;
}

.method-tab:hover {
    border-color: var(--el-color-primary-light-3, #79bbff);
    background-color: rgba(64, 158, 255, 0.04);
}

.method-tab.active {
    border-color: var(--el-color-primary, #409eff);
    background-color: rgba(64, 158, 255, 0.06);
    color: var(--el-color-primary, #409eff);
}

.browser-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.browser-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border: 2px solid #e5e7eb;
    border-radius: 8px;
    cursor: pointer;
    transition: border-color 0.2s, background-color 0.2s;
}

.browser-card:hover {
    border-color: var(--el-color-primary-light-3, #79bbff);
    background-color: rgba(64, 158, 255, 0.04);
}

.browser-card.selected {
    border-color: var(--el-color-primary, #409eff);
    background-color: rgba(64, 158, 255, 0.06);
}

.browser-card.extracting {
    cursor: wait;
    opacity: 0.7;
}

.browser-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    background: #f3f4f6;
    border-radius: 8px;
    color: #6b7280;
}

.browser-info {
    flex: 1;
}

.browser-name {
    font-size: var(--app-font-14);
    font-weight: 500;
    color: #1f2937;
}

.browser-profiles {
    font-size: var(--app-font-12);
    color: #9ca3af;
    margin-top: 2px;
}

.browser-extracting {
    color: var(--el-color-primary, #409eff);
}

.no-browser-tip {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 32px 0;
    color: #9ca3af;
    font-size: var(--app-font-14);
}

.fda-tip {
    gap: 12px;
}

.fda-title {
    font-size: var(--app-font-15);
    font-weight: 500;
    color: #f59e0b;
    margin: 0;
}

.fda-desc {
    font-size: var(--app-font-13);
    color: #6b7280;
    text-align: center;
    max-width: 400px;
    line-height: 1.6;
    margin: 0;
}

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

.import-source-tag {
    margin-bottom: 12px;
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
