<script setup>
import {h, onMounted, ref, reactive, resolveDirective, withDirectives} from 'vue';
import {
  NButton, NDrawer, NDrawerContent, NForm, NFormItem,
  NInput, NPopconfirm, NTree, NSpin, NTag
} from 'naive-ui';
import {storeToRefs} from 'pinia';
import {useUserGroupStore} from '@/stores';
import {formatDate, renderIcon} from '@/utils';
import CommonPage from '@/components/page/CommonPage.vue';
import TheIcon from '@/components/icon/TheIcon.vue';
import CrudTable from '@/components/table/CrudTable.vue';
import QueryBarItem from '@/components/query-bar/QueryBarItem.vue';
import CrudModal from '@/components/table/CrudModal.vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n({ useScope: 'global' });

const $message = window.$message;

defineOptions({name: 'GroupManagement'});


const store = useUserGroupStore();
const {
  groups,
  pagination,
  isTableLoading,
  roleTreeOptions,
  checkedRoleKeys,
  isRoleTreeLoading,
} = storeToRefs(store);
const vPermission = resolveDirective('permission');


const showCrudModal = ref(false);
const modalType = ref('create');
const currentItem = ref({});
const isSubmitting = ref(false);

const showRoleDrawer = ref(false);
const currentGroupId = ref(null);
const roleFilterPattern = ref(''); // role tree filter text


const queryParams = reactive({});

const loadData = (page, pageSize) => {
  store.fetchGroups({...queryParams, page, pageSize})
      .catch(error => $message.error(`${t('views.system.group.load_failed')}: ${error.message}`));
};

onMounted(() => {
  store.fetchAllRoles();
  loadData(1, 10);
});


const handlePageChange = (page) => loadData(page, pagination.value.pageSize);
const handlePageSizeChange = (pageSize) => loadData(1, pageSize);
const handleSearch = () => loadData(1, pagination.value.pageSize);
const handleResetSearch = () => {
  for (const key in queryParams) {
    delete queryParams[key]
  }
  handleSearch()
}

const handleAdd = () => {
  currentItem.value = {};
  modalType.value = 'create';
  showCrudModal.value = true;
};

const handleEdit = (row) => {
  currentItem.value = {...row};
  modalType.value = 'edit';
  showCrudModal.value = true;
};

const handleSave = async () => {
  isSubmitting.value = true;
  try {
    if (modalType.value === 'create') {
      await store.createGroup(currentItem.value);
      $message.success(t('common.messages.create_success'));
    } else {
      await store.updateGroup(currentItem.value.uuid, currentItem.value);
      $message.success(t('common.messages.update_success'));
    }
    showCrudModal.value = false;
  } catch (error) {
    $message.error(`${t('common.messages.save_failed')}: ${error.message}`);
  } finally {
    isSubmitting.value = false;
  }
};

const handleDelete = async (row) => {
  try {
    await store.deleteGroup(row.uuid);
    $message.success(t('common.messages.delete_success'));
  } catch (error) {
    $message.error(`${t('common.messages.delete_failed')}: ${error.message}`);
  }
};


const handleSetRoles = async (row) => {
  currentGroupId.value = row.uuid;
  try {
    // Load the group's roles; the store updates checkedRoleKeys
    await store.fetchGroupRoles(row.uuid);
    showRoleDrawer.value = true;
  } catch (error) {
    $message.error(`${t('views.system.group.roles_load_failed')}: ${error.message}`);
  }
};

const handleRoleCheckChange = async (keys, option, meta) => {
  if (!currentGroupId.value) return;

  const roleId = meta.node.key;
  try {
    if (meta.action === 'check') {
      await store.assignRoleToGroup(currentGroupId.value, roleId);
    } else if (meta.action === 'uncheck') {
      await store.revokeRoleFromGroup(currentGroupId.value, roleId);
    }
  } catch (error) {
    $message.error(`${t('common.messages.operation_failed')}: ${error.message}`);
    // Reload the roles to roll back the wrong check in the UI
    await store.fetchGroupRoles(currentGroupId.value);
  }
};


