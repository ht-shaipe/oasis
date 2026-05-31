<template>
    <MacWindow :title="t('toolbox.title')" @close="emit('close')" @minimize="emit('minimize')">
        <div class="toolbox-container">
            <!-- 左侧工具导航 -->
            <aside class="toolbox-sidebar" :style="{ width: sidebarWidth + 'px' }">
                <div class="sidebar-header">
                    <span class="sidebar-title">{{ t('toolbox.title') }}</span>
                </div>
                <div
                    v-for="tool in tools"
                    :key="tool.id"
                    class="tool-nav-item"
                    :class="{ active: activeTool === tool.id }"
                    @click="activeTool = tool.id"
                >
                    <img :src="tool.icon" class="tool-icon" :alt="t(tool.labelKey)" />
                    <span class="tool-name">{{ t(tool.labelKey) }}</span>
                </div>
            </aside>

            <!-- 分隔条 -->
            <div
                class="split-handle"
                :class="{ dragging: isDragging }"
                @mousedown.prevent="startResize"
            />

            <!-- 右侧工具面板 -->
            <main class="toolbox-main">
                <Transition name="panel-fade" mode="out-in">
                    <!-- CSV 统计 -->
                    <div v-if="activeTool === 'csv-stats'" :key="'csv-stats'" class="tool-panel">
                        <div class="panel-header">
                            <img src="/assets/icons/CsvStats.svg" class="panel-icon" alt="" />
                            <h3>{{ t('toolbox.csvStats') }}</h3>
                        </div>
                        <div class="panel-body">
                            <div class="form-group">
                                <label class="form-label">{{ t('toolbox.dirPlaceholder') }}</label>
                                <el-input v-model="csvStatsDir" :placeholder="t('toolbox.dirPlaceholder')">
                                    <template #append>
                                        <el-button @click="runCsvStats" :loading="csvStatsLoading" type="primary">{{ t('toolbox.run') }}</el-button>
                                    </template>
                                </el-input>
                            </div>
                            <div v-if="csvStatsResult" class="result-area">
                                <div class="result-summary">
                                    <span class="result-badge">{{ t('toolbox.totalLines') }}: <strong>{{ csvStatsResult.total }}</strong></span>
                                </div>
                                <el-table :data="csvStatsResult.entries" max-height="300" size="small" stripe>
                                    <el-table-column prop="path" :label="t('toolbox.filePath')" show-overflow-tooltip />
                                    <el-table-column prop="lines" :label="t('toolbox.lines')" width="100" align="center" />
                                </el-table>
                            </div>
                        </div>
                    </div>

                    <!-- CSV 拆分 -->
                    <div v-if="activeTool === 'csv-split'" :key="'csv-split'" class="tool-panel">
                        <div class="panel-header">
                            <img src="/assets/icons/CsvSplit.svg" class="panel-icon" alt="" />
                            <h3>{{ t('toolbox.csvSplit') }}</h3>
                        </div>
                        <div class="panel-body">
                            <div class="form-group">
                                <label class="form-label">{{ t('toolbox.filePlaceholder') }}</label>
                                <el-input v-model="csvSplitPath" :placeholder="t('toolbox.filePlaceholder')" />
                            </div>
                            <div class="form-group">
                                <label class="form-label">{{ t('toolbox.outputDirPlaceholder') }}</label>
                                <el-input v-model="csvSplitDir" :placeholder="t('toolbox.outputDirPlaceholder')" />
                            </div>
                            <div class="form-group form-row-inline">
                                <label class="form-label">{{ t('toolbox.splitParts') }}</label>
                                <el-input-number v-model="csvSplitParts" :min="2" :max="100" size="default" />
                            </div>
                            <div class="form-group">
                                <el-button type="primary" @click="runCsvSplit">{{ t('toolbox.run') }}</el-button>
                            </div>
                            <p v-if="csvSplitMsg" class="result-msg">{{ csvSplitMsg }}</p>
                        </div>
                    </div>

                    <!-- 格式转换 -->
                    <div v-if="activeTool === 'csv-convert'" :key="'csv-convert'" class="tool-panel">
                        <div class="panel-header">
                            <img src="/assets/icons/CsvConvert.svg" class="panel-icon" alt="" />
                            <h3>{{ t('toolbox.formatConvert') }}</h3>
                        </div>
                        <div class="panel-body">
                            <div class="form-group">
                                <label class="form-label">{{ t('toolbox.inputFilePlaceholder') }}</label>
                                <el-input v-model="convertInput" :placeholder="t('toolbox.inputFilePlaceholder')" />
                            </div>
                            <div class="form-group">
                                <label class="form-label">{{ t('toolbox.outputFilePlaceholder') }}</label>
                                <el-input v-model="convertOutput" :placeholder="t('toolbox.outputFilePlaceholder')" />
                            </div>
                            <div class="form-group form-row-inline">
                                <label class="form-label">{{ t('toolbox.outputFormat') }}</label>
                                <el-select v-model="convertFormat" :placeholder="t('toolbox.outputFormat')" style="width: 160px">
                                    <el-option label="CSV" value="csv" />
                                    <el-option label="JSON" value="json" />
                                    <el-option label="SQL" value="sql" />
                                </el-select>
                            </div>
                            <div class="form-group">
                                <el-button type="primary" @click="runConvert">{{ t('toolbox.run') }}</el-button>
                            </div>
                            <p v-if="convertMsg" class="result-msg">{{ convertMsg }}</p>
                        </div>
                    </div>

                    <!-- Excel 匹配移动 -->
                    <div v-if="activeTool === 'excel-move'" :key="'excel-move'" class="tool-panel">
                        <div class="panel-header">
                            <img src="/assets/icons/ExcelMove.svg" class="panel-icon" alt="" />
                            <h3>{{ t('toolbox.excelMove') }}</h3>
                        </div>
                        <div class="panel-body">
                            <div class="form-group">
                                <label class="form-label">{{ t('toolbox.excelFilePlaceholder') }}</label>
                                <el-input v-model="excelPath" :placeholder="t('toolbox.excelFilePlaceholder')" />
                            </div>
                            <div class="form-group">
                                <label class="form-label">{{ t('toolbox.colHeaderPlaceholder') }}</label>
                                <el-input v-model="excelColHeader" :placeholder="t('toolbox.colHeaderPlaceholder')" />
                            </div>
                            <div class="form-row-split">
                                <div class="form-group">
                                    <label class="form-label">{{ t('toolbox.inputDirPlaceholder') }}</label>
                                    <el-input v-model="excelInputDir" :placeholder="t('toolbox.inputDirPlaceholder')" />
                                </div>
                                <div class="form-group">
                                    <label class="form-label">{{ t('toolbox.outputDirPlaceholder') }}</label>
                                    <el-input v-model="excelOutputDir" :placeholder="t('toolbox.outputDirPlaceholder')" />
                                </div>
                            </div>
                            <div class="form-group">
                                <label class="form-label">{{ t('toolbox.suffixesPlaceholder') }}</label>
                                <el-input v-model="excelSuffixes" :placeholder="t('toolbox.suffixesPlaceholder')" />
                            </div>
                            <div class="form-group">
                                <el-button @click="runExcelPreview" :loading="excelLoading">{{ t('toolbox.preview') }}</el-button>
                                <el-button type="primary" @click="runExcelApply" :disabled="!excelPlan">{{ t('toolbox.apply') }}</el-button>
                            </div>
                            <div v-if="excelPreview" class="result-area">
                                <div class="result-summary">
                                    <span class="result-badge">{{ t('toolbox.matchResult') }}: <strong>{{ excelPreview.found }}/{{ excelPreview.total }}</strong></span>
                                    <span class="result-badge">{{ t('toolbox.missing') }}: <strong>{{ excelPreview.missing }}</strong></span>
                                    <span class="result-badge">{{ t('toolbox.duplicate') }}: <strong>{{ excelPreview.duplicate }}</strong></span>
                                </div>
                                <el-table :data="excelPreview.items" max-height="300" size="small" stripe>
                                    <el-table-column prop="status" :label="t('toolbox.status')" width="80" align="center" />
                                    <el-table-column prop="file_name" :label="t('toolbox.fileName')" show-overflow-tooltip />
                                    <el-table-column prop="base" :label="t('toolbox.keyword')" show-overflow-tooltip />
                                </el-table>
                            </div>
                        </div>
                    </div>

                    <!-- JSON 转换 -->
                    <div v-if="activeTool === 'json-convert'" :key="'json-convert'" class="tool-panel">
                        <div class="panel-header">
                            <img src="/assets/icons/JsonConvert.svg" class="panel-icon" alt="" />
                            <h3>{{ t('toolbox.jsonConvert') }}</h3>
                        </div>
                        <div class="panel-body">
                            <div class="form-group">
                                <label class="form-label">{{ t('toolbox.jsonFilePlaceholder') }}</label>
                                <el-input v-model="jsonInput" :placeholder="t('toolbox.jsonFilePlaceholder')" />
                            </div>
                            <div class="form-row-split">
                                <div class="form-group">
                                    <label class="form-label">{{ t('toolbox.outputFilePlaceholder') }}</label>
                                    <el-input v-model="jsonOutput" :placeholder="t('toolbox.outputFilePlaceholder')" />
                                </div>
                                <div class="form-group">
                                    <label class="form-label">{{ t('toolbox.outputFormat') }}</label>
                                    <el-select v-model="jsonOutFormat" style="width: 100%">
                                        <el-option label="CSV" value="csv" />
                                        <el-option label="Excel" value="excel" />
                                    </el-select>
                                </div>
                            </div>
                            <div class="form-row-split">
                                <div class="form-group">
                                    <label class="form-label">{{ t('toolbox.jsonPathPlaceholder') }}</label>
                                    <el-input v-model="jsonPath" :placeholder="t('toolbox.jsonPathPlaceholder')" />
                                </div>
                                <div class="form-group">
                                    <label class="form-label">{{ t('toolbox.fieldsPlaceholder') }}</label>
                                    <el-input v-model="jsonFields" :placeholder="t('toolbox.fieldsPlaceholder')" />
                                </div>
                            </div>
                            <div class="form-group">
                                <el-button type="primary" @click="runJsonConvert">{{ t('toolbox.singleConvert') }}</el-button>
                            </div>
                            <p v-if="jsonMsg" class="result-msg">{{ jsonMsg }}</p>
                        </div>
                    </div>

                    <!-- JSON 合并 -->
                    <div v-if="activeTool === 'json-merge'" :key="'json-merge'" class="tool-panel">
                        <div class="panel-header">
                            <img src="/assets/icons/JsonMerge.svg" class="panel-icon" alt="" />
                            <h3>{{ t('toolbox.jsonMerge') }}</h3>
                        </div>
                        <div class="panel-body">
                            <div class="form-group">
                                <label class="form-label">{{ t('toolbox.inputDirPlaceholder') }}</label>
                                <el-input v-model="mergeInputDir" :placeholder="t('toolbox.inputDirPlaceholder')" />
                            </div>
                            <div class="form-group">
                                <label class="form-label">{{ t('toolbox.outputFilePlaceholder') }}</label>
                                <el-input v-model="mergeOutput" :placeholder="t('toolbox.outputFilePlaceholder')" />
                            </div>
                            <div class="form-group">
                                <label class="form-label">{{ t('toolbox.jsonPathPlaceholder') }}</label>
                                <el-input v-model="mergeJsonPath" :placeholder="t('toolbox.jsonPathPlaceholder')" />
                            </div>
                            <div class="form-group">
                                <el-button type="primary" @click="runJsonMerge">{{ t('toolbox.run') }}</el-button>
                            </div>
                            <p v-if="mergeMsg" class="result-msg">{{ mergeMsg }}</p>
                        </div>
                    </div>

                    <!-- 网络扫描 -->
                    <div v-if="activeTool === 'network-scan'" :key="'network-scan'" class="tool-panel">
                        <div class="panel-header">
                            <img src="/assets/icons/NetworkScan.svg" class="panel-icon" alt="" />
                            <h3>{{ t('toolbox.networkScan') }}</h3>
                        </div>
                        <div class="panel-body">
                            <div class="form-row-split">
                                <div class="form-group">
                                    <label class="form-label">{{ t('toolbox.ipRangePlaceholder') }}</label>
                                    <el-input v-model="scanIpRange" :placeholder="t('toolbox.ipRangePlaceholder')" />
                                </div>
                                <div class="form-group">
                                    <label class="form-label">{{ t('toolbox.portsPlaceholder') }}</label>
                                    <el-input v-model="scanPorts" :placeholder="t('toolbox.portsPlaceholder')" />
                                </div>
                            </div>
                            <div class="form-row-split">
                                <div class="form-group">
                                    <label class="form-label">{{ t('toolbox.timeout') }}</label>
                                    <div class="inline-controls">
                                        <el-input-number v-model="scanTimeout" :min="100" :max="10000" :step="100" size="default" />
                                        <span class="unit-text">ms</span>
                                    </div>
                                </div>
                                <div class="form-group">
                                    <label class="form-label">&nbsp;</label>
                                    <el-checkbox v-model="scanShowClosed">{{ t('toolbox.showClosed') }}</el-checkbox>
                                </div>
                            </div>
                            <div class="form-group">
                                <el-button type="primary" @click="runNetworkScan" :loading="scanLoading">{{ t('toolbox.startScan') }}</el-button>
                            </div>
                            <div v-if="scanResult" class="result-area">
                                <pre class="scan-result-text">{{ scanResult.format_text }}</pre>
                            </div>
                        </div>
                    </div>
                </Transition>
            </main>
        </div>
    </MacWindow>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import { useI18n } from 'vue-i18n';
