import { defineStore } from 'pinia';
import * as schemaApi from '@/api/modules/cedarSchemas';

export const useSchemaStore = defineStore('schema', {
    state: () => ({
        // Schema list
        schemas: [],
        // Loading state
        isLoading: false,
    }),

    actions: {
        // Fetch the schema list
        async fetchSchemas() {
            this.isLoading = true;
            try {
                const response = await schemaApi.getSchemas();
                this.schemas = response.data;
            } finally {
                this.isLoading = false;
            }
        },

        // Update a schema
        async updateSchema(id, data) {
            await schemaApi.updateSchema(id, data);
            await this.fetchSchemas();
        },
    },
});