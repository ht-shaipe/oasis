<template>
  <MacWindow
    :title="t('credential.title')"
    :isMinimized="isMinimized"
    @close="handleClose"
    @minimize="emit('minimize')"
    width="900"
    height="600"
  >
    <div class="credential-container">
      <!-- ═══ Setup View ═══ -->
      <div v-if="viewState === 'setup'" class="credential-setup">
        <div class="setup-card">
          <el-icon class="setup-icon" :size="48"><Lock /></el-icon>
          <h2>{{ t('credential.setup.title') }}</h2>
          <el-form
            ref="setupFormRef"
            :model="setupForm"
            :rules="setupRules"
            label-position="top"
            @submit.prevent="handleSetup"
          >
            <el-form-item :label="t('credential.setup.password')" prop="password">
              <el-input
                v-model="setupForm.password"
                type="password"
                show-password
                :placeholder="t('credential.setup.passwordHint')"
              />
            </el-form-item>
            <el-form-item :label="t('credential.setup.confirmPassword')" prop="confirmPassword">
              <el-input
                v-model="setupForm.confirmPassword"
                type="password"
                show-password
                :placeholder="t('credential.setup.confirmPassword')"
              />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" :loading="setupLoading" @click="handleSetup" style="width: 100%">
                {{ t('credential.setup.submit') }}
              </el-button>
            </el-form-item>
          </el-form>
        </div>
      </div>

      <!-- ═══ Unlock View ═══ -->
      <div v-else-if="viewState === 'unlock'" class="credential-unlock">
        <div class="unlock-card">
          <el-icon class="unlock-icon" :size="48"><Unlock /></el-icon>
          <h2>{{ t('credential.unlock.title') }}</h2>
          <el-form
            ref="unlockFormRef"
            :model="unlockForm"
            :rules="unlockRules"
            label-position="top"
            @submit.prevent="handleUnlock"
          >
            <el-form-item :label="t('credential.unlock.password')" prop="password">
              <el-input
                v-model="unlockForm.password"
                type="password"
                show-password
                @keyup.enter="handleUnlock"
              />
            </el-form-item>
            <p v-if="unlockError" class="unlock-error">{{ unlockError }}</p>
            <el-form-item>
              <el-button type="primary" :loading="unlockLoading" @click="handleUnlock" style="width: 100%">
                {{ t('credential.unlock.submit') }}
              </el-button>
            </el-form-item>
          </el-form>
        </div>
      </div>

      <!-- ═══ Main View ═══ -->
      <div v-else class="credential-main">
        <!-- Sidebar -->
        <div class="credential-sidebar">
          <div class="sidebar-section">
            <div class="section-title">{{ t('credential.title') }}</div>
            <div
              :class="['sidebar-item', { active: selectedCategoryId === null }]"
              @click="selectCategory(null)"
            >
              <el-icon><FolderOpened /></el-icon>
              <span>{{ t('credential.category.all') }}</span>
            </div>
            <div
              v-for="cat in categories"
              :key="cat.id"
              :class="['sidebar-item', { active: selectedCategoryId === cat.id }]"
              @click="selectCategory(cat.id)"
            >
              <el-icon><Folder /></el-icon>
              <span>{{ cat.name }}</span>
            </div>
          </div>
          <div class="sidebar-footer">
            <el-button text size="small" @click="showAddCategoryDialog = true">
              <el-icon><Plus /></el-icon>
              {{ t('credential.category.add') }}
            </el-button>
          </div>
        </div>

        <!-- Content area -->
        <div class="credential-content">
          <!-- Toolbar -->
          <div class="credential-toolbar">
            <div class="search-box">
              <el-input
                v-model="searchQuery"
                :placeholder="t('credential.list.search')"
                :prefix-icon="Search"
                clearable
                size="small"
              />
            </div>
            <el-button type="primary" size="small" @click="openCreateDialog">
              <el-icon><Plus /></el-icon>
              {{ t('credential.list.add') }}
            </el-button>
            <el-button text @click="handleLock">
              <el-icon><Lock /></el-icon>
              {{ t('credential.lock') }}
            </el-button>
          </div>

          <!-- Credential table -->
          <div class="credential-table-wrapper">
            <el-empty v-if="filteredCredentials.length === 0 && !tableLoading" :description="t('credential.list.empty')" />
            <el-table
              v-else
              v-loading="tableLoading"
              :data="filteredCredentials"
              style="width: 100%"
              @row-dblclick="handleViewCredential"
            >
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
    <el-dialog
      v-model="showAddCategoryDialog"
      :title="t('credential.category.add')"
      width="400"
      append-to-body
    >
      <el-form @submit.prevent="handleAddCategory">
        <el-form-item :label="t('credential.category.name')">
          <el-input v-model="newCategoryName" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddCategoryDialog = false">{{ t('credential.detail.cancel') }}</el-button>
        <el-button type="primary" @click="handleAddCategory">{{ t('credential.detail.save') }}</el-button>
      </template>
    </el-dialog>

    <!-- ═══ Credential Edit/Create Dialog ═══ -->
    <el-dialog
      v-model="showCredDialog"
      :title="isEditMode ? t('credential.detail.editTitle') : t('credential.detail.createTitle')"
      width="600"
      append-to-body
      destroy-on-close
    >
      <el-form
        ref="credFormRef"
        :model="credForm"
        label-position="top"
      >
        <!-- Basic info -->
        <h4 class="section-heading">{{ t('credential.detail.basicInfo') }}</h4>

        <el-form-item :label="t('credential.list.title')" required>
          <el-input v-model="credForm.title" />
        </el-form-item>

        <el-form-item :label="t('credential.list.category')">
          <el-select v-model="credForm.category_id" :placeholder="t('credential.list.category')" style="width: 100%">
            <el-option
              v-for="cat in categories"
              :key="cat.id"
              :label="cat.name"
              :value="cat.id"
            />
          </el-select>
        </el-form-item>

        <el-form-item :label="t('credential.list.username')">
          <el-input v-model="credForm.username" />
        </el-form-item>

        <el-form-item :label="t('credential.list.url')">
          <el-input v-model="credForm.url" />
        </el-form-item>

        <el-form-item :label="t('credential.detail.tags')">
          <el-input v-model="credForm.tags" :placeholder="t('credential.detail.tags')" />
        </el-form-item>

        <el-form-item :label="t('credential.detail.notes')">
          <el-input v-model="credForm.notes" type="textarea" :rows="2" />
        </el-form-item>

        <!-- Sensitive info -->
        <h4 class="section-heading">{{ t('credential.detail.sensitiveInfo') }}</h4>

        <el-form-item :label="t('credential.detail.password')">
          <div class="sensitive-field">
            <el-input
              v-model="credForm.sensitive.password"
              :type="visibleFields.password ? 'text' : 'password'"
            />
            <el-button link @click="toggleVisible('password')">
              <el-icon><component :is="visibleFields.password ? Hide : View" /></el-icon>
            </el-button>
            <el-button link @click="copyToClipboard(credForm.sensitive.password)">
              <el-icon><CopyDocument /></el-icon>
            </el-button>
          </div>
        </el-form-item>

        <el-form-item :label="t('credential.detail.apiKey')">
          <div class="sensitive-field">
            <el-input
              v-model="credForm.sensitive.api_key"
              :type="visibleFields.apiKey ? 'text' : 'password'"
            />
            <el-button link @click="toggleVisible('apiKey')">
              <el-icon><component :is="visibleFields.apiKey ? Hide : View" /></el-icon>
            </el-button>
            <el-button link @click="copyToClipboard(credForm.sensitive.api_key)">
              <el-icon><CopyDocument /></el-icon>
            </el-button>
          </div>
        </el-form-item>

        <el-form-item :label="t('credential.detail.secretKey')">
          <div class="sensitive-field">
            <el-input
              v-model="credForm.sensitive.secret_key"
              :type="visibleFields.secretKey ? 'text' : 'password'"
            />
            <el-button link @click="toggleVisible('secretKey')">
              <el-icon><component :is="visibleFields.secretKey ? Hide : View" /></el-icon>
            </el-button>
            <el-button link @click="copyToClipboard(credForm.sensitive.secret_key)">
              <el-icon><CopyDocument /></el-icon>
            </el-button>
          </div>
        </el-form-item>

        <el-form-item :label="t('credential.detail.accessToken')">
          <div class="sensitive-field">
            <el-input
              v-model="credForm.sensitive.access_token"
              :type="visibleFields.accessToken ? 'text' : 'password'"
            />
            <el-button link @click="toggleVisible('accessToken')">
              <el-icon><component :is="visibleFields.accessToken ? Hide : View" /></el-icon>
            </el-button>
            <el-button link @click="copyToClipboard(credForm.sensitive.access_token)">
              <el-icon><CopyDocument /></el-icon>
            </el-button>
          </div>
        </el-form-item>

        <el-form-item :label="t('credential.detail.refreshToken')">
          <div class="sensitive-field">
            <el-input
              v-model="credForm.sensitive.refresh_token"
              :type="visibleFields.refreshToken ? 'text' : 'password'"
            />
            <el-button link @click="toggleVisible('refreshToken')">
              <el-icon><component :is="visibleFields.refreshToken ? Hide : View" /></el-icon>
            </el-button>
            <el-button link @click="copyToClipboard(credForm.sensitive.refresh_token)">
              <el-icon><CopyDocument /></el-icon>
            </el-button>
          </div>
        </el-form-item>

        <!-- Custom fields -->
        <h4 class="section-heading">{{ t('credential.detail.customFields') }}</h4>

        <div v-for="(field, index) in customFields" :key="index" class="custom-field-row">
          <el-input v-model="field.key" :placeholder="'Key'" class="custom-key" />
          <div class="sensitive-field custom-value">
            <el-input
              v-model="field.value"
              :type="field.visible ? 'text' : 'password'"
              :placeholder="'Value'"
            />
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
      destroy-on-close
    >
      <template v-if="credentialDetail">
        <h4 class="section-heading">{{ t('credential.detail.basicInfo') }}</h4>
        <el-descriptions :column="1" border size="small">
          <el-descriptions-item :label="t('credential.list.title')">{{ credentialDetail.title }}</el-descriptions-item>
          <el-descriptions-item :label="t('credential.list.username')">{{ credentialDetail.username || '-' }}</el-descriptions-item>
          <el-descriptions-item :label="t('credential.list.url')">{{ credentialDetail.url || '-' }}</el-descriptions-item>
          <el-descriptions-item :label="t('credential.list.category')">{{ credentialDetail.category_name || '-' }}</el-descriptions-item>
          <el-descriptions-item :label="t('credential.detail.tags')">{{ credentialDetail.tags || '-' }}</el-descriptions-item>
          <el-descriptions-item :label="t('credential.detail.notes')">{{ credentialDetail.notes || '-' }}</el-descriptions-item>
        </el-descriptions>

        <h4 class="section-heading" style="margin-top: 16px">{{ t('credential.detail.sensitiveInfo') }}</h4>
        <el-descriptions :column="1" border size="small">
          <el-descriptions-item :label="t('credential.detail.password')">
            <div class="detail-sensitive-value">
              <span>{{ detailVisible.password ? (credentialDetail.sensitive_data?.password || '-') : '••••••••' }}</span>
              <el-button link size="small" @click="detailVisible.password = !detailVisible.password">
                <el-icon><component :is="detailVisible.password ? Hide : View" /></el-icon>
              </el-button>
              <el-button link size="small" @click="copyToClipboard(credentialDetail.sensitive_data?.password)">
                <el-icon><CopyDocument /></el-icon>
              </el-button>
            </div>
          </el-descriptions-item>
          <el-descriptions-item :label="t('credential.detail.apiKey')">
            <div class="detail-sensitive-value">
              <span>{{ detailVisible.apiKey ? (credentialDetail.sensitive_data?.api_key || '-') : '••••••••' }}</span>
              <el-button link size="small" @click="detailVisible.apiKey = !detailVisible.apiKey">
                <el-icon><component :is="detailVisible.apiKey ? Hide : View" /></el-icon>
              </el-button>
              <el-button link size="small" @click="copyToClipboard(credentialDetail.sensitive_data?.api_key)">
                <el-icon><CopyDocument /></el-icon>
              </el-button>
            </div>
          </el-descriptions-item>
          <el-descriptions-item :label="t('credential.detail.secretKey')">
            <div class="detail-sensitive-value">
              <span>{{ detailVisible.secretKey ? (credentialDetail.sensitive_data?.secret_key || '-') : '••••••••' }}</span>
              <el-button link size="small" @click="detailVisible.secretKey = !detailVisible.secretKey">
                <el-icon><component :is="detailVisible.secretKey ? Hide : View" /></el-icon>
              </el-button>
              <el-button link size="small" @click="copyToClipboard(credentialDetail.sensitive_data?.secret_key)">
                <el-icon><CopyDocument /></el-icon>
              </el-button>
            </div>
          </el-descriptions-item>
          <el-descriptions-item :label="t('credential.detail.accessToken')">
            <div class="detail-sensitive-value">
              <span>{{ detailVisible.accessToken ? (credentialDetail.sensitive_data?.access_token || '-') : '••••••••' }}</span>
              <el-button link size="small" @click="detailVisible.accessToken = !detailVisible.accessToken">
                <el-icon><component :is="detailVisible.accessToken ? Hide : View" /></el-icon>
              </el-button>
              <el-button link size="small" @click="copyToClipboard(credentialDetail.sensitive_data?.access_token)">
                <el-icon><CopyDocument /></el-icon>
              </el-button>
            </div>
          </el-descriptions-item>
          <el-descriptions-item :label="t('credential.detail.refreshToken')">
            <div class="detail-sensitive-value">
              <span>{{ detailVisible.refreshToken ? (credentialDetail.sensitive_data?.refresh_token || '-') : '••••••••' }}</span>
              <el-button link size="small" @click="detailVisible.refreshToken = !detailVisible.refreshToken">
                <el-icon><component :is="detailVisible.refreshToken ? Hide : View" /></el-icon>
              </el-button>
              <el-button link size="small" @click="copyToClipboard(credentialDetail.sensitive_data?.refresh_token)">
                <el-icon><CopyDocument /></el-icon>
              </el-button>
            </div>
          </el-descriptions-item>
        </el-descriptions>

        <!-- Custom fields in detail -->
        <template v-if="credentialDetail.sensitive_data?.custom_fields && Object.keys(credentialDetail.sensitive_data.custom_fields).length">
          <h4 class="section-heading" style="margin-top: 16px">{{ t('credential.detail.customFields') }}</h4>
          <el-descriptions :column="1" border size="small">
            <el-descriptions-item v-for="(val, key) in credentialDetail.sensitive_data.custom_fields" :key="key" :label="String(key)">
              <div class="detail-sensitive-value">
                <span>{{ detailCustomVisible[String(key)] ? val : '••••••••' }}</span>
                <el-button link size="small" @click="detailCustomVisible[String(key)] = !detailCustomVisible[String(key)]">
                  <el-icon><component :is="detailCustomVisible[String(key)] ? Hide : View" /></el-icon>
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
        <el-button type="primary" @click="openEditDialog(credentialDetail!); showDetailDialog = false">
          <el-icon><Edit /></el-icon>
          {{ t('credential.list.edit') }}
        </el-button>
      </template>
    </el-dialog>
  </MacWindow>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, reactive } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import {
  Lock, Unlock, Folder, FolderOpened, Plus, Search,
  View, Hide, Key, Edit, Delete, CopyDocument,
} from '@element-plus/icons-vue'
import MacWindow from '@/components/common/MacWindow.vue'
import {
  useCredential,
  type Category,
  type CredentialView,
  type CredentialDetail,
  type SensitiveData,
} from '@/composables/useCredential'

