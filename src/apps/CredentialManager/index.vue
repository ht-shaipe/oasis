<template>
    <MacWindow
        :title="t('credential.title')"
        :isMinimized="isMinimized"
        @close="handleClose"
        @minimize="emit('minimize')"
        width="900"
        height="600">
        <div class="credential-container">
            <!-- ═══ Setup View ═══ -->
            <CredentialAuthCard
                v-if="viewState === 'setup'"
                :title="t('credential.setup.title')"
                :loading="setupLoading"
                loading-text="正在解密并同步数据...">
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
                            style="width: 100%">
                            {{ t('credential.setup.submit') }}
                        </el-button>
                    </el-form-item>
                </el-form>
            </CredentialAuthCard>

            <!-- ═══ Unlock View ═══ -->
            <CredentialAuthCard
                v-else-if="viewState === 'unlock'"
                :title="t('credential.unlock.title')"
                :loading="unlockLoading"
                loading-text="正在解密并同步数据...">
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
                            :disabled="unlockLoading" />
                    </el-form-item>
                    <p v-if="unlockError" class="unlock-error">{{ unlockError }}</p>
                    <el-form-item>
                        <el-button
                            type="primary"
                            native-type="submit"
                            :loading="unlockLoading"
                            :disabled="unlockLoading"
                            style="width: 100%">
                            {{ t('credential.unlock.submit') }}
                        </el-button>
                    </el-form-item>
                </el-form>
            </CredentialAuthCard>

            <!-- ═══ Main View ═══ -->
            <div v-else class="credential-main">
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
                            <el-table-column :label="t('credential.list.actions')" width="150" fixed="right">
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
            </div>
        </div>

        <!-- ═══ Add Category Dialog ═══ -->
        <el-dialog v-model="showAddCategoryDialog" :title="t('credential.category.add')" width="400" append-to-body>
            <el-form @submit.prevent="handleAddCategory" label-position="top">
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

        <!-- ═══ Credential Edit/Create Dialog ═══ -->
        <el-dialog
            v-model="showCredDialog"
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

                    <el-form-item :label="t('credential.list.category')">
                        <el-select
                            v-model="credForm.category_id"
                            :placeholder="t('credential.list.category')"
                            style="width: 100%">
                            <el-option
                                v-for="cat in flattenedCategories"
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

                <div v-if="credForm.credential_type === 'account'" class="grid gap-4 md:grid-cols-2">
                    <el-form-item :label="t('credential.form.passwordLabel')">
                        <div class="sensitive-field">
                            <el-input
                                v-model="credForm.sensitive.password"
                                :placeholder="t('credential.form.passwordHint')"
                                :type="visibleFields.password ? 'text' : 'password'" />
                            <el-button link @click="toggleVisible('password')">
                                <el-icon><component :is="visibleFields.password ? Hide : View" /></el-icon>
                            </el-button>
                            <el-button link @click="copyToClipboard(credForm.sensitive.password)">
                                <el-icon><CopyDocument /></el-icon>
                            </el-button>
                        </div>
                    </el-form-item>
                </div>

                <div v-else-if="credForm.credential_type === 'api_key'" class="grid gap-4 md:grid-cols-2">
                    <el-form-item :label="t('credential.form.keyLabel')">
                        <div class="sensitive-field">
                            <el-input
                                v-model="credForm.sensitive.api_key"
                                :placeholder="t('credential.form.keyHint')"
                                :type="visibleFields.apiKey ? 'text' : 'password'" />
                            <el-button link @click="toggleVisible('apiKey')">
                                <el-icon><component :is="visibleFields.apiKey ? Hide : View" /></el-icon>
                            </el-button>
                            <el-button link @click="copyToClipboard(credForm.sensitive.api_key)">
                                <el-icon><CopyDocument /></el-icon>
                            </el-button>
                        </div>
                    </el-form-item>
                </div>

                <div v-else-if="credForm.credential_type === 'key_secret'" class="grid gap-4 md:grid-cols-2">
                    <el-form-item :label="t('credential.form.keyLabel')">
                        <div class="sensitive-field">
                            <el-input
                                v-model="credForm.sensitive.api_key"
                                :placeholder="t('credential.form.keyHint')"
                                :type="visibleFields.apiKey ? 'text' : 'password'" />
                            <el-button link @click="toggleVisible('apiKey')">
                                <el-icon><component :is="visibleFields.apiKey ? Hide : View" /></el-icon>
                            </el-button>
                            <el-button link @click="copyToClipboard(credForm.sensitive.api_key)">
                                <el-icon><CopyDocument /></el-icon>
                            </el-button>
                        </div>
                    </el-form-item>

                    <el-form-item :label="t('credential.form.secretLabel')">
                        <div class="sensitive-field">
                            <el-input
                                v-model="credForm.sensitive.secret_key"
                                :placeholder="t('credential.form.secretHint')"
                                :type="visibleFields.secretKey ? 'text' : 'password'" />
                            <el-button link @click="toggleVisible('secretKey')">
                                <el-icon><component :is="visibleFields.secretKey ? Hide : View" /></el-icon>
                            </el-button>
                            <el-button link @click="copyToClipboard(credForm.sensitive.secret_key)">
                                <el-icon><CopyDocument /></el-icon>
                            </el-button>
                        </div>
                    </el-form-item>
                </div>

                <div v-else-if="credForm.credential_type === 'expiring_key'" class="grid gap-4 md:grid-cols-2">
                    <el-form-item :label="t('credential.form.keyLabel')">
                        <div class="sensitive-field">
                            <el-input
                                v-model="credForm.sensitive.api_key"
                                :placeholder="t('credential.form.keyHint')"
                                :type="visibleFields.apiKey ? 'text' : 'password'" />
                            <el-button link @click="toggleVisible('apiKey')">
                                <el-icon><component :is="visibleFields.apiKey ? Hide : View" /></el-icon>
                            </el-button>
                            <el-button link @click="copyToClipboard(credForm.sensitive.api_key)">
                                <el-icon><CopyDocument /></el-icon>
                            </el-button>
                        </div>
                    </el-form-item>

                    <el-form-item :label="t('credential.form.expiresAtLabel')">
                        <el-date-picker
                            v-model="credForm.sensitive.expires_at"
                            class="w-full"
                            type="datetime"
                            :placeholder="t('credential.form.expiresAtHint')"
                            value-format="YYYY-MM-DD HH:mm:ss" />
                    </el-form-item>
                </div>

                <div v-else class="grid gap-4 md:grid-cols-2">
                    <el-form-item
                        v-if="shouldShowField(credForm.credential_type, 'password')"
                        :label="t('credential.form.passwordLabel')">
                        <div class="sensitive-field">
                            <el-input
                                v-model="credForm.sensitive.password"
                                :placeholder="t('credential.form.passwordHint')"
                                :type="visibleFields.password ? 'text' : 'password'" />
                            <el-button link @click="toggleVisible('password')">
                                <el-icon><component :is="visibleFields.password ? Hide : View" /></el-icon>
                            </el-button>
                            <el-button link @click="copyToClipboard(credForm.sensitive.password)">
                                <el-icon><CopyDocument /></el-icon>
                            </el-button>
                        </div>
                    </el-form-item>

                    <el-form-item
                        v-if="shouldShowField(credForm.credential_type, 'api_key')"
                        :label="t('credential.form.keyLabel')">
                        <div class="sensitive-field">
                            <el-input
                                v-model="credForm.sensitive.api_key"
                                :placeholder="t('credential.form.keyHint')"
                                :type="visibleFields.apiKey ? 'text' : 'password'" />
                            <el-button link @click="toggleVisible('apiKey')">
                                <el-icon><component :is="visibleFields.apiKey ? Hide : View" /></el-icon>
                            </el-button>
                            <el-button link @click="copyToClipboard(credForm.sensitive.api_key)">
                                <el-icon><CopyDocument /></el-icon>
                            </el-button>
                        </div>
                    </el-form-item>

                    <el-form-item
                        v-if="shouldShowField(credForm.credential_type, 'secret_key')"
                        :label="t('credential.form.secretLabel')">
                        <div class="sensitive-field">
                            <el-input
                                v-model="credForm.sensitive.secret_key"
                                :placeholder="t('credential.form.secretHint')"
                                :type="visibleFields.secretKey ? 'text' : 'password'" />
                            <el-button link @click="toggleVisible('secretKey')">
                                <el-icon><component :is="visibleFields.secretKey ? Hide : View" /></el-icon>
                            </el-button>
                            <el-button link @click="copyToClipboard(credForm.sensitive.secret_key)">
                                <el-icon><CopyDocument /></el-icon>
                            </el-button>
                        </div>
                    </el-form-item>

                    <el-form-item
                        v-if="shouldShowField(credForm.credential_type, 'expires_at')"
                        :label="t('credential.form.expiresAtLabel')">
                        <el-date-picker
                            v-model="credForm.sensitive.expires_at"
                            class="w-full"
                            type="datetime"
                            :placeholder="t('credential.form.expiresAtHint')"
                            value-format="YYYY-MM-DD HH:mm:ss" />
                    </el-form-item>

                    <el-form-item
                        v-if="shouldShowField(credForm.credential_type, 'access_token')"
                        :label="t('credential.detail.accessToken')">
                        <div class="sensitive-field">
                            <el-input
                                v-model="credForm.sensitive.access_token"
                                :placeholder="t('credential.form.accessTokenHint')"
                                :type="visibleFields.accessToken ? 'text' : 'password'" />
                            <el-button link @click="toggleVisible('accessToken')">
                                <el-icon><component :is="visibleFields.accessToken ? Hide : View" /></el-icon>
                            </el-button>
                            <el-button link @click="copyToClipboard(credForm.sensitive.access_token)">
                                <el-icon><CopyDocument /></el-icon>
                            </el-button>
                        </div>
                    </el-form-item>

                    <el-form-item
                        v-if="shouldShowField(credForm.credential_type, 'refresh_token')"
                        :label="t('credential.detail.refreshToken')">
                        <div class="sensitive-field">
                            <el-input
                                v-model="credForm.sensitive.refresh_token"
                                :placeholder="t('credential.form.refreshTokenHint')"
                                :type="visibleFields.refreshToken ? 'text' : 'password'" />
                            <el-button link @click="toggleVisible('refreshToken')">
                                <el-icon><component :is="visibleFields.refreshToken ? Hide : View" /></el-icon>
                            </el-button>
                            <el-button link @click="copyToClipboard(credForm.sensitive.refresh_token)">
                                <el-icon><CopyDocument /></el-icon>
                            </el-button>
                        </div>
                    </el-form-item>
                </div>

                <!-- Custom fields -->
                <h4 class="section-heading">{{ t('credential.detail.customFields') }}</h4>

                <div v-for="(field, index) in customFields" :key="index" class="custom-field-row">
                    <el-input v-model="field.key" :placeholder="'Key'" class="custom-key" />
                    <div class="sensitive-field custom-value">
                        <el-input
                            v-model="field.value"
                            :type="field.visible ? 'text' : 'password'"
                            :placeholder="'Value'" />
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
                <el-button @click="showCredDialog = false">{{ t('credential.detail.cancel') }}</el-button>
                <el-button type="primary" :loading="credSaving" @click="handleSaveCredential">
                    {{ t('credential.detail.save') }}
                </el-button>
            </template>
        </el-dialog>

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
import { ref, computed, onMounted, onUnmounted, reactive } from 'vue';
import { useI18n } from 'vue-i18n';
import { ElMessage, ElMessageBox } from 'element-plus';
import type { FormInstance, FormRules } from 'element-plus';
import { View, Hide, Key, Edit, Delete, CopyDocument } from '@element-plus/icons-vue';
import MacWindow from '@/components/common/MacWindow.vue';
import CredentialAuthCard from './AuthCard.vue';
import CredentialSidebar from './Sidebar.vue';
import CredentialToolbar from './Toolbar.vue';
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
    type Category,
    type CredentialView,
    type CredentialDetail,
    type SensitiveData,
} from '@/composables/useCredential';

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
    createCredential,
    updateCredential,
    deleteCredential,
} = useCredential();

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
    const startTime = Date.now();
    try {
        await setupMasterKey(setupForm.password);
        await loadMainData();

        // Ensure minimum loading time for visual feedback
        const elapsed = Date.now() - startTime;
        if (elapsed < 800) {
            await new Promise((r) => setTimeout(r, 800 - elapsed));
        }

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
const unlockError = ref('');
const unlockAttempts = ref(0);

const unlockRules = computed<FormRules>(() => ({
    password: [{ required: true, message: t('credential.unlock.wrongPassword'), trigger: 'blur' }],
}));

const handleUnlock = async () => {
    const form = unlockFormRef.value;
    if (!form) return;
    await form.validate();
    unlockError.value = '';

    unlockLoading.value = true;

    // Show a warning after 5 failed attempts
    if (unlockAttempts.value >= 5) {
        unlockError.value = t('credential.unlock.tooManyAttempts');
    }

    const startTime = Date.now();
    try {
        await unlock(unlockForm.password);
        unlockAttempts.value = 0;
        await loadMainData();

        // Ensure minimum loading time for visual feedback
        const elapsed = Date.now() - startTime;
        if (elapsed < 800) {
            await new Promise((r) => setTimeout(r, 800 - elapsed));
        }

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
const isEditMode = ref(false);
const editingCredId = ref<number | null>(null);
const credSaving = ref(false);

const credForm = reactive<CredentialFormModel>(defaultCredentialForm(null));

const customFields = ref<Array<{ key: string; value: string; visible: boolean }>>([]);

const visibleFields = reactive({
    password: false,
    apiKey: false,
    secretKey: false,
    accessToken: false,
    refreshToken: false,
});

const toggleVisible = (field: keyof typeof visibleFields) => {
    visibleFields[field] = !visibleFields[field];
};

// Clear fields that no longer apply after changing the credential template.
const handleCredentialTypeChange = () => {
    normalizeSensitiveFields(credForm);
};

const resetCredForm = () => {
    Object.assign(credForm, defaultCredentialForm(selectedCategoryId.value));
    customFields.value = [];
    visibleFields.password = false;
    visibleFields.apiKey = false;
    visibleFields.secretKey = false;
    visibleFields.accessToken = false;
    visibleFields.refreshToken = false;
};

const openCreateDialog = () => {
    isEditMode.value = false;
    editingCredId.value = null;
    resetCredForm();
    showCredDialog.value = true;
};

const openEditDialog = async (row: CredentialView | CredentialDetail) => {
    isEditMode.value = true;
    editingCredId.value = row.id;
    resetCredForm();

    // Populate basic fields
    credForm.title = row.title;
    credForm.category_id = row.category_id;
    credForm.username = row.username || '';
    credForm.url = row.url || '';
    credForm.tags = row.tags || '';
    credForm.notes = row.notes || '';

    // If we already have sensitive_data (from detail view), use it
    const detail = row as CredentialDetail;
    if (detail.sensitive_data) {
        credForm.credential_type =
            (detail.sensitive_data.credential_type as CredentialTemplateKey) ??
            inferCredentialType(detail.sensitive_data);
        credForm.sensitive.password = detail.sensitive_data.password || '';
        credForm.sensitive.api_key = detail.sensitive_data.api_key || '';
        credForm.sensitive.secret_key = detail.sensitive_data.secret_key || '';
        credForm.sensitive.expires_at = detail.sensitive_data.expires_at || '';
        credForm.sensitive.access_token = detail.sensitive_data.access_token || '';
        credForm.sensitive.refresh_token = detail.sensitive_data.refresh_token || '';
        if (detail.sensitive_data.custom_fields) {
            customFields.value = Object.entries(detail.sensitive_data.custom_fields).map(([key, value]) => ({
                key,
                value,
                visible: false,
            }));
        }
    } else {
        // Need to fetch detail
        try {
            const d = await getCredential(row.id);
            credForm.credential_type =
                (d.sensitive_data.credential_type as CredentialTemplateKey) ?? inferCredentialType(d.sensitive_data);
            credForm.sensitive.password = d.sensitive_data.password || '';
            credForm.sensitive.api_key = d.sensitive_data.api_key || '';
            credForm.sensitive.secret_key = d.sensitive_data.secret_key || '';
            credForm.sensitive.expires_at = d.sensitive_data.expires_at || '';
            credForm.sensitive.access_token = d.sensitive_data.access_token || '';
            credForm.sensitive.refresh_token = d.sensitive_data.refresh_token || '';
            if (d.sensitive_data.custom_fields) {
                customFields.value = Object.entries(d.sensitive_data.custom_fields).map(([key, value]) => ({
                    key,
                    value,
                    visible: false,
                }));
            }
        } catch {
            // If backend fails, just leave sensitive fields empty
        }
    }

    showCredDialog.value = true;
};

const handleSaveCredential = async () => {
    if (!credForm.title.trim()) {
        ElMessage.warning(t('credential.list.title'));
        return;
    }

    // Build sensitive_data_json
    normalizeSensitiveFields(credForm);
    const sensitiveData: SensitiveData = buildSensitiveData(credForm);
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
                sensitive_data_json: JSON.stringify(sensitiveData),
                tags: credForm.tags || undefined,
                notes: credForm.notes || undefined,
            });
        } else {
            await createCredential({
                category_id: credForm.category_id ?? 0,
                title: credForm.title,
                username: credForm.username || undefined,
                url: credForm.url || undefined,
                sensitive_data_json: JSON.stringify(sensitiveData),
                tags: credForm.tags || undefined,
                notes: credForm.notes || undefined,
            });
        }
        showCredDialog.value = false;
        await loadMainData();
    } catch (err: unknown) {
        ElMessage.error(err instanceof Error ? err.message : String(err));
    } finally {
        credSaving.value = false;
    }
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
        ElMessage.error(err instanceof Error ? err.message : String(err));
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

/* ── Sensitive field row ── */

.sensitive-field {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
}

.sensitive-field .el-input {
    flex: 1;
}

/* ── Custom field row ── */

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
