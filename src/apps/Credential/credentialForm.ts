import type { SensitiveData } from '@/composables/useCredential';

export type CredentialTemplateKey = 'account' | 'api_key' | 'key_secret' | 'expiring_key' | 'custom' | string;

export interface CredentialFormModel {
    credential_type: CredentialTemplateKey;
    title: string;
    category_id: number | null;
    username: string;
    url: string;
    tags: string;
    notes: string;
    sensitive: {
        password: string;
        api_key: string;
        secret_key: string;
        expires_at: string;
        access_token: string;
        refresh_token: string;
    };
}

export interface CredentialTemplateOption {
    value: CredentialTemplateKey;
    label: string;
    description: string;
    fields?: string[];
}

export const defaultCredentialTemplateOptions: Array<CredentialTemplateOption> = [
    {
        value: 'account',
        label: '账号凭证',
        description: '用户名 + 密码，适合网站、系统登录。',
        fields: ['password'],
    },
    {
        value: 'api_key',
        label: '密钥类',
        description: '只有一个 Key，适合 API Key、访问令牌等。',
        fields: ['api_key'],
    },
    {
        value: 'key_secret',
        label: 'Key + Secret',
        description: '同时保存 Key 和 Secret 的双字段凭证。',
        fields: ['api_key', 'secret_key'],
    },
    {
        value: 'expiring_key',
        label: '到期密钥',
        description: 'Key + 到期时间，适合会过期的证书或授权。',
        fields: ['api_key', 'expires_at'],
    },
    {
        value: 'custom',
        label: '自定义',
        description: '保留全部字段，自由组合。',
        fields: ['password', 'api_key', 'secret_key', 'expires_at', 'access_token', 'refresh_token'],
    },
];

// 从localStorage加载自定义模板
export const loadCredentialTemplates = (): Array<CredentialTemplateOption> => {
    try {
        const saved = localStorage.getItem('credential_templates');
        if (saved) {
            const customTemplates = JSON.parse(saved);
            return [...defaultCredentialTemplateOptions, ...customTemplates];
        }
    } catch (error) {
        console.warn('Failed to load custom templates:', error);
    }
    return [...defaultCredentialTemplateOptions];
};

// 保存自定义模板到localStorage
export const saveCredentialTemplates = (customTemplates: Array<CredentialTemplateOption>): void => {
    try {
        localStorage.setItem('credential_templates', JSON.stringify(customTemplates));
    } catch (error) {
        console.error('Failed to save custom templates:', error);
    }
};

// 导出的模板选项（从localStorage加载）
export const credentialTemplateOptions = loadCredentialTemplates();

export const defaultCredentialForm = (categoryId: number | null): CredentialFormModel => ({
    credential_type: 'account',
    title: '',
    category_id: categoryId,
    username: '',
    url: '',
    tags: '',
    notes: '',
    sensitive: {
        password: '',
        api_key: '',
        secret_key: '',
        expires_at: '',
        access_token: '',
        refresh_token: '',
    },
});

export const inferCredentialType = (sensitiveData?: Partial<SensitiveData> | null): CredentialTemplateKey => {
    if (!sensitiveData) return 'account';
    if (sensitiveData.secret_key && sensitiveData.api_key) return 'key_secret';
    if (sensitiveData.expires_at && sensitiveData.api_key) return 'expiring_key';
    if (sensitiveData.api_key && !sensitiveData.password) return 'api_key';
    if (sensitiveData.password) return 'account';
    return 'custom';
};

export const buildSensitiveData = (form: CredentialFormModel): SensitiveData => {
    const data: SensitiveData = {
        credential_type: form.credential_type,
        password: form.sensitive.password || undefined,
        api_key: form.sensitive.api_key || undefined,
        secret_key: form.sensitive.secret_key || undefined,
        expires_at: form.sensitive.expires_at || undefined,
        access_token: form.sensitive.access_token || undefined,
        refresh_token: form.sensitive.refresh_token || undefined,
    };

    return data;
};

export const normalizeSensitiveFields = (form: CredentialFormModel): void => {
    if (form.credential_type === 'custom') return;

    if (form.credential_type !== 'account') {
        form.sensitive.password = '';
    }
    if (
        form.credential_type !== 'api_key' &&
        form.credential_type !== 'key_secret' &&
        form.credential_type !== 'expiring_key'
    ) {
        form.sensitive.api_key = '';
    }
    if (form.credential_type !== 'key_secret' && form.credential_type !== 'custom') {
        form.sensitive.secret_key = '';
    }
    if (form.credential_type !== 'expiring_key') {
        form.sensitive.expires_at = '';
    }
    if (form.credential_type !== 'custom') {
        form.sensitive.access_token = '';
        form.sensitive.refresh_token = '';
    }
};

export const shouldShowField = (
    type: CredentialTemplateKey,
    field: keyof CredentialFormModel['sensitive'],
): boolean => {
    if (type === 'custom') return true;

    // 检查所有模板选项中是否定义了字段
    const template = credentialTemplateOptions.find((t) => t.value === type);
    if (template && template.fields && template.fields.length > 0) {
        return template.fields.includes(field);
    }

    // 默认字段逻辑（向后兼容）
    switch (type) {
        case 'account':
            return field === 'password';
        case 'api_key':
            return field === 'api_key';
        case 'key_secret':
            return field === 'api_key' || field === 'secret_key';
        case 'expiring_key':
            return field === 'api_key' || field === 'expires_at';
        default:
            return false;
    }
};

export const getCredentialTemplateLabel = (type: CredentialTemplateKey): string => {
    return credentialTemplateOptions.find((option) => option.value === type)?.label ?? '自定义';
};
