<template>
    <MacWindow 
        :title="t('generator.windowTitle')" 
        :isMinimized="isMinimized" 
        @close="closeApp" 
        @minimize="toggleMinimize"
        width="800"
        height="600"
    >
        <el-form :model="form" label-width="120px" style="padding: 20px;">
            <el-form-item :label="t('generator.descriptionLabel')">
                <el-input v-model="form.description" type="textarea" :rows="4"
                    :placeholder="t('generator.description')"
                    @focus.stop
                    @click.stop
                    ref="descriptionInputRef" />
            </el-form-item>
            <el-form-item :label="t('generator.modelSelectLabel')">
                <div class="model-selection-wrapper" style="width: 300px;">
                    <el-select v-model="form.model" :placeholder="t('generator.modelSelect')">
                        <el-option 
                            v-for="(cost, model) in modelCreditCosts" 
                            :key="model" 
                            :label="`${model} (${cost}${t('signIn.creditsUnit')})`" 
                            :value="model" 
                        >
                            <div class="model-option">
                                <span>{{ model }}</span>
                                <span class="credit-badge">{{ cost }}{{ t('signIn.creditsUnit') }}</span>
                            </div>
                        </el-option>
                    </el-select>
                </div>
            </el-form-item>

            <el-form-item :label="t('generator.uiFrameworkLabel')">
                <el-select v-model="form.uiLibrary" :placeholder="t('generator.uiFrameworkSelect')">
                    <el-option label="原生样式" value="native" />
                    <el-option label="Element UI" value="element-ui" />
                    <el-option label="Element Plus" value="element-plus" />
                    <el-option label="Layui" value="layui" />
                    <el-option label="Bootstrap" value="bootstrap" />
                    <el-option label="Ant Design" value="ant-design" />
                    <el-option label="Tailwind CSS" value="tailwind" />
                </el-select>
            </el-form-item>

            <el-form-item :label="t('generator.jsFrameworkLabel')">
                <el-select v-model="form.jsFramework" :placeholder="t('generator.jsFrameworkSelect')">
                    <el-option label="原生JavaScript" value="vanilla" />
                    <el-option label="jQuery" value="jquery" />
                    <el-option label="Vue 2" value="vue2" />
                    <el-option label="Vue 3" value="vue3" />
                    <el-option label="React" value="react" />
                    <el-option label="Alpine.js" value="alpine" />
                </el-select>
            </el-form-item>

            <el-form-item :label="t('generator.uiStyleLabel')">
                <el-select v-model="form.uiStyle" :placeholder="t('generator.styleSelect')">
                    <el-option label="现代简约" value="modern" />
                    <el-option label="暗黑风格" value="dark" />
                    <el-option label="明亮风格" value="light" />
                    <el-option label="极简主义" value="minimalist" />
                    <el-option label="Bento Box" value="bento" />
                    <el-option label="玻璃态" value="glassmorphism" />
                    <el-option label="新拟物" value="neumorphism" />
                    <el-option label="复古" value="retro" />
                </el-select>
            </el-form-item>

            <el-form-item :label="t('generator.cdnLabel')">
                <el-select v-model="form.cdnProvider" :placeholder="t('generator.cdnSelect')">
                    <el-option label="自动选择" value="auto" />
                    <el-option label="jsDelivr" value="jsdelivr" />
                    <el-option label="百度CDN" value="baidu" />
                    <el-option label="Unpkg" value="unpkg" />
                    <el-option label="Cloudflare" value="cloudflare" />
                    <el-option label="七牛云" value="qiniu" />
                    <el-option label="BootCDN" value="bootcdn" />
                </el-select>
            </el-form-item>

            <el-form-item :label="t('generator.deviceLabel')">
                <el-select v-model="form.deviceType" :placeholder="t('generator.deviceSelect')">
                    <el-option label="响应式设计" value="responsive" />
                    <el-option label="桌面网页版" value="desktop" />
                    <el-option label="移动端" value="mobile" />
                    <el-option label="平板设备" value="tablet" />
                    <el-option label="多设备兼容" value="multi-device" />
                </el-select>
            </el-form-item>

            <el-form-item :label="t('generator.codeStyleLabel')">
                <el-select v-model="form.style" :placeholder="t('generator.codeStyleSelect')">
                    <el-option label="简洁风格" value="simple" />
                    <el-option label="详细注释" value="detailed" />
                    <el-option label="函数式编程" value="functional" />
                    <el-option label="面向对象" value="oop" />
                </el-select>
            </el-form-item>

            <el-form-item>
                <el-button type="primary" @click="generateCode" :loading="isGenerating">
                    {{ isGenerating ? t('generator.generating') : t('generator.generate') }}
                </el-button>
            </el-form-item>
        </el-form>
    </MacWindow>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import { useI18n } from 'vue-i18n';
import MacWindow from '@/components/common/MacWindow.vue';
import { generateCodeStream, getModelCreditCosts } from '@/utils/apiService';

