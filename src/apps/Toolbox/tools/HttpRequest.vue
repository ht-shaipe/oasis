<template>
    <div class="tool-panel http-request-panel">
        <!-- Top Bar: Method + URL + Send -->
        <div class="request-bar">
            <el-select v-model="method" class="method-select" size="large">
                <el-option label="GET" value="GET" />
                <el-option label="POST" value="POST" />
                <el-option label="PUT" value="PUT" />
                <el-option label="DELETE" value="DELETE" />
                <el-option label="PATCH" value="PATCH" />
                <el-option label="HEAD" value="HEAD" />
            </el-select>
            <el-input
                v-model="url"
                size="large"
                class="url-input"
                :placeholder="t('toolbox.httpRequest.urlPlaceholder')"
                @keyup.enter="send" />
            <el-button
                type="primary"
                size="large"
                class="send-btn"
                :loading="loading"
                @click="send">
                {{ loading ? t('toolbox.httpRequest.sending') : t('toolbox.httpRequest.send') }}
            </el-button>
        </div>

        <!-- Request Tabs -->
        <div class="tabs-container">
            <div class="tabs-header">
                <div
                    v-for="tab in requestTabs"
                    :key="tab.id"
                    class="tab-item"
                    :class="{ active: activeRequestTab === tab.id }"
                    @click="activeRequestTab = tab.id">
                    {{ tab.label }}
                    <span v-if="tab.id === 'headers' && headerCount > 0" class="tab-badge">{{ headerCount }}</span>
                </div>
            </div>

            <div class="tab-content">
                <!-- Params Tab -->
                <div v-if="activeRequestTab === 'params'" class="tab-pane">
                    <div class="kv-table">
                        <div class="kv-header">
                            <span class="kv-col-key">KEY</span>
                            <span class="kv-col-value">VALUE</span>
                            <span class="kv-col-desc">DESCRIPTION</span>
                            <span class="kv-col-action"></span>
                        </div>
                        <div v-for="(param, index) in queryParams" :key="index" class="kv-row">
                            <el-input v-model="param.key" size="small" :placeholder="t('toolbox.httpRequest.headerKey')" />
                            <el-input v-model="param.value" size="small" :placeholder="t('toolbox.httpRequest.headerValue')" />
                            <el-input v-model="param.description" size="small" placeholder="Description" />
                            <button class="kv-remove" @click="removeQueryParam(index)" :disabled="queryParams.length <= 1">×</button>
                        </div>
                    </div>
                </div>

                <!-- Headers Tab -->
                <div v-if="activeRequestTab === 'headers'" class="tab-pane">
                    <div class="kv-table">
                        <div class="kv-header">
                            <span class="kv-col-key">KEY</span>
                            <span class="kv-col-value">VALUE</span>
                            <span class="kv-col-desc">DESCRIPTION</span>
                            <span class="kv-col-action"></span>
                        </div>
                        <div v-for="(header, index) in headers" :key="index" class="kv-row">
                            <el-input v-model="header.key" size="small" :placeholder="t('toolbox.httpRequest.headerKey')" />
                            <el-input v-model="header.value" size="small" :placeholder="t('toolbox.httpRequest.headerValue')" />
                            <el-input v-model="header.description" size="small" placeholder="Description" />
                            <button class="kv-remove" @click="removeHeader(index)" :disabled="headers.length <= 1">×</button>
                        </div>
                    </div>
                    <el-button text size="small" class="add-row-btn" @click="addHeader">
                        + {{ t('toolbox.httpRequest.addHeader') }}
                    </el-button>
                </div>

                <!-- Body Tab -->
                <div v-if="activeRequestTab === 'body'" class="tab-pane body-tab">
                    <div v-if="method === 'GET' || method === 'HEAD'" class="body-disabled">
                        {{ t('toolbox.httpRequest.bodyDisabled') }}
                    </div>
                    <template v-else>
                        <div class="body-type-bar">
                            <div
                                v-for="bt in bodyTypes"
                                :key="bt"
                                class="body-type-item"
                                :class="{ active: bodyType === bt }"
                                @click="bodyType = bt">
                                {{ bt }}
                            </div>
                        </div>
                        <el-input
                            v-model="body"
                            type="textarea"
                            :rows="10"
                            class="body-textarea"
                            :placeholder="t('toolbox.httpRequest.bodyPlaceholder')" />
                    </template>
                </div>

                <!-- Auth Tab -->
                <div v-if="activeRequestTab === 'auth'" class="tab-pane auth-pane">
                    <div class="auth-type-select">
                        <el-select v-model="authType" style="width: 200px">
                            <el-option label="No Auth" value="none" />
                            <el-option label="Bearer Token" value="bearer" />
                            <el-option label="Basic Auth" value="basic" />
                            <el-option label="API Key" value="apikey" />
                        </el-select>
                    </div>
                    <div v-if="authType === 'bearer'" class="auth-fields">
                        <div class="auth-field-row">
                            <span class="auth-label">Token</span>
                            <el-input v-model="authToken" size="small" placeholder="Enter bearer token" />
                        </div>
                    </div>
                    <div v-if="authType === 'basic'" class="auth-fields">
                        <div class="auth-field-row">
                            <span class="auth-label">Username</span>
                            <el-input v-model="authUsername" size="small" placeholder="Username" />
                        </div>
                        <div class="auth-field-row">
                            <span class="auth-label">Password</span>
                            <el-input v-model="authPassword" size="small" type="password" placeholder="Password" show-password />
                        </div>
                    </div>
                    <div v-if="authType === 'apikey'" class="auth-fields">
                        <div class="auth-field-row">
                            <span class="auth-label">Key</span>
                            <el-input v-model="authApiKey" size="small" placeholder="API Key name" />
                        </div>
                        <div class="auth-field-row">
                            <span class="auth-label">Value</span>
                            <el-input v-model="authApiValue" size="small" placeholder="API Key value" />
                        </div>
                        <div class="auth-field-row">
                            <span class="auth-label">Add to</span>
                            <el-select v-model="authApiIn" style="width: 150px">
                                <el-option label="Header" value="header" />
                                <el-option label="Query Param" value="query" />
                            </el-select>
                        </div>
                    </div>
                </div>

            </div>
        </div>

        <!-- Response Section -->
        <div class="response-section">
            <!-- Error Message -->
            <div v-if="error" class="response-error">
                <div class="error-title">{{ t('toolbox.httpRequest.requestFailed') }}</div>
                <div class="error-message">{{ error }}</div>
            </div>

            <div v-if="response" class="response-status-bar">
                <div class="status-left">
                    <span class="status-code" :class="getStatusClass(response.status)">
                        {{ response.status || 'Error' }} {{ response.statusText }}
                    </span>
                    <span class="response-meta">
                        {{ response.duration }}{{ t('toolbox.httpRequest.ms') }}
                        <template v-if="response.size > 0">
                            · {{ formatSize(response.size) }}
                        </template>
                    </span>
                </div>
                <div class="status-right">
                    <el-button v-if="response.body" text size="small" @click="copyResponse">
                        {{ t('app.copy') }}
                    </el-button>
                </div>
            </div>

            <div v-if="response" class="response-tabs">
                <div class="tabs-header">
                    <div
                        v-for="rt in responseTabs"
                        :key="rt"
                        class="tab-item"
                        :class="{ active: activeResponseTab === rt }"
                        @click="activeResponseTab = rt">
                        {{ rt }}
                    </div>
                </div>
            </div>

            <div v-if="response && activeResponseTab === 'Pretty'" class="response-body">
                <pre class="response-pre">{{ formattedBody }}</pre>
            </div>
            <div v-if="response && activeResponseTab === 'Raw'" class="response-body">
                <pre class="response-pre raw-body">{{ response.body }}</pre>
            </div>
            <div v-if="response && activeResponseTab === 'Headers'" class="response-body">
                <div class="response-headers-list">
                    <div v-for="(value, key) in response.headers" :key="key" class="response-header-row">
                        <span class="rh-key">{{ key }}</span>
                        <span class="rh-value">{{ value }}</span>
                    </div>
                </div>
            </div>

            <div v-if="!response && !loading" class="response-empty">
                {{ t('toolbox.httpRequest.noResponse') }}
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { ElMessage } from 'element-plus';
import { useHttpRequest } from '../composables/tools/useHttpRequest';

