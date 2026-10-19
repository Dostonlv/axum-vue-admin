
import http from '../http';

export const getDepartments = (params) => {
    return http.get('/departments', { params });
};

// Create a department
export const createDepartment = (data) => {
    return http.post('/departments', data);
};

// Update a department
export const updateDepartment = (id, data) => {
    return http.put(`/departments/${id}`, data);
};

// Delete a department
export const deleteDepartment = (id) => {
    return http.delete(`/departments/${id}`);
};