const { t } = useI18n()
const {
  isMasterKeySet, setupMasterKey, unlock, lock,
  listCategories, createCategory,
  listCredentials, getCredential, createCredential, updateCredential, deleteCredential,
  isLocked,
} = useCredential()

// ── Props / Emits ──

const props = defineProps<{ isMinimized: boolean }>()
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'minimize'): void
}>()

// ── View state machine ──

type ViewState = 'setup' | 'unlock' | 'main'
const viewState = ref<ViewState>('unlock')

// ── Setup form ──

const setupFormRef = ref<FormInstance>()
const setupForm = reactive({ password: '', confirmPassword: '' })
const setupLoading = ref(false)

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
          callback(new Error(t('credential.setup.mismatch')))
        } else {
          callback()
        }
      },
      trigger: 'blur',
    },
  ],
}))

const handleSetup = async () => {
  const form = setupFormRef.value
  if (!form) return
  await form.validate()
  setupLoading.value = true
  try {
    await setupMasterKey(setupForm.password)
    viewState.value = 'main'
    await loadMainData()
  } catch (err: unknown) {
    ElMessage.error(err instanceof Error ? err.message : String(err))
  } finally {
    setupLoading.value = false
  }
}

// ── Unlock form ──

const unlockFormRef = ref<FormInstance>()
const unlockForm = reactive({ password: '' })
const unlockLoading = ref(false)
const unlockError = ref('')
const unlockAttempts = ref(0)