const { t } = useI18n();
const { method, url, headers, body, loading, response, error, addHeader, removeHeader, send } = useHttpRequest();

const activeRequestTab = ref('params');
const activeResponseTab = ref('Pretty');
const bodyType = ref('JSON');
const authType = ref('none');
const authToken = ref('');
const authUsername = ref('');
const authPassword = ref('');
const authApiKey = ref('');
const authApiValue = ref('');
const authApiIn = ref('header');

const requestTabs = [
    { id: 'params', label: 'Params' },
    { id: 'auth', label: 'Auth' },
    { id: 'headers', label: 'Headers' },
    { id: 'body', label: 'Body' },
];

const responseTabs = ['Pretty', 'Raw', 'Headers'];
const bodyTypes = ['JSON', 'Form Data', 'Raw', 'Binary'];

const headerCount = computed(() => headers.value.filter((h) => h.key.trim()).length);

const queryParams = ref<Array<{ key: string; value: string; description: string }>>([
    { key: '', value: '', description: '' },
]);

function removeQueryParam(index: number) {
    queryParams.value.splice(index, 1);
}

const formattedBody = computed(() => {
    if (!response.value?.body) return '';
    try {
        const parsed = JSON.parse(response.value.body);
        return JSON.stringify(parsed, null, 2);
    } catch {
        return response.value.body;
    }
});

