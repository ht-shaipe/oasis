import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import { useI18n } from 'vue-i18n';
import type { HeaderEntry, HttpResponse } from '../../types';

export function useHttpRequest() {
    const { t } = useI18n();

    const method = ref('GET');
    const url = ref('');
    const headers = ref<HeaderEntry[]>([{ key: '', value: '', description: '' }]);
    const body = ref('');
    const loading = ref(false);
    const response = ref<HttpResponse | null>(null);
    const error = ref('');

    function addHeader() {
        headers.value.push({ key: '', value: '' });
    }

    function removeHeader(index: number) {
        headers.value.splice(index, 1);
    }

    function getErrorMessage(err: any): string {
        const message = err?.message || '';
        
        // Common fetch errors
        if (message.includes('Failed to fetch')) {
            return 'Network error: Unable to connect. Please check:\n• Is the URL correct?\n• Is the server running?\n• Is there a CORS issue? (Try using a CORS proxy)\n• Is your internet connection working?';
        }
        if (message.includes('CORS')) {
            return 'CORS error: The server does not allow cross-origin requests. Try using a CORS proxy or contact the API provider.';
        }
        if (message.includes('timeout')) {
            return 'Request timeout: The server took too long to respond. Please try again later.';
        }
        if (message.includes('Invalid URL') || message.includes('URL')) {
            return 'Invalid URL: Please check the URL format. It should start with http:// or https://';
        }
        
        return message || t('toolbox.httpRequest.requestFailed');
    }

    function buildHeaders(): Record<string, string> {
        const result: Record<string, string> = {};
        for (const h of headers.value) {
            const key = h.key.trim();
            if (key) {
                result[key] = h.value;
            }
        }
        return result;
    }

    async function send() {
        const trimmedUrl = url.value.trim();
        if (!trimmedUrl) {
            ElMessage.warning(t('toolbox.httpRequest.urlRequired'));
            return;
        }

        loading.value = true;
        response.value = null;
        error.value = '';

        const startTime = performance.now();

        try {
            const fetchHeaders = buildHeaders();
            const hasBody = method.value !== 'GET' && method.value !== 'HEAD' && body.value.trim();

            if (hasBody && !fetchHeaders['Content-Type']) {
                fetchHeaders['Content-Type'] = 'application/json';
            }

            const fetchOptions: RequestInit = {
                method: method.value,
                headers: fetchHeaders,
            };

            if (hasBody) {
                fetchOptions.body = body.value;
            }

            const res = await fetch(trimmedUrl, fetchOptions);
            const duration = Math.round(performance.now() - startTime);

            const responseHeaders: Record<string, string> = {};
            res.headers.forEach((value, key) => {
                responseHeaders[key] = value;
            });

            const responseBody = await res.text();
            const size = new Blob([responseBody]).size;

            response.value = {
                status: res.status,
                statusText: res.statusText,
                headers: responseHeaders,
                body: responseBody,
                duration,
                size,
            };
        } catch (err: any) {
            const duration = Math.round(performance.now() - startTime);
            const errorMessage = getErrorMessage(err);
            error.value = errorMessage;
            response.value = {
                status: 0,
                statusText: 'Error',
                headers: {},
                body: '',
                duration,
                size: 0,
            };
        } finally {
            loading.value = false;
        }
    }

    return {
        method,
        url,
        headers,
        body,
        loading,
        response,
        error,
        addHeader,
        removeHeader,
        send,
    };
}