const unlockRules = computed<FormRules>(() => ({
  password: [{ required: true, message: t('credential.unlock.wrongPassword'), trigger: 'blur' }],
}))

const handleUnlock = async () => {
  const form = unlockFormRef.value
  if (!form) return
  await form.validate()
  unlockError.value = ''

  // Exponential back-off after 5 failed attempts
  if (unlockAttempts.value >= 5) {
    const delay = Math.min(2000 * Math.pow(2, unlockAttempts.value - 5), 30000)
    unlockError.value = t('credential.unlock.tooManyAttempts')
    await new Promise((r) => setTimeout(r, delay))
  }

  unlockLoading.value = true
  try {
    await unlock(unlockForm.password)
    unlockAttempts.value = 0
    viewState.value = 'main'
    await loadMainData()
  } catch {
    unlockAttempts.value++
    unlockError.value = t('credential.unlock.wrongPassword')
  } finally {
    unlockLoading.value = false
  }
}

// ── Lock ──

const handleLock = () => {
  lock()
  viewState.value = 'unlock'
  unlockForm.password = ''
  resetAutoLockTimer()
}

const handleClose = () => {
  lock()
  emit('close')
}

// ── Auto-lock (30 min) ──

let autoLockTimer: ReturnType<typeof setTimeout> | null = null