import MacWindow from '@/components/common/MacWindow.vue';

const { t } = useI18n();

const emit = defineEmits<{
    close: [];
    minimize: [];
}>();

// ── 工具导航 ──
import CsvStatsIcon from '/assets/icons/CsvStats.svg';
import CsvSplitIcon from '/assets/icons/CsvSplit.svg';
import CsvConvertIcon from '/assets/icons/CsvConvert.svg';
import ExcelMoveIcon from '/assets/icons/ExcelMove.svg';
import JsonConvertIcon from '/assets/icons/JsonConvert.svg';
import JsonMergeIcon from '/assets/icons/JsonMerge.svg';
import NetworkScanIcon from '/assets/icons/NetworkScan.svg';

const tools = [
    { id: 'csv-stats', icon: CsvStatsIcon, labelKey: 'toolbox.csvStats' },
    { id: 'csv-split', icon: CsvSplitIcon, labelKey: 'toolbox.csvSplit' },
    { id: 'csv-convert', icon: CsvConvertIcon, labelKey: 'toolbox.formatConvert' },
    { id: 'excel-move', icon: ExcelMoveIcon, labelKey: 'toolbox.excelMove' },
    { id: 'json-convert', icon: JsonConvertIcon, labelKey: 'toolbox.jsonConvert' },
    { id: 'json-merge', icon: JsonMergeIcon, labelKey: 'toolbox.jsonMerge' },
    { id: 'network-scan', icon: NetworkScanIcon, labelKey: 'toolbox.networkScan' },
];
const activeTool = ref('csv-stats');

