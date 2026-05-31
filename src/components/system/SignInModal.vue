<template>
    <el-dialog
        v-model="dialogVisible"
        :title="t('signIn.title')"
        width="400px"
        custom-class="sign-in-dialog"
        :show-close="true"
        :close-on-click-modal="false"
        :close-on-press-escape="true"
        @close="closeDialog"
    >
        <div class="sign-in-container">
            <div class="sign-in-header">
                <div class="consecutive-days">
                    <span class="days-count">{{ signInData.consecutiveDays }}</span>
                    <span class="days-text">{{ t('signIn.consecutiveDays') }}</span>
                </div>
                <div class="next-reward">
                    <div class="reward-text">{{ t('signIn.nextReward') }}</div>
                    <div class="reward-amount">{{ signInData.nextReward }} {{ t('signIn.creditsUnit') }}</div>
                </div>
            </div>

            <div class="calendar-container">
                <div class="calendar-header">
                    <span>{{ currentMonth }}月 {{ currentYear }}</span>
                </div>
                <div class="weekdays">
                    <div class="weekday" v-for="day in weekdays" :key="day">{{ day }}</div>
                </div>
                <div class="calendar-days">
                    <div
                        v-for="(day, idx) in calendarDays"
                        :key="idx"
                        class="calendar-day"
                        :class="{
                            'empty': !day.date,
                            'signed': day.signed,
                            'today': day.isToday,
                            'future': day.isFuture
                        }"
                    >
                        <span v-if="day.date">{{ day.date }}</span>
                        <el-icon v-if="day.signed" class="signed-icon"><Check /></el-icon>
                    </div>
                </div>
            </div>

            <div class="sign-in-button-container">
                <el-button
                    type="primary"
                    class="sign-in-button"
                    :disabled="signInData.hasSigned"
                    @click="handleSignIn"
                    :loading="loading"
                >
                    {{ signInData.hasSigned ? t('signIn.signedToday') : t('signIn.signInNow') }}
                </el-button>
            </div>

            <div class="credit-rules">
                <div class="rule-title">{{ t('signIn.signInRules') }}</div>
                <div class="rule-item">
                    <div class="rule-days">{{ t('signIn.consecutive1to2') }}</div>
                    <div class="rule-credits">{{ t('signIn.credits100') }}</div>
                </div>
                <div class="rule-item">
                    <div class="rule-days">{{ t('signIn.consecutive3to5') }}</div>
                    <div class="rule-credits">{{ t('signIn.credits150') }}</div>
                </div>
                <div class="rule-item">
                    <div class="rule-days">{{ t('signIn.consecutive6plus') }}</div>
                    <div class="rule-credits">{{ t('signIn.credits200') }}</div>
                </div>
            </div>
        </div>
    </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watchEffect } from 'vue';
import { Check } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import { useI18n } from 'vue-i18n';
import { getSignInStatus, signIn, type SignInStatus } from '../../utils/apiService';

const { t } = useI18n();

const props = defineProps({
    visible: Boolean
});

const emit = defineEmits(['update:visible']);

const dialogVisible = computed({
    get: () => props.visible,
    set: (val) => emit('update:visible', val)
});

const closeDialog = () => {
    emit('update:visible', false);
};

// 签到数据
const signInData = ref<SignInStatus>({
    hasSigned: false,
    consecutiveDays: 0,
    signedDates: [],
    nextReward: 100
});

// 当前日期信息
const currentDate = new Date();
const currentYear = currentDate.getFullYear();
const currentMonth = currentDate.getMonth() + 1;
const currentDay = currentDate.getDate();

// 加载状态
const loading = ref(false);

// 星期名称
const weekdays = t('calendar.weekdays') as unknown as string[];

// 计算当月的日历数据
const calendarDays = computed(() => {
    const firstDayOfMonth = new Date(currentYear, currentMonth - 1, 1);
    const lastDayOfMonth = new Date(currentYear, currentMonth, 0);
    const daysInMonth = lastDayOfMonth.getDate();
    const startingDayOfWeek = firstDayOfMonth.getDay(); // 0-6 代表周日到周六

    const days = [];

    // 添加月初的空白天数
    for (let i = 0; i < startingDayOfWeek; i++) {
        days.push({ date: null as number | null });
    }

    // 添加月份中的天数
    for (let day = 1; day <= daysInMonth; day++) {
        const isToday = day === currentDay;
        const isFuture = day > currentDay;
        
        // 检查此日期是否有签到记录
        const signed = signInData.value.signedDates.some(d => d.date === day);
        
        days.push({
            date: day,
            signed,
            isToday,
            isFuture
        });
    }

    return days;
});