const resetAutoLockTimer = () => {
  if (autoLockTimer) clearTimeout(autoLockTimer)
  if (viewState.value === 'main') {
    autoLockTimer = setTimeout(() => {
      lock()
      viewState.value = 'unlock'
      ElMessage.warning(t('credential.autoLockWarning'))
    }, 30 * 60 * 1000)
  }
}

const onUserActivity = () => {
  if (viewState.value === 'main') resetAutoLockTimer()
}

// ── Main view data ──

const categories = ref<Category[]>([])
const credentials = ref<CredentialView[]>([])
const selectedCategoryId = ref<number | null>(null)
const searchQuery = ref('')
const tableLoading = ref(false)

const filteredCredentials = computed(() => {
  if (!searchQuery.value) return credentials.value
  const q = searchQuery.value.toLowerCase()
  return credentials.value.filter(
    (c) =>
      c.title.toLowerCase().includes(q) ||
      (c.username && c.username.toLowerCase().includes(q)) ||
      (c.url && c.url.toLowerCase().includes(q)),
  )
})

const loadMainData = async () => {
  try {
    const [cats, creds] = await Promise.all([listCategories(), listCredentials(selectedCategoryId.value ?? undefined)])
    categories.value = cats
    credentials.value = creds
  } catch (err: unknown) {
    ElMessage.error(err instanceof Error ? err.message : String(err))
  }
}

