import type { SensitiveData } from '@/composables/useCredential';

export type CredentialTemplateKey = 'account' | 'api_key' | 'key_secret' | 'expiring_key' | 'custom';

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

export const credentialTemplateOptions: Array<{
    value: CredentialTemplateKey;
    label: string;
    description: string;
}> = [
    {
        value: 'account',
        label: '账号凭证',
        description: '用户名 + 密码，适合网站、系统登录。',
    },
    {
        value: 'api_key',
        label: '密钥类',
        description: '只有一个 Key，适合 API Key、访问令牌等。',
    },
    {
        value: 'key_secret',
        label: 'Key + Secret',
        description: '同时保存 Key 和 Secret 的双字段凭证。',
    },
    {
        value: 'expiring_key',
        label: '到期密钥',
        description: 'Key + 到期时间，适合会过期的证书或授权。',
    },
    {
        value: 'custom',
        label: '自定义',
        description: '保留全部字段，自由组合。',
    },
];

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
