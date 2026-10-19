
import http from '../http';

// Fetch the schema list
export const getSchemas = (params) => {
    return http.get('/cedar_schema', { params });
};

// Update a schema
export const updateSchema = (id, data) => {
    return http.put(`/cedar_schema/${id}`, data);
};