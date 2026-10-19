import axios from 'axios';
import { useUserStore } from '@/stores';
import i18n from '@/i18n';

const { t } = i18n.global;

const http = axios.create({
    baseURL: import.meta.env.VITE_BASE_API,
    timeout: 10000,
    headers: { 'Content-Type': 'application/json' },
});


function cleanObject(obj) {
    if (obj === null || typeof obj !== 'object') {
        return obj;
    }

    const cleaned = Array.isArray(obj) ? [] : {};

    for (const key in obj) {
        if (Object.prototype.hasOwnProperty.call(obj, key)) {
            const value = obj[key];

            if (typeof value === 'object' && value !== null) {
                const cleanedValue = cleanObject(value);

                if (Object.keys(cleanedValue).length > 0) {
                    cleaned[key] = cleanedValue;
                }

            }
            else if (value !== null && value !== undefined && value !== '') {
                cleaned[key] = value;
            }
        }
    }

    return cleaned;
}

// --- Request interceptor ---
http.interceptors.request.use(
    (config) => {
        // window.$loadingBar?.start();

        if (config.method === 'post' || config.method === 'put') {
            if (config.data && typeof config.data === 'object' && !(config.data instanceof FormData)) {
                config.data = cleanObject(config.data);
            }
        }

        if (config.noNeedToken) {
            return config;
        }

        const userStore = useUserStore();
        const token = userStore.accessToken;
        if (token) {
            config.headers.Authorization = `Bearer ${token}`;
        }
        return config;
    },
    (error) => {
        return Promise.reject(error);
    }
);


// --- Response interceptor ---

// Manages token refresh so concurrent requests don't refresh twice
let isRefreshing = false;
let failedQueue = [];

const processQueue = (error, token = null) => {
    failedQueue.forEach(prom => {
        if (error) {
            prom.reject(error);
        } else {
            prom.resolve(token);
        }
    });
    failedQueue = [];
};

http.interceptors.response.use(
    (response) => {
        // window.$loadingBar?.finish();
        return response.data;
    },
    async (error) => {
        // window.$loadingBar?.error();
        const originalRequest = error.config;
        const userStore = useUserStore();

        // Token refresh
        if (error.response?.status === 401 && !originalRequest._retry) {
            if (isRefreshing) {
                return new Promise(function (resolve, reject) {
                    failedQueue.push({ resolve, reject });
                }).then(token => {
                    originalRequest.headers['Authorization'] = 'Bearer ' + token;
                    return http(originalRequest);
                }).catch(err => {
                    return Promise.reject(err);
                });
            }

            originalRequest._retry = true;
            isRefreshing = true;

            try {
                const newAccessToken = await userStore.refreshToken();
                originalRequest.headers['Authorization'] = `Bearer ${newAccessToken}`;
                processQueue(null, newAccessToken);
                return http(originalRequest);
            } catch (refreshError) {
                processQueue(refreshError, null);
                await userStore.logout();
                // Return a more specific error instead of the raw refresh error
                return Promise.reject({ message: t('api.session_expired') });
            } finally {
                isRefreshing = false;
            }
        }

        const customError = {
            message: t('api.unknown_error'),
            statusCode: error.response?.status,
            originalError: error,
            data: error.response?.data,
        };

        if (error.response) {
            const serverMessage = error.response.data?.message || error.response.data?.message;
            switch (error.response.status) {
                case 400: customError.message = serverMessage || t('api.bad_request'); break;
                case 403: customError.message = serverMessage || t('api.forbidden'); break;
                case 404: customError.message = serverMessage || `${t('api.not_found')} (${originalRequest.url})`; break;
                case 500: customError.message = serverMessage || t('api.server_error'); break;
                default: customError.message = serverMessage || `${t('api.request_failed')} (${error.response.status})`;
            }
        } else if (error.request) {
            customError.message = t('api.network_error');
        } else if (axios.isCancel(error)) {
            console.log('Request canceled:', error.message);
            customError.message = t('api.cancelled');
        }

        // Reject with the formatted error and let the caller deal with it
        return Promise.reject(customError);
    }
);

export default http;