const columns = [
  {
    title: t('views.system.group.name'),
    key: 'name',
    width: 80,
    align: 'center',
    ellipsis: {tooltip: true},
    render(row) {
      return h(NTag, {type: 'info'}, {default: () => row.name})
    },
  },
  {
    title: t('views.system.group.description'),
    key: 'description',
    width: 80,
    align: 'center',
  },
  {
    title: t('common.fields.created_date'),
    key: 'created_at',
    width: 60,
    align: 'center',
    render(row) {
      return h('span', formatDate(row.created_at))
    },
  },
  {
    title: t('common.fields.operate'),
    key: 'actions',
    width: 260,
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
                  onClick: () => {
                    handleEdit(row)
                  },
                },
                {
                  default: () => t('common.buttons.edit'),
                  icon: renderIcon('material-symbols:edit-outline', {size: 16}),
                }
            ),
            [[vPermission, 'button:group_update']]
        ),
        h(NPopconfirm,
            {
              onPositiveClick: () => handleDelete(row)
            },
            {
              trigger: () => withDirectives(
                  h(
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
                  [[vPermission, 'button:group_delete']]
              ),
              default: () => h('div', {}, t('views.system.group.confirm_delete')),
            }
        ),

        withDirectives(
            h(NButton, {
                  size: 'small',
                  type: 'primary',
                  onClick: () => handleSetRoles(row),
                },
                {default: () => t('views.system.group.set_roles'), icon: renderIcon('icon-park-outline:permissions', {size: 16})}
            ),
            [[vPermission, 'button:group_update']]
        ),
      ];
    },
  },
];
</script>

<template>
  <CommonPage show-footer :title="t('views.system.group.list')">
    <template #action>
      <NButton v-permission="'button:group_create'" type="primary" @click="handleAdd">
        <TheIcon icon="material-symbols:add" :size="18" class="mr-5"/>
        {{ t('views.system.group.new') }}
      </NButton>
    </template>

    <CrudTable
        :columns="columns"
        :data="groups"
        :loading="isTableLoading"
        :pagination="pagination"
        @update:page="handlePageChange"
        @update:page-size="handlePageSizeChange"
        @search="handleSearch"
        @reset="handleResetSearch"
    >
      <template #queryBar>
        <QueryBarItem :label="t('views.system.group.name')" :label-width="60">
          <NInput
              v-model:value="queryParams.name"
              clearable
              type="text"
              :placeholder="t('views.system.group.enter_name')"
              @keypress.enter="handleSearch()"
          />
        </QueryBarItem>
      </template>
    </CrudTable>

    <CrudModal
        v-model:visible="showCrudModal"
        :title="modalType === 'create' ? t('views.system.group.new') : t('views.system.group.edit')"
        :loading="isSubmitting"
        @save="handleSave"
    >
      <NForm
          ref="modalFormRef"
          label-placement="left"
          label-align="left"
          :label-width="80"
          :model="userGroupCrud.state.currentItem"
          :disabled="userGroupCrud.state.modalType === 'view'"
      >
        <NFormItem
            :label="t('views.system.group.name')"
            path="name"
            :rule="{
            required: true,
            message: t('views.system.group.enter_name'),
            trigger: ['input', 'blur'],
          }"
        >
          <NInput v-model:value="currentItem.name" :placeholder="t('views.system.group.enter_name')"/>
        </NFormItem>
        <NFormItem :label="t('views.system.group.description')" path="desc">
          <NInput v-model:value="currentItem.description" :placeholder="t('views.system.group.enter_description')"/>
        </NFormItem>
      </NForm>
    </CrudModal>

    <NDrawer v-model:show="showRoleDrawer" placement="right" :width="500">
      <NDrawerContent :title="t('views.system.group.set_roles')">
        <NSpace vertical :size="20">
          <NInput
              v-model:value="roleFilterPattern"
              :placeholder="t('views.system.group.filter_roles')"
              clearable
              size="large"
          />

          <NSpin :show="isRoleTreeLoading" :description="t('views.system.group.roles_busy')" size="medium">
            <div :style="{ minHeight: '300px', maxHeight: 'calc(100vh - 250px)', overflowY: 'auto' }">
              <NTree
                  block-line
                  checkable
                  cascade
                  :data="roleTreeOptions"
                  :pattern="roleFilterPattern"
                  key-field="uuid"
                  :show-irrelevant-nodes="false"
                  :checked-keys="checkedRoleKeys"
                  @update:checked-keys="handleRoleCheckChange"
              />
            </div>
          </NSpin>
        </NSpace>
      </NDrawerContent>
    </NDrawer>
  </CommonPage>
</template>