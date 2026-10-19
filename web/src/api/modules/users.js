import http from '../http';

// User list (paginated, searchable)
export const getUsers = (params) => {
    return http.get('/users', { params });
};

// Get one user
export const getUserById = (id) => {
    return http.get(`/users/${id}`);
};

// Create a user
export const createUser = (data) => {
    return http.post('/users', data);
};

// Update a user
// Note: we send the full payload instead of diffing in the UI.
// The backend should only update the fields it receives.
export const updateUser = (id, data) => {
    return http.put(`/users/${id}`, data);
};

// Delete a user
export const deleteUser = (id) => {
    return http.delete(`/users/${id}`);
};

// Bulk delete users
export const batchDeleteUsers = (ids) => {
    return http.delete('/users/batch', { data: { ids } });
};

// Relations
export const getUserRoles = (userId, params) => {
    return http.get(`/users/${userId}/roles`, { params });
}
export const addUserRoles = (userId, roleIds) => {
    return http.post(`/users/${userId}/roles`, { ids: roleIds });
}
export const removeUserRole = (userId, roleId) => {
    return http.delete(`/users/${userId}/roles/${roleId}`);
}