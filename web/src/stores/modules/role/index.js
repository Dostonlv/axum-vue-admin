// src/stores/roleStore.js

import { defineStore } from 'pinia';
import * as roleApi from '@/api/modules/roles';

export const useRoleStore = defineStore('role', {
    state: () => ({
        // Role list
        roles: [],

        // Pagination
        pagination: {
            page: 1,
            pageSize: 10,
            total: 0,
        },

        // Table loading state
        isTableLoading: false,
    }),

    actions: {
        // Fetch the role list
        async fetchRoles(params) {
            this.isTableLoading = true;
            try {
                const response = await roleApi.getRoles(params);
                this.roles = response.data;
                this.pagination.total = response.total;
                // Keep the local pagination in sync
                this.pagination.page = params.page;
                this.pagination.pageSize = params.pageSize;
            } finally {
                this.isTableLoading = false;
            }
        },

        // Create a role
        async createRole(data) {
            await roleApi.createRole(data);
            await this.fetchRoles({ page: 1, pageSize: this.pagination.pageSize });
        },

        // Update a role
        async updateRole(id, data) {
            await roleApi.updateRole(id, data);
            await this.fetchRoles({ page: this.pagination.page, pageSize: this.pagination.pageSize });
        },

        // Delete a role
        async deleteRole(id) {
            await roleApi.deleteRole(id);
            await this.fetchRoles({ page: this.pagination.page, pageSize: this.pagination.pageSize });
        },
    },
});