const selectCategory = async (catId: number | null) => {
  selectedCategoryId.value = catId
  tableLoading.value = true
  try {
    credentials.value = await listCredentials(catId ?? undefined)
  } catch (err: unknown) {
    ElMessage.error(err instanceof Error ? err.message : String(err))
  } finally {
    tableLoading.value = false
  }
}

// ── Add category ──

const showAddCategoryDialog = ref(false)
const newCategoryName = ref('')

const handleAddCategory = async () => {
  if (!newCategoryName.value.trim()) return
  try {
    const cat = await createCategory(newCategoryName.value.trim())
    categories.value.push(cat)
    newCategoryName.value = ''
    showAddCategoryDialog.value = false
  } catch (err: unknown) {
    ElMessage.error(err instanceof Error ? err.message : String(err))
  }
}

// ── Credential edit/create dialog ──

const showCredDialog = ref(false)
const isEditMode = ref(false)
const editingCredId = ref<number | null>(null)
const credSaving = ref(false)

const credForm = reactive({
  title: '',
  category_id: null as number | null,
  username: '',
  url: '',
  tags: '',
  notes: '',
  sensitive: {
    password: '',
    api_key: '',
    secret_key: '',
    access_token: '',
    refresh_token: '',
  } as Record<string, string>,
})

const customFields = ref<Array<{ key: string; value: string; visible: boolean }>>([])

const visibleFields = reactive({
  password: false,
  apiKey: false,
  secretKey: false,
  accessToken: false,
  refreshToken: false,
})

