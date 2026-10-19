import { defineStore } from 'pinia';
import { resetRouter } from '@/router';
import { useTagsStore } from '@/stores';
import { toLogin } from '@/utils';
import { authApi } from '@/api/authApi';
import i18n from '@/i18n';

const { t } = i18n.global;


export const useUserStore = defineStore('user', {
    state: () => ({
        accessToken: null,
        userInfo: {},
        uiPolicies: [],
        roles: [],
        departments: {},
        groups: [],
        // Tracks the initial auth check so the route guard waits for it
        authStatusChecked: false,
        isDynamicRoutesAdded: false,
    }),

    getters: {
        isAuthenticated: (state) => !!state.accessToken,
        userId: (state) => state.userInfo?.id,
        name: (state) => state.userInfo?.username,
        isSuperUser: (state) => state.userInfo?.is_superuser,
    },

    actions: {
        /**
         * Log in
         * @param {object} credentials
         */
        async login(credentials) {
            try {
                const response = await authApi.login(credentials);
                this.accessToken = response.data.access_token;
                await this.getUserProfile();
            } catch (error) {
                $message.error(error.message || t('views.login.login_failed'));
                console.error('Login failed:', error);
                throw error;
            }
        },

        /**
         * Fetch the current user's profile
         */
        async getUserProfile() {
            try {
                const response = await authApi.getCurrentUserProfile();
                const { id, username, email, avatar, is_superuser, is_active } = response.data.info;
                this.userInfo = { id, username, email, avatar, is_superuser, is_active };
                this.uiPolicies = response.data.ui_policies;
                this.roles = response.data.roles;
                this.departments = response.data.departments;
                this.groups = response.data.groups;
            } catch (error) {
                console.error('Failed to get user profile:', error);
                // If fetching the profile fails (e.g. the token is dead), we could log out here
                await this.logout();
                throw error;
            }
        },

        /**
         * Refresh the token. Mostly called by the http interceptor.
         */
        async refreshToken() {
            try {
                const response = await authApi.refreshToken();
                this.accessToken = response.data.access_token;
                return response.access_token;
            } catch (error) {
                console.error('Failed to refresh token:', error);
                // A failed refresh means the session is gone: clear everything and go to the login page
                this.clearAuthData();
                toLogin();
                throw error;
            }
        },

        /**
         * Log out
         */
        async logout() {
            try {
                // Only call the logout endpoint when there is a token
                if (this.accessToken) {
                    await authApi.logout();
                }
            } catch (error) {
                console.error('Logout API call failed:', error);
            } finally {
                const { resetTags } = useTagsStore();
                if (resetTags) resetTags();

                this.$reset();

                sessionStorage.removeItem(this.$id);

                await resetRouter();
                toLogin();
            }
        },

        async checkAuthStatus() {
            try {
                // If there is a token in session storage, try to load the profile
                if (this.accessToken) {
                    await this.getUserProfile();
                }
            } catch (error) {
                // An error means the session is invalid; handle it silently and leave the user logged out
                console.log("No valid session found or failed to restore session.");
                // Clear any stale token
                this.accessToken = null;
            } finally {
                // Mark the check as done either way
                this.authStatusChecked = true;
            }
        },

        setDynamicRoutesAdded(isAdded) {
            this.isDynamicRoutesAdded = isAdded;
        },

        clearAuthData() {
            this.accessToken = null;
            this.userInfo = {};
            this.UIPolicies = [];
            this.roles = [];
            // ... reset all auth-related state
        },
    },

    persist: {
        // Pinia persistence plugin config
        storage: sessionStorage,
    },
});