function getStatusClass(status: number): string {
    if (status === 0) return 'status-error';
    if (status >= 200 && status < 300) return 'status-success';
    if (status >= 300 && status < 400) return 'status-redirect';
    if (status >= 400 && status < 500) return 'status-client-error';
    return 'status-server-error';
}

function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

async function copyResponse() {
    if (!response.value?.body) return;
    try {
        await navigator.clipboard.writeText(response.value.body);
        ElMessage.success(t('toolbox.httpRequest.copied'));
    } catch {
        ElMessage.error(t('toolbox.httpRequest.copyFailed'));
    }
}
</script>

<style scoped>
.http-request-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
}

/* ── Request Bar ── */
.request-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
    flex-shrink: 0;
}

.method-select {
    width: 110px;
    flex-shrink: 0;
}

.method-select :deep(.el-input__wrapper) {
    border-radius: 6px 0 0 6px;
}

.url-input {
    flex: 1;
}

.url-input :deep(.el-input__wrapper) {
    border-radius: 0;
}

.send-btn {
    flex-shrink: 0;
    border-radius: 0 6px 6px 0;
}

/* ── Tabs ── */
.tabs-container {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    border: 1px solid var(--color-border, rgba(0, 0, 0, 0.08));
    border-radius: 6px;
    overflow: hidden;
}

.tabs-header {
    display: flex;
    align-items: center;
    gap: 0;
    border-bottom: 1px solid var(--color-border, rgba(0, 0, 0, 0.08));
    background: var(--color-sidebar-bg, rgba(245, 245, 245, 0.6));
    padding: 0 8px;
    flex-shrink: 0;
}