const toggleVisible = (field: keyof typeof visibleFields) => {
  visibleFields[field] = !visibleFields[field]
}

const resetCredForm = () => {
  credForm.title = ''
  credForm.category_id = selectedCategoryId.value
  credForm.username = ''
  credForm.url = ''
  credForm.tags = ''
  credForm.notes = ''
  credForm.sensitive = { password: '', api_key: '', secret_key: '', access_token: '', refresh_token: '' }
  customFields.value = []
  visibleFields.password = false
  visibleFields.apiKey = false
  visibleFields.secretKey = false
  visibleFields.accessToken = false
  visibleFields.refreshToken = false
}

const openCreateDialog = () => {
  isEditMode.value = false
  editingCredId.value = null
  resetCredForm()
  showCredDialog.value = true
}

const openEditDialog = async (row: CredentialView | CredentialDetail) => {
  isEditMode.value = true
  editingCredId.value = row.id
  resetCredForm()

  // Populate basic fields
  credForm.title = row.title
  credForm.category_id = row.category_id
  credForm.username = row.username || ''
  credForm.url = row.url || ''
  credForm.tags = row.tags || ''
  credForm.notes = row.notes || ''

  // If we already have sensitive_data (from detail view), use it
  const detail = row as CredentialDetail
  if (detail.sensitive_data) {
    credForm.sensitive.password = detail.sensitive_data.password || ''
    credForm.sensitive.api_key = detail.sensitive_data.api_key || ''
    credForm.sensitive.secret_key = detail.sensitive_data.secret_key || ''
    credForm.sensitive.access_token = detail.sensitive_data.access_token || ''
    credForm.sensitive.refresh_token = detail.sensitive_data.refresh_token || ''
    if (detail.sensitive_data.custom_fields) {
      customFields.value = Object.entries(detail.sensitive_data.custom_fields).map(([key, value]) => ({
        key,
        value,
        visible: false,
      }))
    }
  } else {
    // Need to fetch detail
    try {
      const d = await getCredential(row.id)
      credForm.sensitive.password = d.sensitive_data.password || ''
      credForm.sensitive.api_key = d.sensitive_data.api_key || ''
      credForm.sensitive.secret_key = d.sensitive_data.secret_key || ''
      credForm.sensitive.access_token = d.sensitive_data.access_token || ''
      credForm.sensitive.refresh_token = d.sensitive_data.refresh_token || ''
      if (d.sensitive_data.custom_fields) {
        customFields.value = Object.entries(d.sensitive_data.custom_fields).map(([key, value]) => ({
          key,
          value,
          visible: false,
        }))
      }
    } catch {
      // If backend fails, just leave sensitive fields empty
    }
  }

  showCredDialog.value = true
}

const handleSaveCredential = async () => {
  if (!credForm.title.trim()) {
    ElMessage.warning(t('credential.list.title'))
    return
  }

  // Build sensitive_data_json
  const sensitiveData: SensitiveData = {
    password: credForm.sensitive.password || undefined,
    api_key: credForm.sensitive.api_key || undefined,
    secret_key: credForm.sensitive.secret_key || undefined,
    access_token: credForm.sensitive.access_token || undefined,
    refresh_token: credForm.sensitive.refresh_token || undefined,
  }
  const customObj: Record<string, string> = {}
  for (const f of customFields.value) {
    if (f.key.trim()) customObj[f.key.trim()] = f.value
  }
  if (Object.keys(customObj).length > 0) {
    sensitiveData.custom_fields = customObj
  }

  credSaving.value = true
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
      })
    } else {
      await createCredential({
        category_id: credForm.category_id ?? 0,
        title: credForm.title,
        username: credForm.username || undefined,
        url: credForm.url || undefined,
        sensitive_data_json: JSON.stringify(sensitiveData),
        tags: credForm.tags || undefined,
        notes: credForm.notes || undefined,
      })
    }
    showCredDialog.value = false
    await loadMainData()
  } catch (err: unknown) {
    ElMessage.error(err instanceof Error ? err.message : String(err))
  } finally {
    credSaving.value = false
  }
}

// ── Delete credential ──

const handleDeleteCredential = async (row: CredentialView) => {
  try {
    await ElMessageBox.confirm(t('credential.list.deleteConfirm'), {
      type: 'warning',
    })
    await deleteCredential(row.id)
    await loadMainData()
  } catch {
    // User cancelled or deletion failed silently
  }
}

