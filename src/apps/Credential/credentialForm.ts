import type { SensitiveData } from '@/composables/useCredential';

export type CredentialTemplateKey =
    | 'account'
    | 'api_key'
    | 'key_secret'
    | 'expiring_key'
    | 'database'
    | 'server'
    | 'bank_card'
    | 'custom'
    | string;

export interface CredentialAccountForm {
    username: string;
    password: string;
    notes: string;
    api_key: string;
    secret_key: string;
    access_token: string;
    refresh_token: string;
    expires_at: string;
    /** 类型专属字段，如数据库的 host/port/db_name，银行卡的 card_holder/cvv/valid_thru 等 */
    custom_fields: Record<string, string>;
}

export const createEmptyCredentialAccount = (): CredentialAccountForm => ({
    username: '',
    password: '',
    notes: '',
    api_key: '',
    secret_key: '',
    access_token: '',
    refresh_token: '',
    expires_at: '',
    custom_fields: {},
});

export interface CredentialFormModel {
    credential_type: CredentialTemplateKey;
    title: string;
    category_id: number | null;
    username: string;
    url: string;
    api_url: string;
    doc_url: string;
    tags: string;
    notes: string;
    accounts: CredentialAccountForm[];
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

// ── 类型专属字段定义 ─────────────────────────────────────────────────────

export interface CustomFieldDef {
    key: string;
    label: string;
    placeholder?: string;
    /** 是否为密码类型字段（默认显示隐藏） */
    secret?: boolean;
}

/** 每种凭证类型对应的自定义字段定义 */
export const typeCustomFieldDefs: Record<string, CustomFieldDef[]> = {
    database: [
        { key: 'host', label: '主机地址', placeholder: '192.168.1.100' },
        { key: 'port', label: '端口', placeholder: '3306' },
        { key: 'db_type', label: '数据库类型', placeholder: 'MySQL' },
        { key: 'db_name', label: '数据库名', placeholder: 'mydb' },
    ],
    server: [
        { key: 'host', label: '主机地址', placeholder: '192.168.1.100 或 example.com' },
        { key: 'port', label: '端口', placeholder: '22' },
    ],
    bank_card: [
        { key: 'bank_name', label: '银行名称', placeholder: '招商银行' },
        { key: 'card_number', label: '卡号', placeholder: '**** **** **** ****', secret: true },
        { key: 'card_holder', label: '持卡人', placeholder: '张三' },
        { key: 'cvv', label: 'CVV', placeholder: '***', secret: true },
        { key: 'valid_thru', label: '有效期', placeholder: 'MM/YY', secret: true },
    ],
};

/** 获取指定类型的自定义字段定义 */
export const getCustomFieldDefs = (type: CredentialTemplateKey): CustomFieldDef[] => {
    return typeCustomFieldDefs[type] ?? [];
};

/** 判断该类型是否有自定义字段定义 */
export const hasCustomFieldDefs = (type: CredentialTemplateKey): boolean => {
    return (typeCustomFieldDefs[type]?.length ?? 0) > 0;
};

// ── 模板选项 ─────────────────────────────────────────────────────────────

export const defaultCredentialTemplateOptions: Array<CredentialTemplateOption> = [
    {
        value: 'account',
        label: '账号凭证',
        description: '用户名 + 密码，适合网站、系统登录。',
        fields: ['password'],
    },
    {
        value: 'api_key',
        label: 'Api Key',
        description: '只有一个 Key，适合 API Key、访问令牌等。',
        fields: ['api_key'],
    },
    {
        value: 'key_secret',
        label: 'Key Secret',
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
        value: 'database',
        label: '数据库',
        description: '主机 + 端口 + 用户名 + 密码 + 数据库名。',
        fields: ['password'],
    },
    {
        value: 'server',
        label: '服务器',
        description: '主机 + 端口 + 用户名 + 密码/密钥。',
        fields: ['password'],
    },
    {
        value: 'bank_card',
        label: '银行卡',
        description: '银行名 + 卡号 + 密码 + CVV + 有效期等。',
        fields: ['password'],
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
        credentialTemplateOptions = [...defaultCredentialTemplateOptions, ...customTemplates];
    } catch (error) {
        console.error('Failed to save custom templates:', error);
    }
};

// 导出的模板选项（从localStorage加载，可在保存后更新）
export let credentialTemplateOptions = loadCredentialTemplates();

export const defaultCredentialForm = (categoryId: number | null): CredentialFormModel => ({
    credential_type: 'account',
    title: '',
    category_id: categoryId,
    username: '',
    url: '',
    api_url: '',
    doc_url: '',
    tags: '',
    notes: '',
    accounts: [createEmptyCredentialAccount()],
    sensitive: {
        password: '',
        api_key: '',
        secret_key: '',
        expires_at: '',
        access_token: '',
        refresh_token: '',
    },
});

/** 根据 custom_fields 中的特征 key 推断凭证类型 */
const inferFromCustomFields = (customFields?: Record<string, string>): CredentialTemplateKey | null => {
    if (!customFields) return null;
    if (customFields['db_name'] !== undefined) return 'database';
    if (customFields['card_number'] !== undefined) return 'bank_card';
    // server 和 database 都有 host/port，但 server 没有 db_name
    if (customFields['host'] !== undefined && customFields['db_name'] === undefined) return 'server';
    return null;
};

export const inferCredentialType = (sensitiveData?: Partial<SensitiveData> | null): CredentialTemplateKey => {
    if (!sensitiveData) return 'account';

    // 先从 custom_fields 推断
    const fromCustom = inferFromCustomFields(sensitiveData.custom_fields);
    if (fromCustom) return fromCustom;

    if (sensitiveData.sensitive_sets && sensitiveData.sensitive_sets.length > 0) {
        const firstSet = sensitiveData.sensitive_sets[0];
        if (firstSet.secret_key && firstSet.api_key) return 'key_secret';
        if (firstSet.expires_at && firstSet.api_key) return 'expiring_key';
        if (firstSet.api_key && !firstSet.password) return 'api_key';
        if (firstSet.password) return 'account';
    }
    if (sensitiveData.account_sets && sensitiveData.account_sets.length > 0) return 'account';
    if (sensitiveData.secret_key && sensitiveData.api_key) return 'key_secret';
    if (sensitiveData.expires_at && sensitiveData.api_key) return 'expiring_key';
    if (sensitiveData.api_key && !sensitiveData.password) return 'api_key';
    if (sensitiveData.password) return 'account';
    return 'custom';
};

export const buildSensitiveData = (form: CredentialFormModel): SensitiveData => {
    const accountLikeType = form.credential_type === 'account' || form.credential_type === 'custom';
    const normalizedSets = form.accounts
        .map((account) => ({
            username: account.username.trim() || undefined,
            notes: account.notes.trim() || undefined,
            password: account.password || undefined,
            api_key: account.api_key || undefined,
            secret_key: account.secret_key || undefined,
            access_token: account.access_token || undefined,
            refresh_token: account.refresh_token || undefined,
            expires_at: account.expires_at || undefined,
        }))
        .filter(
            (set) =>
                set.username ||
                set.notes ||
                set.password ||
                set.api_key ||
                set.secret_key ||
                set.access_token ||
                set.refresh_token ||
                set.expires_at,
        );

    const accountSets = normalizedSets
        .filter((set) => set.username || set.password || set.notes)
        .map((set) => ({
            username: set.username || '',
            password: set.password,
            notes: set.notes,
        }));

    const primarySet = normalizedSets[0];
    const primaryAccountPassword = primarySet?.password ?? form.sensitive.password;

    // 收集类型专属字段
    const customFieldsFromAccounts: Record<string, string> = {};
    for (const account of form.accounts) {
        for (const [key, value] of Object.entries(account.custom_fields)) {
            if (value.trim()) {
                customFieldsFromAccounts[key] = value.trim();
            }
        }
    }

    const data: SensitiveData = {
        credential_type: form.credential_type,
        api_url: form.api_url.trim() || undefined,
        doc_url: form.doc_url.trim() || undefined,
        sensitive_sets: normalizedSets.length > 0 ? normalizedSets : undefined,
        account_sets: accountLikeType && accountSets.length > 0 ? accountSets : undefined,
        password: accountLikeType ? primaryAccountPassword || undefined : form.sensitive.password || undefined,
        api_key: primarySet?.api_key || form.sensitive.api_key || undefined,
        secret_key: primarySet?.secret_key || form.sensitive.secret_key || undefined,
        expires_at: primarySet?.expires_at || form.sensitive.expires_at || undefined,
        access_token: primarySet?.access_token || form.sensitive.access_token || undefined,
        refresh_token: primarySet?.refresh_token || form.sensitive.refresh_token || undefined,
        custom_fields: Object.keys(customFieldsFromAccounts).length > 0 ? customFieldsFromAccounts : undefined,
    };

    return data;
};

/** 判断是否为"账号类"凭证（用户名+密码模式，支持多套账号） */
export const isAccountLikeType = (type: CredentialTemplateKey): boolean => {
    return type === 'account' || type === 'database' || type === 'server' || type === 'bank_card' || type === 'custom';
};

/** 判断是否为"密钥类"凭证（api_url/doc_url + 密钥字段模式） */
export const isKeyCredentialType = (type: CredentialTemplateKey): boolean => {
    return ['api_key', 'key_secret', 'expiring_key'].includes(type);
};

export const normalizeSensitiveFields = (form: CredentialFormModel): void => {
    if (form.credential_type === 'custom') return;

    const type = form.credential_type;
    const isAccountLike = isAccountLikeType(type);
    const isKeyLike = isKeyCredentialType(type);

    // 账号类：清空密钥字段
    if (isAccountLike) {
        for (const account of form.accounts) {
            account.api_key = '';
            account.secret_key = '';
            account.access_token = '';
            account.refresh_token = '';
            account.expires_at = '';
        }
        // 非账号类：清空密码
    } else {
        form.sensitive.password = '';
        for (const account of form.accounts) {
            account.password = '';
        }
    }

    // 密钥类之间互斥
    if (!isKeyLike && !isAccountLike) {
        form.sensitive.api_key = '';
        for (const account of form.accounts) {
            account.api_key = '';
        }
    }
    if (type !== 'key_secret' && !isAccountLike) {
        form.sensitive.secret_key = '';
        for (const account of form.accounts) {
            account.secret_key = '';
        }
    }
    if (type !== 'expiring_key' && !isAccountLike) {
        form.sensitive.expires_at = '';
        for (const account of form.accounts) {
            account.expires_at = '';
        }
    }

    // 切换类型时，为新类型初始化 custom_fields
    const defs = getCustomFieldDefs(type);
    for (const account of form.accounts) {
        if (!account.custom_fields) account.custom_fields = {};
        // 保留已有的值，补充新定义的 key
        for (const def of defs) {
            if (!(def.key in account.custom_fields)) {
                account.custom_fields[def.key] = '';
            }
        }
        // 清除不属于当前类型的 custom_fields key
        const validKeys = new Set(defs.map((d) => d.key));
        for (const key of Object.keys(account.custom_fields)) {
            if (!validKeys.has(key)) {
                delete account.custom_fields[key];
            }
        }
    }
};

export const templateFieldsLength = (type: CredentialTemplateKey): number => {
    const template = credentialTemplateOptions.find((t) => t.value === type);

    return template && template.fields ? template.fields.length : 0;
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
        case 'database':
        case 'server':
        case 'bank_card':
            return field === 'password';
        default:
            return false;
    }
};

export const getCredentialTemplateLabel = (type: CredentialTemplateKey): string => {
    return credentialTemplateOptions.find((option) => option.value === type)?.label ?? '自定义';
};