// 获取签到状态
const fetchSignInStatus = async () => {
    try {
        loading.value = true;
        const token = localStorage.getItem('auth_token');
        
        if (!token) {
            ElMessage.warning(t('login.pleaseLogin'));
            closeDialog();
            return;
        }
        
        const response = await getSignInStatus();
        
        if (response.success && response.data) {
            signInData.value = response.data.data || response.data as any;
        }
    } catch (error) {
        console.error('获取签到状态失败:', error);
        ElMessage.error(t('signIn.fetchStatusFailed'));
    } finally {
        loading.value = false;
    }
};

// 执行签到
const handleSignIn = async () => {
    try {
        loading.value = true;
        const token = localStorage.getItem('auth_token');
        
        if (!token) {
            ElMessage.warning(t('login.pleaseLogin'));
            closeDialog();
            return;
        }
        
        const response = await signIn();
        
        if (response.success && response.data) {
            const result = response.data;
            // 刷新签到状态
            await fetchSignInStatus();
            ElMessage.success(`${t('signIn.signInSuccess')} ${result.creditsAdded} ${t('signIn.creditsUnit')}`);
        }
    } catch (error: any) {
        console.error('签到失败:', error);
        if (error?.response?.data?.message) {
            ElMessage.error(error.response.data.message);
        } else {
            ElMessage.error(t('signIn.signInFailedRetry'));
        }
    } finally {
        loading.value = false;
    }
};

// 监听对话框打开，获取签到状态
watchEffect(() => {
    if (dialogVisible.value) {
        fetchSignInStatus();
    }
});
</script>

<style scoped>
.sign-in-dialog :deep(.el-dialog__header) {
    text-align: center;
    padding: 15px 0;
    border-bottom: 1px solid #f0f0f0;
}

.sign-in-container {
    padding: 15px;
}

.sign-in-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
}

.consecutive-days {
    display: flex;
    flex-direction: column;
    align-items: center;
}

.days-count {
    font-size: 32px;
    font-weight: bold;
    color: #409EFF;
}

.days-text {
    font-size: 14px;
    color: var(--color-text-secondary);
}

.next-reward {
    text-align: right;
}

.reward-text {
    font-size: 14px;
    color: var(--color-text-secondary);
}

.reward-amount {
    font-size: 20px;
    font-weight: bold;
    color: #FF9500;
}

.calendar-container {
    background-color: var(--color-input-bg);
    border-radius: 8px;
    padding: 15px;
    margin-bottom: 20px;
}

.calendar-header {
    text-align: center;
    margin-bottom: 10px;
    font-size: 16px;
    font-weight: bold;
}

.weekdays {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    text-align: center;
    margin-bottom: 10px;
}

.weekday {
    font-size: 14px;
    color: var(--color-text-secondary);
    padding: 5px 0;
}

.calendar-days {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 5px;
}

.calendar-day {
    height: 36px;
    display: flex;
    justify-content: center;
    align-items: center;
    position: relative;
    font-size: 14px;
    border-radius: 4px;
}

.calendar-day.empty {
    background: none;
}

.calendar-day.signed {
    background-color: #409EFF;
    color: white;
}

.calendar-day.today {
    font-weight: bold;
}

.calendar-day.future {
    color: var(--color-text-tertiary);
}

.signed-icon {
    position: absolute;
    right: 2px;
    bottom: 2px;
    font-size: 10px;
}

.sign-in-button-container {
    text-align: center;
    margin: 20px 0;
}

.sign-in-button {
    width: 200px;
    height: 44px;
    font-size: 16px;
}

.credit-rules {
    background-color: var(--color-input-bg);
    border-radius: 8px;
    padding: 15px;
}

.rule-title {
    font-size: 16px;
    font-weight: bold;
    margin-bottom: 10px;
    color: var(--color-text-primary);
}

.rule-item {
    display: flex;
    justify-content: space-between;
    margin-bottom: 8px;
    font-size: 14px;
}

.rule-days {
    color: var(--color-text-secondary);
}

.rule-credits {
    color: #FF9500;
    font-weight: bold;
}
</style> 