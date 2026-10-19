<script setup>
import { h, onMounted, ref, resolveDirective, withDirectives, computed } from 'vue';
import { NButton, NForm, NFormItem, NInput } from 'naive-ui';
import { useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import { useSchemaStore } from '@/stores';
import { renderIcon } from '@/utils';
import CommonPage from '@/components/page/CommonPage.vue';
import CrudModal from '@/components/table/CrudModal.vue';
import CrudTable from '@/components/table/CrudTable.vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n({ useScope: 'global' });

const $message = window.$message;
const $dialog = window.$dialog;

defineOptions({ name: 'SchemaManagement' });


const store = useSchemaStore();
const { schemas, isLoading } = storeToRefs(store);
const router = useRouter();
const vPermission = resolveDirective('permission');


const showModal = ref(false);
const modalTitle = ref(t('views.system.schema.edit'));
const currentItem = ref({});
const isSubmitting = ref(false);


onMounted(() => {
  store.fetchSchemas().catch(error => {
    $message.error(`${t('views.system.schema.load_failed')}: ${error.message}`);
  });


  setTimeout(() => {
    $dialog.warning({
      title: t('views.system.cedar.warn_title'),
      content: t('views.system.cedar.warn_content'),
      positiveText: t('views.system.cedar.warn_ok'),
      negativeText: t('views.system.cedar.warn_cancel'),
      maskClosable: false,
      closable: false,
      onNegativeClick: () => router.push('/'),
    });
  }, 500);
});


const handleEdit = (row) => {
  currentItem.value = { ...row };
  showModal.value = true;
};

const handleSave = async () => {
  isSubmitting.value = true;
  try {
    await store.updateSchema(currentItem.value.id, currentItem.value);
    $message.success(t('common.messages.update_success'));
    showModal.value = false;
  } catch (error) {
    $message.error(`${t('common.messages.update_failed')}: ${error.message}`);
  } finally {
    isSubmitting.value = false;
  }
};


const columns = [
  {
    title: t('common.fields.description'),
    key: 'description',
    width: 80,
    align: 'center',
  },
  {
    title: t('common.fields.operate'),
    key: 'actions',
    width: 80,
    align: 'center',
    fixed: 'right',
    render(row) {
      return [
        withDirectives(
            h(
                NButton,
                {
                  size: 'small',
                  type: 'primary',
                  style: 'margin-right: 8px;',
                  onClick: () => handleEdit(row),
                },
                {
                  default: () => t('common.buttons.edit'),
                  icon: renderIcon('material-symbols:edit-outline', { size: 16 }),
                }
            ),
            [[vPermission, 'button:policy_update']]
        ),
      ];
    },
  },
];
</script>

<template>
  <CommonPage show-footer :title="t('views.system.schema.list')">
    <CrudTable
        :columns="columns"
        :data="schemas"
        :loading="isLoading"
        :pagination="false"
    />

    <CrudModal
        v-model:visible="showModal"
        :title="modalTitle"
        :loading="isSubmitting"
        @save="handleSave"
    >
      <NForm :model="currentItem">
        <NFormItem label="Schema" path="schema" required>
          <NInput
              type="textarea"
              v-model:value="currentItem.schema"
              :autosize="{ minRows: 10, maxRows: 20 }"
          />
        </NFormItem>
        <NFormItem :label="t('common.fields.description')" path="description" required>
          <NInput v-model:value="currentItem.description" />
        </NFormItem>
      </NForm>
    </CrudModal>
  </CommonPage>
</template>