// ── 分隔条拖拽 ──
const SIDEBAR_MIN = 140;
const SIDEBAR_MAX = 360;
const SIDEBAR_KEY = 'toolbox-sidebar-width';

function loadSidebarWidth(): number {
    try {
        return parseInt(localStorage.getItem(SIDEBAR_KEY) || '', 10) || 200;
    } catch {
        return 200;
    }
}

const sidebarWidth = ref(loadSidebarWidth());
const isDragging = ref(false);
let dragStartX = 0;
let dragStartWidth = 0;

function startResize(e: MouseEvent) {
    isDragging.value = true;
    dragStartX = e.clientX;
    dragStartWidth = sidebarWidth.value;

    const onMove = (ev: MouseEvent) => {
        const delta = ev.clientX - dragStartX;
        sidebarWidth.value = Math.min(SIDEBAR_MAX, Math.max(SIDEBAR_MIN, dragStartWidth + delta));
    };
    const onUp = () => {
        isDragging.value = false;
        localStorage.setItem(SIDEBAR_KEY, String(sidebarWidth.value));
        document.removeEventListener('mousemove', onMove);
        document.removeEventListener('mouseup', onUp);
        document.body.style.cursor = '';
        document.body.style.userSelect = '';
    };

    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
}

// ── CSV 统计 ──
const csvStatsDir = ref('');
const csvStatsLoading = ref(false);
const csvStatsResult = ref<{ entries: { path: string; lines: number }[]; total: number } | null>(null);

