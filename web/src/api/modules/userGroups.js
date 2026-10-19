import http from '../http';

// --- Group CRUD ---
export const getUserGroups = (params) => http.get('/groups', { params });
export const createUserGroup = (data) => http.post('/groups', data);
export const updateUserGroup = (id, data) => http.put(`/groups/${id}`, data);
export const deleteUserGroup = (id) => http.delete(`/groups/${id}`);

// --- Group/role relations ---
// Roles already held by a group
export const getGroupRoles = (groupId) => http.get(`/groups/${groupId}/roles`);

// Add a role to a group
export const addRoleToGroup = (groupId, roleId) => http.post(`/groups/${groupId}/roles`, { role_id: roleId });

// Remove a role from a group
export const removeRoleFromGroup = (groupId, roleId) => http.delete(`/groups/${groupId}/roles/${roleId}`);