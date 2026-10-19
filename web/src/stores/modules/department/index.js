import { defineStore } from 'pinia';
import * as deptApi from '@/api/modules/departments';

export const useDepartmentStore = defineStore('department', {
    state: () => ({
        departments: [],

        // Loading state
        isLoading: false,
    }),

    getters: {
        departmentOptions: (state) => state.departments,
    },

    actions: {
        // Fetch the full department tree
        async fetchDepartments(params) {
            this.isLoading = true;
            try {
                const response = await deptApi.getDepartments(params);
                this.departments = response.data;
            } finally {
                this.isLoading = false;
            }
        },

        // Create a department
        async createDepartment(data) {
            await deptApi.createDepartment(data);
            // Refetch the whole tree afterwards so the structure stays correct
            await this.fetchDepartments();
        },

        // Update a department
        async updateDepartment(id, data) {
            await deptApi.updateDepartment(id, data);
            await this.fetchDepartments();
        },

        // Delete a department
        async deleteDepartment(id) {
            await deptApi.deleteDepartment(id);
            await this.fetchDepartments();
        },
    },
});