async function runCsvStats() {
    if (!csvStatsDir.value) return ElMessage.warning(t('toolbox.dirRequired'));
    csvStatsLoading.value = true;
    try {
        csvStatsResult.value = await invoke('csv_scan_dir', { dir: csvStatsDir.value });
    } catch (e: any) {
        ElMessage.error(e);
    } finally {
        csvStatsLoading.value = false;
    }
}

// ── CSV 拆分 ──
const csvSplitPath = ref('');
const csvSplitDir = ref('');
const csvSplitParts = ref(2);
const csvSplitMsg = ref('');

async function runCsvSplit() {
    if (!csvSplitPath.value || !csvSplitDir.value) return ElMessage.warning(t('toolbox.pathRequired'));
    try {
        await invoke('csv_split_file', {
            inputPath: csvSplitPath.value,
            outputDir: csvSplitDir.value,
            parts: csvSplitParts.value,
        });
        csvSplitMsg.value = t('toolbox.splitDone');
    } catch (e: any) {
        csvSplitMsg.value = '';
        ElMessage.error(e);
    }
}

// ── 格式转换 ──
const convertInput = ref('');
const convertOutput = ref('');
const convertFormat = ref('json');
const convertMsg = ref('');

async function runConvert() {
    if (!convertInput.value || !convertOutput.value) return ElMessage.warning(t('toolbox.pathRequired'));
    try {
        await invoke('csv_convert_file', {
            params: {
                input_path: convertInput.value,
                output_path: convertOutput.value,
                format: convertFormat.value,
            },
        });
        convertMsg.value = t('toolbox.convertDone');
    } catch (e: any) {
        convertMsg.value = '';
        ElMessage.error(e);
    }
}

