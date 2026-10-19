import http from '../http';

// Policy list (paginated, searchable)
export const getPolicies = (params) => {
    return http.get('/cedar_policies', { params });
};

// Create a policy
export const createPolicy = (data) => {
    return http.post('/cedar_policies', data);
};

export const getPolicyByID = (id) => {
    return http.get(`/cedar_policies/${id}`);
};

// Update a policy
export const updatePolicy = (id, data) => {
    return http.put(`/cedar_policies/${id}`, data);
};

// Delete a policy
export const deletePolicy = (id) => {
    return http.delete(`/cedar_policies/${id}`);
};

// --- Non-CRUD operations ---
export const syncPolicies = () => {
    return http.post('/cedar_policies/cache');
};