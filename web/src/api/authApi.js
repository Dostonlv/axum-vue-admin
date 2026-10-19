// src/api/authApi.js
import http from './http'; // shared http client

export const authApi = {
    /**
     * Log in
     * @param {object} credentials - { username, password }
     * @returns {Promise<object>} - response with the token
     */
    login: (credentials) => {
        return http.post('/auth/basic/login', credentials);
    },

    /**
     * Log out
     * @returns {Promise<any>}
     */
    logout: () => {
        return http.post('/auth/basic/logout');
    },

    /**
     * Refresh the access token
     * @returns {Promise<object>} - response with the new token
     */
    refreshToken: () => {
        return http.post('/auth/basic/refresh_token');
    },

    /**
     * Fetch the current user's profile
     * @returns {Promise<object>} - user profile
     */
    getCurrentUserProfile: () => {
        return http.get('/me/profile');
    },

    /**
     * Update the current user's profile
     * @param {object} userData - fields to update
     * @returns {Promise<object>} - updated profile
     */
    updateCurrentUserProfile: (userData) => {
        return http.put('/me/profile', userData);
    },

    /**
     * Change the current user's password
     * @param {object} passwordData - { oldPassword, newPassword }
     * @returns {Promise<any>}
     */
    changePassword: (passwordData) => {
        return http.put('/me/password', passwordData);
    },

    /**
     * Forgot password (request a reset)
     * @param {object} resetData - { email: '', language: '' }
     * @returns {Promise<any>}
     */
    forgotPassword: (resetData) => {
        return http.post('/password-resets', resetData);
    },

    /**
     * Reset the password with a token
     * @param {string} resetToken
     * @param {object} resetData - { password }
     * @returns {Promise<any>}
     */
    resetPassword: (resetToken, resetData) => {
        return http.post(`/password-resets/${resetToken}`, resetData);
    },
};