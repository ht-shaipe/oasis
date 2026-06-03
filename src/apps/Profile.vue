<template>
    <MacWindow
        ref="macWindowRef"
        :title="t('profile.title')"
        :isMinimized="isMinimized"
        width="480"
        height="600"
        @close="closeWindow"
        @minimize="minimizeWindow"
    >
        <div class="window-content">
            <div class="loading-container" v-if="loading">
                <div class="mac-spinner"></div>
                <p>{{ t('profile.loadingUserInfo') }}</p>
            </div>
            <div class="profile-container" v-else>
                <div class="profile-header">
                    <div class="avatar">
                        <img :src="avatarUrl" :alt="t('profile.userAvatar')">
                    </div>
                    <div class="user-info">
                        <h2>{{ userInfo.username || t('profile.unsetUsername') }}</h2>
                        <div class="member-badge" :class="memberLevelClass">
                            {{ memberLevelText }}
                        </div>
                    </div>
                    <button class="sign-in-btn" @click="openSignInModal">
                        <span class="sign-in-icon">📅</span>
                        {{ t('profile.signInNow') }}
                    </button>
                </div>
                
                
                <div class="info-section">
                    <h3>{{ t('profile.basicInfo') }}</h3>
                    <div class="info-row">
                        <span class="label">{{ t('profile.emailColon') }}</span>
                        <span class="value">{{ userInfo.email || t('profile.unbindEmail') }}</span>
                    </div>
                    <div class="info-row">
                        <span class="label">{{ t('profile.registerTimeColon') }}</span>
                        <span class="value">{{ formatDate(userInfo.registrationDate) }}</span>
                    </div>
                    <div class="info-row">
                        <span class="label">{{ t('profile.lastLoginColon') }}</span>
                        <span class="value">{{ formatDate(userInfo.lastLoginDate) }}</span>
                    </div>
                </div>
                
                <div class="credits-section">
                    <h3>{{ t('profile.creditsInfo') }}</h3>
                    <div class="credits-container">
                        <div class="credit-card">
                            <div class="credit-icon">🎁</div>
                            <div class="credit-amount">{{ userInfo.freeCredits || 0 }}</div>
                            <div class="credit-label">{{ t('profile.freeCredits') }}</div>
                        </div>
                        <div class="credit-card">
                            <div class="credit-icon">💎</div>
                            <div class="credit-amount">{{ userInfo.paidCredits || 0 }}</div>
                            <div class="credit-label">{{ t('profile.paidCredits') }}</div>
                        </div>
                    </div>
                </div>

                <div class="actions-section">
                    <button class="mac-btn primary" v-if="!userInfo.email" @click="showBindEmailDialog">
                        {{ t('profile.bindEmail') }}
                    </button>
                    <button class="mac-btn" @click="refreshUserInfo">
                        {{ t('profile.refreshInfo') }}
                    </button>
                </div>

                <div class="error-message" v-if="error">
                    {{ error }}
                </div>
            </div>
        </div>
    </MacWindow>

    <!-- 签到弹窗 -->
    <SignInModal v-model:visible="showSignInModal"/>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { getCurrentUser } from '@/utils/apiService';
import { ElMessage } from 'element-plus';
import { useI18n } from 'vue-i18n';
import MacWindow from '@/components/common/MacWindow.vue';
import SignInModal from '@/components/system/SignInModal.vue';

const { t } = useI18n();

// 定义用户信息类型
interface UserInfo {
    username?: string;
    email?: string;
    registrationDate?: string;
    lastLoginDate?: string;
    freeCredits?: number;
    paidCredits?: number;
    memberLevel?: number;
    avatarUrl?: string;
}

// 组件属性
const props = defineProps({
    isMinimized: {
        type: Boolean,
        default: false
    }
});

// 事件
const emit = defineEmits(['close', 'minimize']);

// 用户信息状态
const userInfo = ref<UserInfo>({});
const loading = ref(true);
const error = ref('');

// 签到弹窗显示状态
const showSignInModal = ref(false);

// 计算头像URL
const avatarUrl = computed(() => {
    return userInfo.value.avatarUrl || '/assets/profile.jpg';
});

// 计算会员等级样式
const memberLevelClass = computed(() => {
    const level = userInfo.value.memberLevel || 0;
    if (level === 0) return 'free';
    if (level === 1) return 'basic';
    if (level === 2) return 'premium';
    return 'vip';
});

// 计算会员等级文本
const memberLevelText = computed(() => {
    const level = userInfo.value.memberLevel || 0;
    if (level === 0) return t('profile.freeUser');
    if (level === 1) return t('profile.basicMember');
    if (level === 2) return t('profile.premiumMember');
    return t('profile.proMember');
});

// 获取用户信息
const fetchUserInfo = async () => {
    loading.value = true;
    error.value = '';

    try {
        const response = await getCurrentUser();
        if ((response as any).success) {
            userInfo.value = (response as any).user;
        } else {
            error.value = (response as any).message || t('profile.fetchUserFailed');
        }
    } catch (err: unknown) {
        console.error('获取用户信息出错:', err);
        const errorMessage = err instanceof Error ? err.message : t('app.unknownError');
        error.value = `${t('profile.networkErrorRetry')}: ${errorMessage}`;
    } finally {
        loading.value = false;
    }
};