// ── Excel 匹配移动 ──
const excelPath = ref('');
const excelColHeader = ref('');
const excelInputDir = ref('');
const excelOutputDir = ref('');
const excelSuffixes = ref('.pdf,.docx,.xlsx');
const excelLoading = ref(false);
const excelPreview = ref<any>(null);
const excelPlan = ref(false);

async function runExcelPreview() {
    if (!excelPath.value || !excelInputDir.value) return ElMessage.warning(t('toolbox.pathRequired'));
    excelLoading.value = true;
    try {
        excelPreview.value = await invoke('excel_move_preview', {
            excelPath: excelPath.value,
            colHeader: excelColHeader.value,
            colIndex: 0,
            inputDir: excelInputDir.value,
            suffixes: excelSuffixes.value.split(',').map((s: string) => s.trim()).filter(Boolean),
            outputDir: excelOutputDir.value || excelInputDir.value,
        });
        excelPlan.value = true;
    } catch (e: any) {
        ElMessage.error(e);
    } finally {
        excelLoading.value = false;
    }
}

async function runExcelApply() {
    if (!excelPlan.value) return;
    try {
        const msg = await invoke<string>('excel_move_apply', {
            excelPath: excelPath.value,
            colHeader: excelColHeader.value,
            colIndex: 0,
            inputDir: excelInputDir.value,
            suffixes: excelSuffixes.value.split(',').map((s: string) => s.trim()).filter(Boolean),
            outputDir: excelOutputDir.value || excelInputDir.value,
        });
        ElMessage.success(msg);
    } catch (e: any) {
        ElMessage.error(e);
    }
}

// ── JSON 转换 ──
const jsonInput = ref('');
const jsonOutput = ref('');
const jsonOutFormat = ref('csv');
const jsonPath = ref('');
const jsonFields = ref('');
const jsonMsg = ref('');

async function runJsonConvert() {
    if (!jsonInput.value || !jsonOutput.value) return ElMessage.warning(t('toolbox.pathRequired'));
    try {
        await invoke('json_convert_file', {
            params: {
                input_path: jsonInput.value,
                output_path: jsonOutput.value,
                output_format: jsonOutFormat.value,
                json_path: jsonPath.value,
                fields: jsonFields.value.split(',').map((s: string) => s.trim()).filter(Boolean),
            },
        });
        jsonMsg.value = t('toolbox.convertDone');
    } catch (e: any) {
        jsonMsg.value = '';
        ElMessage.error(e);
    }
}

// ── JSON 合并 ──
const mergeInputDir = ref('');
const mergeOutput = ref('');
const mergeJsonPath = ref('');
const mergeMsg = ref('');

async function runJsonMerge() {
    if (!mergeInputDir.value || !mergeOutput.value) return ElMessage.warning(t('toolbox.pathRequired'));
    try {
        mergeMsg.value = await invoke<string>('json_merge_files', {
            inputDir: mergeInputDir.value,
            outputPath: mergeOutput.value,
            jsonPath: mergeJsonPath.value,
        });
    } catch (e: any) {
        mergeMsg.value = '';
        ElMessage.error(e);
    }
}

// ── 网络扫描 ──
const scanIpRange = ref('192.168.1.1-254');
const scanPorts = ref('80,443,22,8080');
const scanTimeout = ref(1000);
const scanShowClosed = ref(false);
const scanLoading = ref(false);
const scanResult = ref<{ format_text: string } | null>(null);

async function runNetworkScan() {
    scanLoading.value = true;
    try {
        scanResult.value = await invoke('network_scan_ports', {
            ipRange: scanIpRange.value,
            portsStr: scanPorts.value,
            timeoutMs: scanTimeout.value,
            showClosed: scanShowClosed.value,
        });
    } catch (e: any) {
        ElMessage.error(e);
    } finally {
        scanLoading.value = false;
    }
}

