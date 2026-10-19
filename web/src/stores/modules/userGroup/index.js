import { defineStore } from 'pinia';
import * as groupApi from '@/api/modules/userGroups';
import * as roleApi from '@/api/modules/roles'; // reuse the role API

export const useUserGroupStore = defineStore('userGroup', {
    state: () => ({
        // Group list
        groups: [],
        pagination: {
            page: 1,
            pageSize: 10,
            total: 0,
        },
        isTableLoading: false,

        // State for the role assignment drawer
        allRoles: [], // all available roles
        currentGroupRoles: [], // roles of the group being edited
        isRoleTreeLoading: false, // role tree loading state (e.g. while checking/unchecking)
    }),

    getters: {
        // Convert the role list to the format NTree expects
        roleTreeOptions: (state) => {
            return state.allRoles.map(role => ({
                key: role.uuid,
                label: role.name,
            }));
        },
        // Work out which keys to check from the group's roles
        checkedRoleKeys: (state) => {
            return state.currentGroupRoles.map(role => role.uuid);
        },
    },

    actions: {
        async fetchGroups(params) {
            this.isTableLoading = true;
            try {
                const response = await groupApi.getUserGroups(params);
                this.groups = response.data;
                this.pagination.total = response.total;
                this.pagination.page = params.page;
                this.pagination.pageSize = params.pageSize;
            } finally {
                this.isTableLoading = false;
            }
        },
        async createGroup(data) {
            await groupApi.createUserGroup(data);
            await this.fetchGroups({ page: 1, pageSize: this.pagination.pageSize });
        },
        async updateGroup(id, data) {
            await groupApi.updateUserGroup(id, data);
            await this.fetchGroups({ page: this.pagination.page, pageSize: this.pagination.pageSize });
        },
        async deleteGroup(id) {
            await groupApi.deleteUserGroup(id);
            await this.fetchGroups({ page: this.pagination.page, pageSize: this.pagination.pageSize });
        },


        // Fetch all roles to fill the permission tree
        async fetchAllRoles() {
            // Skip if already loaded
            if (this.allRoles.length > 0) return;
            const response = await roleApi.getRoles({ pageSize: 999, fields: 'uuid,name' });
            this.allRoles = response.data;
        },

        // Fetch the roles of one group
        async fetchGroupRoles(groupId) {
            const response = await groupApi.getGroupRoles(groupId);
            this.currentGroupRoles = response.data;
        },

        // Assign a role to a group (on check)
        async assignRoleToGroup(groupId, roleId) {
            this.isRoleTreeLoading = true;
            try {
                await groupApi.addRoleToGroup(groupId, roleId);
                await this.fetchGroupRoles(groupId);
            } finally {
                this.isRoleTreeLoading = false;
            }
        },

        // Revoke a role from a group (on uncheck)
        async revokeRoleFromGroup(groupId, roleId) {
            this.isRoleTreeLoading = true;
            try {
                await groupApi.removeRoleFromGroup(groupId, roleId);
                await this.fetchGroupRoles(groupId);
            } finally {
                this.isRoleTreeLoading = false;
            }
        },
    },
});