// ── View credential detail ──

const showDetailDialog = ref(false)
const credentialDetail = ref<CredentialDetail | null>(null)
const detailVisible = reactive({
  password: false,
  apiKey: false,
  secretKey: false,
  accessToken: false,
  refreshToken: false,
})
const detailCustomVisible = reactive<Record<string, boolean>>({})

const handleViewCredential = async (row: CredentialView) => {
  try {
    const detail = await getCredential(row.id)
    credentialDetail.value = detail
    detailVisible.password = false
    detailVisible.apiKey = false
    detailVisible.secretKey = false
    detailVisible.accessToken = false
    detailVisible.refreshToken = false
    // Reset custom field visibility
    for (const k of Object.keys(detailCustomVisible)) delete detailCustomVisible[k]
    if (detail.sensitive_data?.custom_fields) {
      for (const k of Object.keys(detail.sensitive_data.custom_fields)) {
        detailCustomVisible[k] = false
      }
    }
    showDetailDialog.value = true
  } catch (err: unknown) {
    ElMessage.error(err instanceof Error ? err.message : String(err))
  }
}

// ── Clipboard ──

const copyToClipboard = async (text: string | undefined) => {
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success(t('credential.detail.copied'))
  } catch {
    ElMessage.error(t('credential.detail.copy'))
  }
}

// ── Date formatting ──

const formatDate = (dateStr: string | null | undefined): string => {
  if (!dateStr) return '-'
  const d = new Date(dateStr)
  return `${d.toLocaleDateString()} ${d.toLocaleTimeString()}`
}

// ── Lifecycle ──

onMounted(async () => {
  try {
    const keySet = await isMasterKeySet()
    viewState.value = keySet ? 'unlock' : 'setup'
  } catch {
    // If backend not ready, default to setup
    viewState.value = 'setup'
  }

  // Activity listeners for auto-lock
  document.addEventListener('mousemove', onUserActivity)
  document.addEventListener('keydown', onUserActivity)
  document.addEventListener('click', onUserActivity)
})

onUnmounted(() => {
  if (autoLockTimer) clearTimeout(autoLockTimer)
  document.removeEventListener('mousemove', onUserActivity)
  document.removeEventListener('keydown', onUserActivity)
  document.removeEventListener('click', onUserActivity)
})
</script>

<style scoped>
.credential-container {
  display: flex;
  height: 100%;
  background-color: var(--color-sidebar-bg);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
}

/* ── Setup & Unlock centered cards ── */

.credential-setup,
.credential-unlock {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  background-color: var(--color-input-bg);
}

.setup-card,
.unlock-card {
  width: 380px;
  padding: 32px;
  background: var(--color-sidebar-bg);
  border-radius: 12px;
  text-align: center;
}

.setup-icon,
.unlock-icon {
  color: var(--color-text-secondary);
  margin-bottom: 12px;
}

.setup-card h2,
.unlock-card h2 {
  margin: 0 0 20px;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-primary);
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

/* ── Sidebar ── */

.credential-sidebar {
  width: 200px;
  background-color: var(--color-sidebar-bg);
  border-right: 1px solid var(--color-window-titlebar-border);
  display: flex;
  flex-direction: column;
  padding: 10px 0;
  overflow-y: auto;
}

.sidebar-section {
  flex: 1;
}

.section-title {
  padding: 0 16px;
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-tertiary);
  margin-bottom: 5px;
  text-transform: uppercase;
}

.sidebar-item {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  cursor: pointer;
  border-radius: 5px;
  margin: 0 4px;
  transition: background-color 0.15s;
}

.sidebar-item .el-icon {
  margin-right: 8px;
  font-size: 16px;
  color: var(--color-text-secondary);
}

.sidebar-item:hover {
  background-color: rgba(0, 0, 0, 0.05);
}

.sidebar-item.active {
  background-color: rgba(0, 123, 255, 0.1);
  color: #007bff;
}

.sidebar-item.active .el-icon {
  color: #007bff;
}

.sidebar-footer {
  padding: 8px 12px;
  border-top: 1px solid var(--color-window-titlebar-border);
}

/* ── Content ── */

.credential-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: var(--color-input-bg);
}

.credential-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background-color: var(--color-sidebar-bg);
  border-bottom: 1px solid var(--color-window-titlebar-border);
  height: 50px;
}

.search-box {
  flex: 1;
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
</style>
