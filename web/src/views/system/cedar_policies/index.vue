<script setup>
import {h, onMounted, ref, reactive, resolveDirective, withDirectives, computed} from 'vue';
import {
  NButton, NFlex, NForm, NFormItem, NInput, NPopconfirm, NSwitch, NTag, NSelect
} from 'naive-ui';
import {storeToRefs} from 'pinia';
import {usePolicyStore} from '@/stores';
import {useRouter} from 'vue-router';
import {renderIcon} from '@/utils';
import CommonPage from '@/components/page/CommonPage.vue';
import QueryBarItem from '@/components/query-bar/QueryBarItem.vue';
import CrudModal from '@/components/table/CrudModal.vue';
import CrudTable from '@/components/table/CrudTable.vue';
import TheIcon from '@/components/icon/TheIcon.vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n({ useScope: 'global' });

const $message = window.$message;
const $dialog = window.$dialog;

defineOptions({name: 'PolicyManagement'});


const store = usePolicyStore();
const {policies, pagination, isTableLoading, isSyncing} = storeToRefs(store);
const router = useRouter();
const vPermission = resolveDirective('permission');


const showModal = ref(false);
const modalType = ref('create');
const modalTitle = computed(() => (modalType.value === 'create' ? t('views.system.policy.new') : t('views.system.policy.edit')));
const currentItem = ref({});
const isSubmitting = ref(false);

const queryParams = reactive({});


onMounted(() => {
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
  loadData(1, 10)
});


const loadData = (page, pageSize) => {
  store.fetchPolicies({...queryParams, page, pageSize})
      .catch(error => $message.error(`${t('views.system.policy.load_failed')}: ${error.message}`));
};

const handlePageChange = (page) => loadData(page, pagination.value.pageSize);
const handlePageSizeChange = (pageSize) => loadData(1, pageSize);
const handleSearch = () => loadData(1, pagination.value.pageSize);
const handleResetSearch = () => {
  for (const key in queryParams) {
    delete queryParams[key]
  }
  handleSearch()
}

// CRUD handlers
const handleAdd = () => {
  currentItem.value = {is_active: true};
  modalType.value = 'create';
  showModal.value = true;
};

const handleEdit = async (row) => {
  let data = await store.fetchPolicyByID(row.uuid)
  currentItem.value = {...data};
  modalType.value = 'edit';
  showModal.value = true;
};

const handleSave = async () => {
  isSubmitting.value = true;
  try {
    if (modalType.value === 'create') {
      await store.createPolicy(currentItem.value);
      $message.success(t('common.messages.create_success'));
    } else {
      await store.updatePolicy(currentItem.value.id, currentItem.value);
      $message.success(t('common.messages.update_success'));
    }
    showModal.value = false;
  } catch (error) {
    $message.error(`${t('common.messages.save_failed')}: ${error.message}`);
  } finally {
    isSubmitting.value = false;
  }
};

const handleDelete = async (row) => {
  try {
    await store.deletePolicy(row.id);
    $message.success(t('common.messages.delete_success'));
  } catch (error) {
    $message.error(`${t('common.messages.delete_failed')}: ${error.message}`);
  }
};

// Special actions
const handleSyncPolicies = async () => {
  try {
    await store.syncPolicies();
    $message.success(t('views.system.policy.sync_success'));
  } catch (error) {
    $message.error(`${t('views.system.policy.sync_failed')}: ${error.message}`);
  }
};


const columns = [
  {
    title: t('views.system.policy.annotation'),
    key: 'annotation',
    width: 450,
    align: 'center',
    ellipsis: {tooltip: true},
  },
  {
    title: t('views.system.policy.effect'),
    key: 'effect',
    width: 80,
    align: 'center',
    ellipsis: {tooltip: true},
  },
  {
    title: t('views.system.policy.type'),
    key: 'policy_type',
    width: 80,
    align: 'center',
    ellipsis: {tooltip: true},
  },
  {
    title: t('common.fields.status'),
    key: 'is_active',
    width: 100,
    align: 'center',
    render(row) {
      return h(NSwitch,
          {defaultValue: row.is_active === 1, disabled: true},
          {
            checked: () => t('common.status.enabled'),
            unchecked: () => t('common.status.disabled')
          })
    },
  },
  {
    title: t('common.fields.description'),
    key: 'description',
    width: 150,
    align: 'center',
  },
  {
    title: t('common.fields.operate'),
    key: 'actions',
    width: 350,
    align: 'center',
    render(row) {
      return [
        withDirectives(
            h(NButton,
                {
                  size: 'small',
                  type: 'primary',
                  style: 'margin-right: 8px;',
                  onClick: () => handleEdit(row)
                },
                {
                  default: () => t('common.buttons.edit'),
                  icon: renderIcon('material-symbols:edit-outline', {size: 16}),
                }
            ),
            [[vPermission, 'button:policy_update']]
        ),
        h(NPopconfirm,
                {onPositiveClick: () => handleDelete(row)},
                {
                  trigger: () => withDirectives( h(
                      NButton,
                      {
                        size: 'small',
                        type: 'error',
                        style: 'margin-right: 8px;',
                      },
                      {
                        default: () => t('common.buttons.delete'),
                        icon: renderIcon('material-symbols:delete-outline', {size: 16}),
                      }
                  ),
                      [[vPermission, 'button:policy_delete']]
                  ),
                  default: () => h('div', {}, t('views.system.policy.confirm_delete'))
                }
            ),
      ];
    },
  },
];