// 日期格式化
const formatDate = (dateString: string | number | undefined) => {
    if (!dateString) return '未知';

    try {
        const timestamp = parseInt(String(dateString));
        const date = isNaN(timestamp) ? new Date(dateString) : new Date(timestamp * 1000);

        return date.toLocaleString('zh-CN', {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit'
        });
    } catch (_e) {
        return String(dateString);
    }
};

// 绑定邮箱对话框（暂时只显示提示）
const showBindEmailDialog = () => {
    ElMessage.warning(t('profile.bindEmailDev'));
};

// 打开签到弹窗
const openSignInModal = () => {
    showSignInModal.value = true;
};

// 刷新用户信息
const refreshUserInfo = () => {
    fetchUserInfo();
};

// 关闭窗口
const closeWindow = () => {
    emit('close');
};

// 最小化窗口
const minimizeWindow = () => {
    emit('minimize');
};

// MacWindow 组件引用
const macWindowRef = ref<InstanceType<typeof MacWindow> | null>(null);

// 暴露 bringToFront 方法
defineExpose({
    bringToFront: () => macWindowRef.value?.bringToFront()
});

// 组件加载完成后获取用户信息
onMounted(() => {
    fetchUserInfo();
});
</script>

<style scoped>
/* 窗口内容样式 */
.window-content {
    padding: 20px;
    overflow-y: auto;
    height: 100%;
}

/* 加载状态 */
.loading-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
}

.mac-spinner {
    width: 30px;
    height: 30px;
    border: 3px solid rgba(0, 122, 255, 0.2);
    border-radius: 50%;
    border-top-color: #007aff;
    animation: spin 1s linear infinite;
    margin-bottom: 15px;
}

@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}

/* 个人信息样式 */
.profile-container {
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.profile-header {
    display: flex;
    align-items: center;
    padding-bottom: 20px;
    border-bottom: 1px solid #eee;
}

.avatar {
    width: 80px;
    height: 80px;
    border-radius: 50%;
    overflow: hidden;
    border: 2px solid #007aff;
    margin-right: 20px;
}

.avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.user-info {
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: 1;
}

.user-info h2 {
    margin: 0;
    font-size: var(--app-font-20);
    color: var(--color-text-primary);
}

.member-badge {
    padding: 4px 10px;
    border-radius: 20px;
    font-size: var(--app-font-12);
    font-weight: 500;
    display: inline-block;
}

.member-badge.free {
    background-color: var(--color-window-titlebar);
    color: var(--color-text-secondary);
    width: 100px;
}

.member-badge.basic {
    background-color: #e3f2fd;
    color: #1976d2;
}

.member-badge.premium {
    background-color: #fff8e1;
    color: #f57c00;
}

.member-badge.vip {
    background-color: #fce4ec;
    color: #c2185b;
}

/* 信息区域样式 */
.info-section, .credits-section, .daily-check-section {
    background-color: var(--color-sidebar-bg);
    border-radius: 8px;
    padding: 15px;
}

.info-section h3, .credits-section h3, .daily-check-section h3 {
    margin-top: 0;
    margin-bottom: 15px;
    font-size: var(--app-font-16);
    color: var(--color-text-primary);
    border-bottom: 1px solid #eee;
    padding-bottom: 8px;
}

.sign-in-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 8px 12px;
    background-color: #ff9500;
    color: white;
    border: none;
    border-radius: 6px;
    font-size: var(--app-font-14);
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
}

.sign-in-btn:hover {
    background-color: #e67e00;
}

.sign-in-icon {
    font-size: var(--app-font-16);
}

.sign-in-tip {
    font-size: var(--app-font-12);
    color: var(--color-text-secondary);
    margin-top: 10px;
}

.info-row {
    display: flex;
    margin-bottom: 10px;
}

.info-row .label {
    width: 80px;
    color: var(--color-text-secondary);
}

.info-row .value {
    flex: 1;
    color: var(--color-text-primary);
}

/* 积分卡片样式 */
.credits-container {
    display: flex;
    gap: 15px;
}

.credit-card {
    flex: 1;
    background: white;
    border-radius: 8px;
    padding: 15px;
    text-align: center;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.credit-icon {
    font-size: var(--app-font-24);
    margin-bottom: 10px;
}

.credit-amount {
    font-size: var(--app-font-22);
    font-weight: bold;
    color: #007aff;
    margin-bottom: 5px;
}

.credit-label {
    font-size: var(--app-font-12);
    color: var(--color-text-secondary);
}

/* 按钮样式 */
.actions-section {
    display: flex;
    gap: 10px;
    justify-content: flex-end;
}

.mac-btn {
    padding: 8px 16px;
    border-radius: 6px;
    border: none;
    font-size: var(--app-font-14);
    cursor: pointer;
    background-color: var(--color-window-titlebar);
    color: var(--color-text-primary);
    transition: all 0.2s;
}

.mac-btn:hover {
    background-color: #e5e5e5;
}

.mac-btn.primary {
    background-color: #007aff;
    color: white;
}

.mac-btn.primary:hover {
    background-color: #0066cc;
}

/* 错误信息 */
.error-message {
    color: #ff3b30;
    font-size: var(--app-font-14);
    text-align: center;
    margin-top: 10px;
}
</style> 