.tab-item {
    padding: 8px 14px;
    font-size: var(--app-font-13);
    color: var(--color-text-secondary, #666);
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition: all 0.15s;
    user-select: none;
    display: flex;
    align-items: center;
    gap: 4px;
}

.tab-item:hover {
    color: var(--color-text-primary, #333);
}

.tab-item.active {
    color: var(--color-primary, #409eff);
    border-bottom-color: var(--color-primary, #409eff);
}

.tab-badge {
    background: var(--color-primary, #409eff);
    color: #fff;
    font-size: 10px;
    padding: 1px 5px;
    border-radius: 8px;
    line-height: 1.4;
}

.tab-content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 12px;
}

.tab-pane {
    height: 100%;
}

.tab-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-tertiary, #999);
    font-size: var(--app-font-13);
    padding: 40px 0;
}

/* ── KV Table ── */
.kv-table {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.kv-header {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr 28px;
    gap: 8px;
    padding: 4px 0;
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text-tertiary, #999);
    letter-spacing: 0.5px;
    text-transform: uppercase;
}

.kv-row {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr 28px;
    gap: 8px;
    align-items: center;
}

.kv-remove {
    width: 24px;
    height: 24px;
    border: none;
    background: none;
    color: var(--color-text-tertiary, #999);
    font-size: 16px;
    cursor: pointer;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
}

.kv-remove:hover {
    color: #f56c6c;
    background: rgba(245, 108, 108, 0.1);
}

.add-row-btn {
    margin-top: 8px;
    font-size: var(--app-font-12);
}

/* ── Body ── */
.body-type-bar {
    display: flex;
    gap: 0;
    margin-bottom: 8px;
    border-bottom: 1px solid var(--color-border, rgba(0, 0, 0, 0.08));
}

.body-type-item {
    padding: 6px 14px;
    font-size: var(--app-font-12);
    color: var(--color-text-secondary, #666);
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition: all 0.15s;
}

.body-type-item:hover {
    color: var(--color-text-primary, #333);
}

.body-type-item.active {
    color: var(--color-primary, #409eff);
    border-bottom-color: var(--color-primary, #409eff);
}

.body-textarea :deep(.el-textarea__inner) {
    font-family: 'SF Mono', 'Monaco', 'Menlo', monospace;
    font-size: var(--app-font-13);
}

.body-disabled {
    color: var(--color-text-tertiary, #999);
    font-size: var(--app-font-13);
    padding: 20px 0;
    text-align: center;
}

.body-tab {
    display: flex;
    flex-direction: column;
}

.body-tab .body-textarea {
    flex: 1;
}

.body-tab .body-textarea :deep(.el-textarea__inner) {
    height: 100% !important;
    min-height: 200px;
}

/* ── Auth ── */
.auth-type-select {
    margin-bottom: 16px;
}

.auth-fields {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.auth-field-row {
    display: grid;
    grid-template-columns: 100px 1fr;
    gap: 12px;
    align-items: center;
}

.auth-label {
    font-size: var(--app-font-13);
    color: var(--color-text-secondary, #666);
    font-weight: 500;
}

/* ── Response ── */
.response-section {
    flex-shrink: 0;
    border-top: 1px solid var(--color-border, rgba(0, 0, 0, 0.08));
    margin-top: 12px;
    padding-top: 12px;
}

.response-error {
    background: rgba(245, 108, 108, 0.08);
    border: 1px solid rgba(245, 108, 108, 0.2);
    border-radius: 6px;
    padding: 12px 16px;
    margin-bottom: 12px;
}

.error-title {
    font-size: var(--app-font-13);
    font-weight: 600;
    color: #f56c6c;
    margin-bottom: 6px;
}

.error-message {
    font-size: var(--app-font-12);
    color: var(--color-text-secondary, #666);
    word-break: break-all;
    line-height: 1.5;
}

.response-status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--color-sidebar-bg, rgba(245, 245, 245, 0.6));
    border-radius: 6px;
    margin-bottom: 8px;
}

.status-left {
    display: flex;
    align-items: center;
    gap: 12px;
}

.status-code {
    font-weight: 600;
    font-size: var(--app-font-13);
}

.status-success { color: #2d8c3c; }
.status-redirect { color: #e6a23c; }
.status-client-error { color: #f56c6c; }
.status-server-error { color: #f56c6c; }
.status-error { color: #f56c6c; }

.response-meta {
    font-size: var(--app-font-12);
    color: var(--color-text-tertiary, #999);
}

.response-tabs {
    margin-bottom: 8px;
}

.response-tabs .tabs-header {
    background: none;
    border-bottom: 1px solid var(--color-border, rgba(0, 0, 0, 0.08));
    padding: 0;
}

.response-body {
    max-height: 220px;
    overflow-y: auto;
}

.response-pre {
    background: var(--color-sidebar-bg, rgba(245, 245, 245, 0.6));
    border-radius: 6px;
    padding: 12px;
    font-size: var(--app-font-12);
    font-family: 'SF Mono', 'Monaco', 'Menlo', monospace;
    white-space: pre-wrap;
    word-break: break-all;
    margin: 0;
    color: var(--color-text-primary, #333);
    line-height: 1.5;
}

.raw-body {
    white-space: pre-wrap;
}

.response-headers-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.response-header-row {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: 12px;
    padding: 4px 8px;
    font-size: var(--app-font-12);
    font-family: 'SF Mono', 'Monaco', 'Menlo', monospace;
    border-radius: 4px;
}

.response-header-row:nth-child(even) {
    background: var(--color-sidebar-bg, rgba(245, 245, 245, 0.4));
}

.rh-key {
    color: var(--color-text-secondary, #666);
    font-weight: 500;
}

.rh-value {
    color: var(--color-text-primary, #333);
    word-break: break-all;
}

.response-empty {
    text-align: center;
    color: var(--color-text-tertiary, #999);
    font-size: var(--app-font-13);
    padding: 32px 0;
}
</style>