const policyTypeOptions = [
  {
    label: "STATIC",
    value: "STATIC"
  },
  {
    label: "TEMPLATE",
    value: "TEMPLATE"
  }
]
</script>

<template>
  <CommonPage show-footer :title="t('views.system.policy.list')">
    <template #action>
      <NFlex>
        <NButton v-permission="'button:policy_create'" type="primary" @click="handleSyncPolicies" :loading="isSyncing">
          <TheIcon icon="material-symbols:sync" :size="18" class="mr-5"/>
          {{ t('views.system.policy.apply_now') }}
        </NButton>
        <NButton v-permission="'button:policy_create'" type="primary" @click="handleAdd">
          <TheIcon icon="material-symbols:add" :size="18" class="mr-5"/>
          {{ t('views.system.policy.new') }}
        </NButton>
      </NFlex>
    </template>

    <CrudTable
        :columns="columns"
        :data="policies"
        :loading="isTableLoading"
        :pagination="pagination"
        @update:page="handlePageChange"
        @update:page-size="handlePageSizeChange"
        @search="handleSearch"
        @reset="handleResetSearch"
    >
      <template #queryBar>
        <NFlex>
          <NFlex>
            <QueryBarItem :label="t('views.system.policy.annotation')" :label-width="50">
              <NInput
                  v-model:value="queryParams.name"
                  clearable
                  type="text"
                  @keypress.enter="handleSearch()"
              />
            </QueryBarItem>
            <QueryBarItem :label="t('views.system.policy.effect')" :label-width="50">
              <NSelect
                  min-w-sm
                  v-model:value="queryParams.effect"
                  :options="[{label: t('views.system.policy.permit'), value: 'permit'}, {label: t('views.system.policy.forbid'), value: 'forbid'}]"
                  clearable
              />
            </QueryBarItem>
            <QueryBarItem :label="t('common.fields.status')" :label-width="50">
              <NSelect
                  min-w-sm
                  v-model:value="queryParams.is_active"
                  :options="[{label: t('common.status.is_enabled'), value: 1}, {label: t('common.status.not_enabled'), value: 0}]"
                  clearable
              />
            </QueryBarItem>
          </NFlex>
        </NFlex>
      </template>
    </CrudTable>

    <CrudModal
        v-model:visible="showModal"
        :title="modalTitle"
        :loading="isSubmitting"
        @save="handleSave"
    >
      <NForm
          label-placement="left"
          label-align="left"
          :label-width="80"
          :model="currentItem"
          :disabled="modalType === 'view'"
      >
        <NFormItem :label="t('views.system.policy.text')" path="policy_text" required>
          <NInput
              type="textarea"
              v-model:value="currentItem.policy_text"
          />
        </NFormItem>
        <NFormItem :label="t('views.system.policy.policy_type')" path="policy_type" required>
          <NSelect
              :options="policyTypeOptions"
              v-model:value="currentItem.policy_type"
          />
        </NFormItem>
        <NFormItem :label="t('common.fields.description')" path="description" required>
          <NInput
              v-model:value="currentItem.description"
          />
        </NFormItem>
        <NFormItem :label="t('common.fields.status')" path="is_active">
          <NSwitch :value="currentItem.is_active"
                   :default-value="true">
            <template #checked>
              {{ t('common.status.enabled') }}
            </template>
            <template #unchecked>
              {{ t('common.status.disabled') }}
            </template>
          </NSwitch>
        </NFormItem>
      </NForm>
    </CrudModal>
  </CommonPage>
</template>

  