const { t } = useI18n();

// 定义属性
const props = defineProps({
    isMinimized: {
        type: Boolean,
        default: false
    },
    browserFingerprint: {
        type: String,
        default: ''
    }
});

// 事件发射
const emit = defineEmits(['close', 'minimize', 'updateGeneratedCode', 'updateSessionInfo', 'openApp']);

// 表单数据
const form = ref({
    description: '',
    model: 'gpt-3.5-turbo',
    style: 'simple',
    uiLibrary: 'native',
    jsFramework: 'vanilla',
    uiStyle: 'modern',
    cdnProvider: 'auto',
    deviceType: 'responsive'
});

// 状态变量
const isGenerating = ref(false);
const currentGeneratedCode = ref(''); // 用于累积代码
const projectId = ref(''); // 项目ID
const versionId = ref(''); // 版本ID
const modelCreditCosts = ref<Record<string, number>>({}); // 用户积分信息

onMounted(async () => {
    try {
        const costsResponse = await getModelCreditCosts();
        modelCreditCosts.value = (costsResponse as any).data;

        // 设置默认选中最后一个模型
        if (Object.keys(modelCreditCosts.value).length > 0) {
            const models = Object.keys(modelCreditCosts.value);
            form.value.model = models[models.length - 1];
        }
    } catch (error: unknown) {
        const errorMessage = error instanceof Error ? error.message : t('app.unknownError');
        ElMessage.error(`${t('generator.modelCostFailed')}: ${errorMessage}`);
    }
});

// 关闭应用
const closeApp = () => {
    emit('close');
};

// 切换最小化状态
const toggleMinimize = () => {
    emit('minimize');
};

// 生成代码方法
const generateCode = async () => {
    if (!form.value.description) {
        ElMessage.warning(t('generator.pleaseInputDescription'));
        return;
    }

    isGenerating.value = true;
    currentGeneratedCode.value = `// ${t('generator.codeGenerating')}\n// ${t('generator.pleaseWait')}`;
    projectId.value = ''; 

    // 立刻更新编辑器显示加载状态
    emit('updateGeneratedCode', currentGeneratedCode.value);
    emit('updateSessionInfo', form.value.description, projectId.value, false);

    const params = {
        description: form.value.description,
        model: form.value.model,
        style: form.value.style,
        uiLibrary: form.value.uiLibrary,
        jsFramework: form.value.jsFramework,
        uiStyle: form.value.uiStyle,
        cdnProvider: form.value.cdnProvider,
        deviceType: form.value.deviceType,
        browserFingerprint: props.browserFingerprint || localStorage.getItem('browserFingerprint') || ''
    };

    // 定义SSE事件处理回调
    const handleData = (data: any) => {
        if (data.error) {
            ElMessage.error(data.message);
            isGenerating.value = false;
            return;
        }

        if (data.code) {
            if (currentGeneratedCode.value.includes(`// ${t('generator.codeGenerating')}`)) {
                currentGeneratedCode.value = '';
            }
            currentGeneratedCode.value += data.code;
            emit('updateGeneratedCode', currentGeneratedCode.value);

            if (data.projectId && !projectId.value) {
                projectId.value = data.projectId;
                emit('updateSessionInfo', form.value.description, data.projectId, false);
            }
        }
    };

    const handleComplete = (data: any) => {
        isGenerating.value = false;
        if (data && data.projectId) {
            projectId.value = data.projectId;
            versionId.value = data.versionId;
        }

        console.log('生成完成，最终项目ID:', projectId.value);
        console.log('生成完成，最终版本ID:', versionId.value);
        ElMessage.success(t('generator.generateComplete'));

        emit('updateGeneratedCode', currentGeneratedCode.value);

        const isCodeGenerating = currentGeneratedCode.value.includes(`// ${t('generator.codeGenerating')}`);
        emit('updateSessionInfo', form.value.description, projectId.value, versionId.value, !isCodeGenerating);

        emit('openApp', 'safari');
    };

    const handleError = (errorMessage: string) => {
        console.error('SSE Error Callback:', errorMessage);
        isGenerating.value = false;
        ElMessage.error(`${t('generator.generateError')}: ${errorMessage}`);
        currentGeneratedCode.value += `\n\n// ${t('generator.generateError')}: ${errorMessage}`;
        emit('updateGeneratedCode', currentGeneratedCode.value);
        emit('updateSessionInfo', form.value.description, projectId.value, false);
    };

    // 调用apiService中的方法
    await generateCodeStream(params, handleData, handleComplete, handleError);
};
</script>

<style scoped>
.model-option {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
}

.credit-badge {
    background-color: #f0f9eb;
    color: #67c23a;
    border-radius: 10px;
    padding: 1px 4px;
    font-size: 13px;
    border: 1px solid #e1f3d8;
    line-height: 13px;
    margin-left: 10px;
}
</style> 