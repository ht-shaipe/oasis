import { invoke } from '@tauri-apps/api/core';
import { ref, computed } from 'vue';

// ── Types ──────────────────────────────────────────────────────────────

export interface Category {
    id: number;
    name: string;
    icon: string | null;
    sort_order: number;
    created_at: string;
    parent_id?: number | null;
}

export interface CredentialView {
    id: number;
    category_id: number;
    title: string;
    username: string | null;
    url: string | null;
    tags: string | null;
    notes: string | null;
    created_at: string;
    updated_at: string;
    category_name: string | null;
}

export interface SensitiveData {
    credential_type?: string;
    password?: string;
    secret_key?: string;
    access_token?: string;
    refresh_token?: string;
    api_key?: string;
    expires_at?: string;
    custom_fields?: Record<string, string>;
}

export interface CredentialDetail extends CredentialView {
    sensitive_data: SensitiveData;
}

export interface CreateCredentialRequest {
    category_id: number;
    title: string;
    username?: string;
    url?: string;
    sensitive_data_json: string;
    dekBase64: string;
    nonceBase64: string;
    tags?: string;
    notes?: string;
}

export interface UpdateCredentialRequest {
    id: number;
    category_id?: number;
    title?: string;
    username?: string;
    url?: string;
    sensitive_data_json?: string;
    dekBase64?: string;
    nonceBase64?: string;
    tags?: string;
    notes?: string;
}

// ── Site & Account Types ───────────────────────────────────────────────

export interface SiteAccount {
    username: string;
    password: string;
    api_key?: string;
    secret_key?: string;
}

export interface Site {
    id: number;
    name: string;
    url: string;
    category_id: number;
    tags: string;
    notes: string;
    created_at: string;
    updated_at: string;
    category_name?: string;
    accounts_count?: number;
}

export interface SiteDetail extends Site {
    accounts: SiteAccount[];
}

export interface CreateSiteRequest {
    name: string;
    url?: string;
    category_id: number;
    tags?: string;
    notes?: string;
    accounts: SiteAccount[];
}

export interface UpdateSiteRequest {
    id: number;
    name?: string;
    url?: string;
    category_id?: number;
    tags?: string;
    notes?: string;
    accounts?: SiteAccount[];
}

// ── Composable ─────────────────────────────────────────────────────────

export function useCredential() {
    // ── DEK memory-only store (instance-level) ─────────────────────────────
    // DEK 仅存内存，不持久化（不写入 localStorage / sessionStorage）

    const dek = ref<string | null>(null);
    const isLocked = computed(() => dek.value === null);

    // ── Master key management ──

    const isMasterKeySet = async (): Promise<boolean> => {
        return invoke<boolean>('is_master_key_set');
    };

    const setupMasterKey = async (password: string): Promise<void> => {
        const dekBase64 = await invoke<string>('setup_master_key', { password });
        dek.value = dekBase64;
    };

    const unlock = async (password: string): Promise<void> => {
        const dekBase64 = await invoke<string>('verify_master_key', { password });
        dek.value = dekBase64;
    };

    const lock = () => {
        dek.value = null;
    };

    // ── Category ──

    const listCategories = async (): Promise<Category[]> => {
        return invoke<Category[]>('list_categories');
    };

    const createCategory = async (name: string, icon?: string, parentId?: number): Promise<Category> => {
        return invoke<Category>('create_category', { name, icon, parentId: parentId ?? null });
    };

    const deleteCategory = async (id: number): Promise<void> => {
        return invoke('delete_category', { id });
    };

    // ── Credential ──

    const listCredentials = async (categoryId?: number): Promise<CredentialView[]> => {
        return invoke<CredentialView[]>('list_credentials', { categoryId: categoryId ?? null });
    };

    const getCredential = async (id: number): Promise<CredentialDetail> => {
        if (!dek.value) throw new Error('Vault is locked');
        return invoke<CredentialDetail>('get_credential', { id, dekBase64: dek.value });
    };

    const createCredential = async (data: CreateCredentialRequest): Promise<CredentialView> => {
        if (!dek.value) throw new Error('Vault is locked');
        return invoke<CredentialView>('create_credential', {
            credential: {
                ...data,
                dekBase64: dek.value,
            },
        });
    };

    const updateCredential = async (data: UpdateCredentialRequest): Promise<CredentialView> => {
        if (!dek.value) throw new Error('Vault is locked');
        return invoke<CredentialView>('update_credential', {
            credential: {
                ...data,
                dekBase64: dek.value,
            },
        });
    };

    const deleteCredential = async (id: number): Promise<void> => {
        return invoke('delete_credential', { id });
    };

    const diagnoseCredential = async (id: number): Promise<string> => {
        if (!dek.value) throw new Error('Vault is locked');
        return invoke<string>('diagnose_credential', { id, dekBase64: dek.value, fix: false });
    };

    const fixCredential = async (id: number): Promise<string> => {
        if (!dek.value) throw new Error('Vault is locked');
        return invoke<string>('diagnose_credential', { id, dekBase64: dek.value, fix: true });
    };

    const changeMasterKey = async (oldPassword: string, newPassword: string): Promise<void> => {
        const dekBase64 = await invoke<string>('change_master_key', { oldPassword, newPassword });
        dek.value = dekBase64;
    };

    // ── Site & Account Management ──

    const listSites = async (categoryId?: number): Promise<Site[]> => {
        return invoke<Site[]>('list_sites', { categoryId: categoryId ?? null });
    };

    const getSite = async (id: number): Promise<SiteDetail> => {
        if (!dek.value) throw new Error('Vault is locked');
        return invoke<SiteDetail>('get_site', { id, dekBase64: dek.value });
    };

    const createSite = async (data: CreateSiteRequest): Promise<Site> => {
        if (!dek.value) throw new Error('Vault is locked');
        return invoke<Site>('create_site', {
            site: {
                ...data,
                dekBase64: dek.value,
            },
        });
    };

    const updateSite = async (data: UpdateSiteRequest): Promise<Site> => {
        if (!dek.value) throw new Error('Vault is locked');
        return invoke<Site>('update_site', {
            site: {
                ...data,
                dekBase64: dek.value,
            },
        });
    };

    const deleteSite = async (id: number): Promise<void> => {
        return invoke('delete_site', { id });
    };

    const searchSites = async (query: string): Promise<Site[]> => {
        return invoke<Site[]>('search_sites', { query });
    };

    return {
        // state
        dek,
        isLocked,

        // master key
        isMasterKeySet,
        setupMasterKey,
        unlock,
        lock,
        changeMasterKey,

        // category
        listCategories,
        createCategory,
        deleteCategory,

        // credential
        listCredentials,
        getCredential,
        createCredential,
        updateCredential,
        deleteCredential,

        // site & account
        listSites,
        getSite,
        createSite,
        updateSite,
        deleteSite,
        searchSites,

        // diagnostic
        diagnoseCredential,
        fixCredential,
    };
}
