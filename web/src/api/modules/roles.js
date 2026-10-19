
import http from '../http';

// Role list (paginated, searchable)
export const getRoles = (params) => {
    return http.get('/roles', { params });
};

// Get one role
export const getRoleById = (id) => {
    return http.get(`/roles/${id}`);
};

// Create a role
export const createRole = (data) => {
    return http.post('/roles', data);
};

// Update a role
export const updateRole = (id, data) => {
    return http.put(`/roles/${id}`, data);
};

// Delete a role
export const deleteRole = (id) => {
    return http.delete(`/roles/${id}`);
};