defineOptions({ name: 'ToolboxApp' });
</script>

<style scoped>
.toolbox-container {
    display: flex;
    height: 100%;
    background: var(--color-bg, #1e1e1e);
    color: var(--color-text, #ccc);
}

/* ── Sidebar ── */
.toolbox-sidebar {
    /* width controlled by inline :style */
    padding: 0;
    flex-shrink: 0;
    overflow-y: auto;
    overflow-x: hidden;
    background: var(--color-bg-elevated, #1a1a1a);
}

.sidebar-header {
    padding: 16px 16px 12px;
    border-bottom: 1px solid var(--color-border, #333);
    margin-bottom: 4px;
}

.sidebar-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: var(--color-text-muted, #666);
}

/* ── Split Handle ── */
.split-handle {
    width: 4px;
    background: var(--color-border, #333);
    cursor: col-resize;
    flex-shrink: 0;
    transition: background 0.15s;
    position: relative;
}
.split-handle::after {
    content: '';
    position: absolute;
    inset: 0 -4px;
}
.split-handle:hover,
.split-handle.dragging {
    background: var(--color-accent, #4a9eff);
}

.tool-nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 16px;
    margin: 0 6px 2px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    color: var(--color-text-secondary, #999);
    transition: all 0.15s ease;
}
.tool-nav-item:hover {
    background: var(--color-hover, #252525);
    color: var(--color-text, #ccc);
}
.tool-nav-item.active {
    background: var(--color-active, #094771);
    color: #fff;
}
.tool-icon {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    opacity: 0.7;
    transition: opacity 0.15s;
}
.tool-nav-item.active .tool-icon { opacity: 1; }
.tool-nav-item:hover .tool-icon { opacity: 0.9; }

/* ── Main ── */
.toolbox-main {
    flex: 1;
    padding: 24px 28px;
    overflow-y: auto;
    background: var(--color-bg, #1e1e1e);
}

/* ── Panel transitions ── */
.panel-fade-enter-active,
.panel-fade-leave-active {
    transition: opacity 0.15s ease, transform 0.15s ease;
}
.panel-fade-enter-from {
    opacity: 0;
    transform: translateY(6px);
}
.panel-fade-leave-to {
    opacity: 0;
    transform: translateY(-6px);
}

/* ── Tool Panel ── */
.tool-panel {
    max-width: 640px;
}

.panel-header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 24px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--color-border, #333);
}

.panel-header h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text, #ddd);
}

.panel-icon {
    width: 24px;
    height: 24px;
    opacity: 0.85;
}

.panel-body {
    display: flex;
    flex-direction: column;
    gap: 0;
}

/* ── Form Elements ── */
.form-group {
    margin-bottom: 16px;
}

.form-label {
    display: block;
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text-muted, #888);
    margin-bottom: 6px;
    letter-spacing: 0.3px;
}

.form-row-inline {
    display: flex;
    align-items: center;
    gap: 12px;
}
.form-row-inline .form-label {
    margin-bottom: 0;
    white-space: nowrap;
}

.form-row-split {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin-bottom: 16px;
}
.form-row-split .form-group {
    margin-bottom: 0;
}

.inline-controls {
    display: flex;
    align-items: center;
    gap: 6px;
}

.unit-text {
    font-size: 12px;
    color: var(--color-text-muted, #888);
}

/* ── Results ── */
.result-area {
    margin-top: 16px;
    padding: 16px;
    background: var(--color-bg-elevated, #1a1a1a);
    border-radius: 8px;
    border: 1px solid var(--color-border, #333);
}

.result-summary {
    display: flex;
    gap: 16px;
    margin-bottom: 12px;
    flex-wrap: wrap;
}

.result-badge {
    font-size: 13px;
    color: var(--color-text, #ccc);
}
.result-badge strong {
    color: var(--color-accent, #4a9eff);
}

.result-msg {
    color: var(--color-success, #67c23a);
    margin-top: 12px;
    font-size: 13px;
    font-weight: 500;
}

.scan-result-text {
    background: var(--color-code-bg, #111);
    padding: 14px;
    border-radius: 6px;
    font-size: 12px;
    line-height: 1.6;
    overflow-x: auto;
    white-space: pre;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
    color: var(--color-text, #ccc);
    border: 1px solid var(--color-border, #333);
}
</style>