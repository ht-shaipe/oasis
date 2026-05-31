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
                                v-if="filteredCredentialTotal === 0 && !tableLoading"
                                :description="t('credential.list.empty')" />
                            <el-table
                                v-else
                                v-loading="tableLoading"
                                :data="displayCredentials"
                                style="width: 100%"
                                @row-dblclick="handleViewCredential">
                                <el-table-column :label="t('credential.list.title')" min-width="200">
                                    <template #default="{ row }">
                                        <div class="cred-title">
                                            <el-icon><Key /></el-icon>
                                            <span>{{ row.title }}</span>
                                        </div>
                                    </template>
                                </el-table-column>
                                <el-table-column :label="t('credential.list.username')" min-width="120">
                                    <template #default="{ row }">
                                        <div class="truncate" :title="row.username || '-'">
                                            {{ row.username || '-' }}
                                        </div>
                                    </template>
                                </el-table-column>
                                <el-table-column :label="t('credential.list.url')" min-width="180">
                                    <template #default="{ row }">
                                        <div class="truncate" :title="row.url || '-'">
                                            {{ row.url || '-' }}
                                        </div>
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
                                <el-table-column :label="t('credential.list.actions')" width="120" fixed="right">
                                    <template #default="{ row }">
                                        <el-button type="primary" link size="small" @click="handleViewCredential(row)">
                                            <el-icon :size="16"><View /></el-icon>
                                        </el-button>
                                        <el-button link size="small" @click="openEditDialog(row)">
                                            <el-icon :size="16"><Edit /></el-icon>
                                        </el-button>
                                        <el-button link size="small" type="danger" @click="handleDeleteCredential(row)">
                                            <el-icon :size="16"><Delete /></el-icon>
                                        </el-button>
                                    </template>
                                </el-table-column>
                            </el-table>
                            <div v-if="filteredCredentialTotal > 0" class="flex justify-end px-2 py-3">
                                <el-pagination
                                    v-model:current-page="currentPage"
                                    v-model:page-size="pageSize"
                                    :page-sizes="[10, 20, 50, 100]"
                                    :total="filteredCredentialTotal"
                                    layout="total, sizes, prev, pager, next, jumper"
                                    background
                                    small
                                    @size-change="handlePageSizeChange"
                                    @current-change="handleCurrentPageChange" />
                            </div>
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
                <h4
                    class="mb-2.5 mt-0 pb-1.5 text-sm font-600 text-[var(--color-text-secondary)] border-b border-solid border-0 border-[var(--color-window-titlebar-border)]">
                    {{ t('credential.detail.basicInfo') }}
                </h4>
                <el-descriptions :column="1" border size="small">
                    <el-descriptions-item :label="t('credential.list.title')">
                        <span class="truncate">{{ credentialDetail.title }}</span>
                        <el-button link size="small" @click="copyToClipboard(credentialDetail.title)">
                            <el-icon><CopyDocument /></el-icon>
                        </el-button>
                    </el-descriptions-item>
                    <el-descriptions-item :label="t('credential.list.username')">
                        <span class="truncate">{{ credentialDetail.username || '-' }}</span>
                        <el-button link size="small" @click="copyToClipboard(credentialDetail.username ?? undefined)">
                            <el-icon><CopyDocument /></el-icon>
                        </el-button>
                    </el-descriptions-item>
                    <el-descriptions-item :label="t('credential.list.url')">
                        <span class="truncate">{{ credentialDetail.url || '-' }}</span>
                        <el-button link size="small" @click="copyToClipboard(credentialDetail.url ?? undefined)">
                            <el-icon><CopyDocument /></el-icon>
                        </el-button>
                    </el-descriptions-item>
                    <el-descriptions-item :label="t('credential.list.category')">
                        <span class="truncate">{{ credentialDetail.category_name || '-' }}</span>
                    </el-descriptions-item>
                    <el-descriptions-item :label="t('credential.detail.credentialType')">
                        <span class="truncate">{{
                            credentialDetail.sensitive_data?.credential_type
                                ? getCredentialTemplateLabel(
                                      credentialDetail.sensitive_data.credential_type as CredentialTemplateKey,
                                  )
                                : '-'
                        }}</span>
                    </el-descriptions-item>
                    <el-descriptions-item :label="t('credential.detail.tags')">
                        <span class="truncate">{{ credentialDetail.tags || '-' }}</span>
                        <el-button link size="small" @click="copyToClipboard(credentialDetail.tags ?? undefined)">
                            <el-icon><CopyDocument /></el-icon>
                        </el-button>
                    </el-descriptions-item>
                    <el-descriptions-item :label="t('credential.detail.notes')">
                        <span class="truncate">{{ credentialDetail.notes || '-' }}</span>
                        <el-button link size="small" @click="copyToClipboard(credentialDetail.notes ?? undefined)">
                            <el-icon><CopyDocument /></el-icon>
                        </el-button>
                    </el-descriptions-item>
                    <el-descriptions-item
                        v-if="credentialDetail.sensitive_data?.api_url"
                        :label="t('credential.detail.apiUrl')">
                        <span class="truncate">{{ credentialDetail.sensitive_data?.api_url }}</span>
                        <el-button
                            link
                            size="small"
                            @click="copyToClipboard(credentialDetail.sensitive_data?.api_url ?? undefined)">
                            <el-icon><CopyDocument /></el-icon>
                        </el-button>
                    </el-descriptions-item>
                    <el-descriptions-item
                        v-if="credentialDetail.sensitive_data?.doc_url"
                        :label="t('credential.detail.docUrl')">
                        <span class="truncate">{{ credentialDetail.sensitive_data?.doc_url }}</span>
                        <el-button
                            link
                            size="small"
                            @click="copyToClipboard(credentialDetail.sensitive_data?.doc_url ?? undefined)">
                            <el-icon><CopyDocument /></el-icon>
                        </el-button>
                    </el-descriptions-item>
                </el-descriptions>

                <template v-if="credentialDetail.sensitive_data?.sensitive_sets?.length">
                    <h4
                        class="mb-2.5 mt-4 pb-1.5 text-sm font-600 text-[var(--color-text-secondary)] border-b border-solid border-0 border-[var(--color-window-titlebar-border)]">
                        {{ t('credential.detail.sensitiveSets') }}
                    </h4>
                    <div class="space-y-3">
                        <div
                            v-for="(set, index) in credentialDetail.sensitive_data.sensitive_sets"
                            :key="index"
                            class="rounded-2 border border-solid border-[var(--color-window-titlebar-border)] bg-[var(--color-bg-page)] p-4">
                            <div class="flex items-center justify-between mb-2">
                                <span class="font-600 text-[var(--color-text-primary)]"
                                    >{{ t('credential.detail.information') }} {{ index + 1 }}</span
                                >
                            </div>
                            <el-descriptions :column="1" border size="small">
                                <el-descriptions-item v-if="set.username" :label="t('credential.list.username')">
                                    <span class="truncate">{{ set.username }}</span>
                                    <el-button link size="small" @click="copyToClipboard(set.username)">
                                        <el-icon><CopyDocument /></el-icon>
                                    </el-button>
                                </el-descriptions-item>
                                <el-descriptions-item v-if="set.password" :label="t('credential.detail.password')">
                                    <span v-if="detailAccountVisible[`sensitive-${index}-password`]">{{
                                        set.password
                                    }}</span>
                                    <span v-else class="text-gray-400">••••••••</span>
                                    <el-button
                                        link
                                        size="small"
                                        @click="
                                            detailAccountVisible[`sensitive-${index}-password`] =
                                                !detailAccountVisible[`sensitive-${index}-password`]
                                        ">
                                        <el-icon
                                            ><View v-if="!detailAccountVisible[`sensitive-${index}-password`]" /><Hide
                                                v-else
                                        /></el-icon>
                                    </el-button>
                                    <el-button link size="small" @click="copyToClipboard(set.password)">
                                        <el-icon><CopyDocument /></el-icon>
                                    </el-button>
                                </el-descriptions-item>
                                <el-descriptions-item v-if="set.api_key" :label="t('credential.detail.apiKey')">
                                    <span v-if="detailAccountVisible[`sensitive-${index}-api_key`]">{{
                                        set.api_key
                                    }}</span>
                                    <span v-else class="text-gray-400">••••••••</span>
                                    <el-button
                                        link
                                        size="small"
                                        @click="
                                            detailAccountVisible[`sensitive-${index}-api_key`] =
                                                !detailAccountVisible[`sensitive-${index}-api_key`]
                                        ">
                                        <el-icon
                                            ><View v-if="!detailAccountVisible[`sensitive-${index}-api_key`]" /><Hide
                                                v-else
                                        /></el-icon>
                                    </el-button>
                                    <el-button link size="small" @click="copyToClipboard(set.api_key)">
                                        <el-icon><CopyDocument /></el-icon>
                                    </el-button>
                                </el-descriptions-item>
                                <el-descriptions-item v-if="set.secret_key" :label="t('credential.detail.secretKey')">
                                    <span v-if="detailAccountVisible[`sensitive-${index}-secret_key`]">{{
                                        set.secret_key
                                    }}</span>
                                    <span v-else class="text-gray-400">••••••••</span>
                                    <el-button
                                        link
                                        size="small"
                                        @click="
                                            detailAccountVisible[`sensitive-${index}-secret_key`] =
                                                !detailAccountVisible[`sensitive-${index}-secret_key`]
                                        ">
                                        <el-icon
                                            ><View v-if="!detailAccountVisible[`sensitive-${index}-secret_key`]" /><Hide
                                                v-else
                                        /></el-icon>
                                    </el-button>
                                    <el-button link size="small" @click="copyToClipboard(set.secret_key)">
                                        <el-icon><CopyDocument /></el-icon>
                                    </el-button>
                                </el-descriptions-item>
                                <el-descriptions-item
                                    v-if="set.access_token"
                                    :label="t('credential.detail.accessToken')">
                                    <span v-if="detailAccountVisible[`sensitive-${index}-access_token`]">{{
                                        set.access_token
                                    }}</span>
                                    <span v-else class="text-gray-400">••••••••</span>
                                    <el-button
                                        link
                                        size="small"
                                        @click="
                                            detailAccountVisible[`sensitive-${index}-access_token`] =
                                                !detailAccountVisible[`sensitive-${index}-access_token`]
                                        ">
                                        <el-icon
                                            ><View
                                                v-if="!detailAccountVisible[`sensitive-${index}-access_token`]" /><Hide
                                                v-else
                                        /></el-icon>
                                    </el-button>
                                    <el-button link size="small" @click="copyToClipboard(set.access_token)">
                                        <el-icon><CopyDocument /></el-icon>
                                    </el-button>
                                </el-descriptions-item>
                                <el-descriptions-item
                                    v-if="set.refresh_token"
                                    :label="t('credential.detail.refreshToken')">
                                    <span v-if="detailAccountVisible[`sensitive-${index}-refresh_token`]">{{
                                        set.refresh_token
                                    }}</span>
                                    <span v-else class="text-gray-400">••••••••</span>
                                    <el-button
                                        link
                                        size="small"
                                        @click="
                                            detailAccountVisible[`sensitive-${index}-refresh_token`] =
                                                !detailAccountVisible[`sensitive-${index}-refresh_token`]
                                        ">
                                        <el-icon
                                            ><View
                                                v-if="!detailAccountVisible[`sensitive-${index}-refresh_token`]" /><Hide
                                                v-else
                                        /></el-icon>
                                    </el-button>
                                    <el-button link size="small" @click="copyToClipboard(set.refresh_token)">
                                        <el-icon><CopyDocument /></el-icon>
                                    </el-button>
                                </el-descriptions-item>
                                <el-descriptions-item
                                    v-if="set.expires_at"
                                    :label="t('credential.form.expiresAtLabel')">
                                    <span class="truncate">{{ set.expires_at }}</span>
                                    <el-button link size="small" @click="copyToClipboard(set.expires_at)">
                                        <el-icon><CopyDocument /></el-icon>
                                    </el-button>
                                </el-descriptions-item>
                                <el-descriptions-item v-if="set.notes" :label="t('credential.detail.notes')">
                                    <span class="truncate">{{ set.notes }}</span>
                                    <el-button link size="small" @click="copyToClipboard(set.notes)">
                                        <el-icon><CopyDocument /></el-icon>
                                    </el-button>
                                </el-descriptions-item>
                            </el-descriptions>
                        </div>
                    </div>
                </template>

                <template
                    v-if="
                        !credentialDetail.sensitive_data?.sensitive_sets?.length &&
                        credentialDetail.sensitive_data?.account_sets?.length
                    ">
                    <h4
                        class="mb-2.5 mt-4 pb-1.5 text-sm font-600 text-[var(--color-text-secondary)] border-b border-solid border-0 border-[var(--color-window-titlebar-border)]">
                        {{ t('credential.detail.multipleAccounts') }}
                    </h4>
                    <div class="space-y-3">
                        <div
                            v-for="(account, index) in credentialDetail.sensitive_data.account_sets"
                            :key="index"
                            class="rounded-2 border border-solid border-[var(--color-window-titlebar-border)] bg-[var(--color-bg-page)] p-4">
                            <div class="flex items-center justify-between mb-2">
                                <span class="font-600 text-[var(--color-text-primary)]"
                                    >{{ t('credential.detail.account') }} {{ index + 1 }}</span
                                >
                            </div>
                            <el-descriptions :column="1" border size="small">
                                <el-descriptions-item :label="t('credential.list.username')">
                                    <span class="truncate">{{ account.username || '-' }}</span>
                                    <el-button link size="small" @click="copyToClipboard(account.username)">
                                        <el-icon><CopyDocument /></el-icon>
                                    </el-button>
                                </el-descriptions-item>
                                <el-descriptions-item :label="t('credential.detail.password')">
                                    <div class="flex items-center gap-1">
                                        <span class="flex-1 break-all">{{
                                            detailAccountVisible[`account-${index}-password`]
                                                ? account.password || '-'
                                                : '••••••••'
                                        }}</span>
                                        <el-button
                                            link
                                            size="small"
                                            @click="
                                                detailAccountVisible[`account-${index}-password`] =
                                                    !detailAccountVisible[`account-${index}-password`]
                                            ">
                                            <el-icon
                                                ><View v-if="!detailAccountVisible[`account-${index}-password`]" /><Hide
                                                    v-else
                                            /></el-icon>
                                        </el-button>
                                        <el-button link size="small" @click="copyToClipboard(account.password)">
                                            <el-icon><CopyDocument /></el-icon>
                                        </el-button>
                                    </div>
                                </el-descriptions-item>
                                <el-descriptions-item :label="t('credential.detail.notes')">
                                    <span class="truncate">{{ account.notes || '-' }}</span>
                                    <el-button link size="small" @click="copyToClipboard(account.notes)">
                                        <el-icon><CopyDocument /></el-icon>
                                    </el-button>
                                </el-descriptions-item>
                            </el-descriptions>
                        </div>
                    </div>
                </template>

                <template
                    v-if="
                        !credentialDetail.sensitive_data?.sensitive_sets?.length &&
                        !credentialDetail.sensitive_data?.account_sets?.length
                    ">
                    <h4
                        class="mb-2.5 mt-4 pb-1.5 text-sm font-600 text-[var(--color-text-secondary)] border-b border-solid border-0 border-[var(--color-window-titlebar-border)]">
                        {{ t('credential.detail.sensitiveInfo') }}
                    </h4>
                    <el-descriptions :column="1" border size="small">
                        <el-descriptions-item :label="t('credential.detail.password')">
                            <div class="flex items-center gap-1">
                                <span class="flex-1 break-all">{{
                                    detailVisible.password
                                        ? credentialDetail.sensitive_data?.password || '-'
                                        : '••••••••'
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
                            <div class="flex items-center gap-1">
                                <span class="flex-1 break-all">{{
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
                            <div class="flex items-center gap-1">
                                <span class="flex-1 break-all">{{
                                    detailVisible.secretKey
                                        ? credentialDetail.sensitive_data?.secret_key || '-'
                                        : '••••••••'
                                }}</span>
                                <el-button
                                    link
                                    size="small"
                                    @click="detailVisible.secretKey = !detailVisible.secretKey">
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
                            <div class="flex items-center gap-1">
                                <span class="flex-1 break-all">{{
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
                            <div class="flex items-center gap-1">
                                <span class="flex-1 break-all">{{
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
                </template>

                <!-- Custom fields in detail -->
                <template
                    v-if="
                        credentialDetail.sensitive_data?.custom_fields &&
                        Object.keys(credentialDetail.sensitive_data.custom_fields).length
                    ">
                    <h4
                        class="mb-2.5 mt-4 pb-1.5 text-sm font-600 text-[var(--color-text-secondary)] border-b border-solid border-0 border-[var(--color-window-titlebar-border)]">
                        {{ t('credential.detail.customFields') }}
                    </h4>
                    <el-descriptions :column="1" border size="small">
                        <el-descriptions-item
                            v-for="(val, key) in credentialDetail.sensitive_data.custom_fields"
                            :key="key"
                            :label="String(key)">
                            <div class="flex items-center gap-1">
                                <span class="flex-1 break-all">{{
                                    detailCustomVisible[String(key)] ? val : '••••••••'
                                }}</span>
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
const pageSize = ref(10);
const currentPage = ref(1);

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

const categoryFilteredCredentials = computed(() => {
    if (selectedCategoryId.value === null) return filteredCredentials.value;
    const targetIds = getCategoryChildrenIds(selectedCategoryId.value);
    return filteredCredentials.value.filter((c) => targetIds.includes(c.category_id));
});

const filteredCredentialTotal = computed(() => categoryFilteredCredentials.value.length);

const displayCredentials = computed(() => {
    const start = (currentPage.value - 1) * pageSize.value;
    return categoryFilteredCredentials.value.slice(start, start + pageSize.value);
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

const selectCategory = (catId: number | null) => {
    selectedCategoryId.value = catId;
};

const handlePageSizeChange = (nextSize: number) => {
    pageSize.value = nextSize;
    currentPage.value = 1;
};

const handleCurrentPageChange = (nextPage: number) => {
    currentPage.value = nextPage;
};

watch([selectedCategoryId, searchQuery], () => {
    currentPage.value = 1;
});

watch(filteredCredentialTotal, (total) => {
    const maxPage = Math.max(1, Math.ceil(total / pageSize.value));
    if (currentPage.value > maxPage) {
        currentPage.value = maxPage;
    }
});

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
const detailAccountVisible = reactive<Record<string, boolean>>({});
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
        for (const key of Object.keys(detailAccountVisible)) delete detailAccountVisible[key];
        // Initialize sensitive_sets visibility
        if (detail.sensitive_data?.sensitive_sets) {
            detail.sensitive_data.sensitive_sets.forEach((set, index) => {
                if (set.password) detailAccountVisible[`sensitive-${index}-password`] = false;
                if (set.api_key) detailAccountVisible[`sensitive-${index}-api_key`] = false;
                if (set.secret_key) detailAccountVisible[`sensitive-${index}-secret_key`] = false;
                if (set.access_token) detailAccountVisible[`sensitive-${index}-access_token`] = false;
                if (set.refresh_token) detailAccountVisible[`sensitive-${index}-refresh_token`] = false;
            });
        }
        // Initialize account_sets visibility
        if (detail.sensitive_data?.account_sets) {
            detail.sensitive_data.account_sets.forEach((account, index) => {
                if (account.password) detailAccountVisible[`account-${index}-password`] = false;
            });
        }
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
    flex-direction: column;
    height: 100%;
    min-height: 0;
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
    min-height: 0;
}

/* ── Content ── */

.credential-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
    background-color: var(--color-input-bg);
}

.credential-table-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow-y: auto;
    padding: 12px 16px;
}

.credential-table-wrapper :deep(.el-table) {
    flex: 1;
    min-height: 0;
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

.truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
</style>
