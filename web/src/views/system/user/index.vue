<script setup>
import {h, onMounted, ref, reactive, resolveDirective, withDirectives, computed} from 'vue'
import {
  NButton, NForm, NFormItem, NInput, NLayout, NLayoutContent, NLayoutSider,
  NPopconfirm, NPopover, NSpace, NSwitch, NTag, NTreeSelect, NSelect, NModal, NDataTable, NTree
} from 'naive-ui'
import {storeToRefs} from 'pinia'
import {useUserManagementStore} from '@/stores'
import {formatDate, renderIcon, isEmail} from '@/utils'

import CommonPage from '@/components/page/CommonPage.vue'
import QueryBarItem from '@/components/query-bar/QueryBarItem.vue'
import CrudModal from '@/components/table/CrudModal.vue'
import CrudTable from '@/components/table/CrudTable.vue'
import TheIcon from '@/components/icon/TheIcon.vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n({ useScope: 'global' })

defineOptions({name: 'UserManagement'})


const store = useUserManagementStore()
const {
  users,
  pagination,
  isTableLoading,
  departmentOptions,
  userGroupOptions,
  roleOptions,
  currentUserRoles,
} = storeToRefs(store)

const vPermission = resolveDirective('permission')


const showCrudModal = ref(false)
const modalType = ref('create')
const modalTitle = computed(() => modalType.value === 'create' ? t('views.system.user.new') : t('views.system.user.edit'))
const currentItem = ref({})
const isSubmitting = ref(false)


const showRoleModal = ref(false)
const currentUserId = ref(null)
const selectedRoleIds = ref([])


const queryParams = reactive({})


onMounted(() => {
  store.fetchDependencies()
  handleSearch()
})

// --- Event handlers ---
const loadData = () => {
  console.log("queryParams", queryParams)
  store.fetchUsers({
    ...queryParams,
    page: pagination.value.page,
    pageSize: pagination.value.pageSize,
  }).catch(error => {
    $message.error(`${t('views.system.user.load_failed')}: ${error.message}`)
  })
}

// Search event from CrudTable
const handleSearch = () => {
  pagination.value.page = 1 // back to the first page on search
  loadData()
}

// Reset event
const handleResetSearch = () => {
  for (const key in queryParams) {
    delete queryParams[key]
  }
  handleSearch()
}

// Page change event
const handlePageChange = (page) => {
  pagination.value.page = page
  loadData()
}

// Page size change event
const handlePageSizeChange = (pageSize) => {
  pagination.value.pageSize = pageSize
  pagination.value.page = 1
  loadData()
}

// CURD

const handleAdd = () => {
  currentItem.value = { is_active: true }
  modalType.value = 'create'
  showCrudModal.value = true
}

const handleEdit = (row) => {
  // store.fetchUsers()
  currentItem.value = {
    ...row,
    dept: row.dept?.uuid,
    groups: row.groups?.map(g => g.uuid),
  }
  modalType.value = 'edit'
  showCrudModal.value = true
}

const handleSave = async () => {
  isSubmitting.value = true
  try {
    if (modalType.value === 'create') {
      await store.createUser(currentItem.value)
      $message.success(t('common.messages.create_success'))
    } else {
      await store.updateUser(currentItem.value.uuid, currentItem.value)
      $message.success(t('common.messages.update_success'))
    }
    loadData()
    showCrudModal.value = false
  } catch (error) {
    $message.error(`${t('common.messages.operation_failed')}: ${error.message}`)
  } finally {
    isSubmitting.value = false
  }
}

const handleDelete = async (row) => {
  try {
    await store.deleteUser(row.uuid)
    $message.success(t('common.messages.delete_success'))
  } catch (error) {
    $message.error(`${t('common.messages.delete_failed')}: ${error.message}`)
  }
}

// Role Modal
const handleViewRoles = async (row) => {
  currentUserId.value = row.uuid
  selectedRoleIds.value = []
  try {
    await store.fetchUserRoles(row.uuid)
    showRoleModal.value = true
  } catch (error) {
    $message.error(`${t('views.system.user.roles_load_failed')}: ${error.message}`)
  }
}

const handleAddRoles = async () => {
  if (!selectedRoleIds.value || selectedRoleIds.value.length === 0) {
    $message.warning(t('views.system.user.select_role'));
    return;
  }
  try {
    await store.addUserRoles(currentUserId.value, selectedRoleIds.value);
    $message.success(t('views.system.user.role_added'));
    selectedRoleIds.value = [];
  } catch (error) {
    $message.error(`${t('views.system.user.add_failed')}: ${error.message}`);
  }
}

const handleRemoveRole = async (roleId) => {
  try {
    await store.removeUserRole(currentUserId.value, roleId);
    $message.success(t('views.system.user.role_removed'));
  } catch (error) {
    $message.error(`${t('views.system.user.remove_failed')}: ${error.message}`);
  }
}


const columns = [
  {
    title: t('views.system.user.col_name'),
    key: 'username',
    width: 60,
    align: 'center',
    ellipsis: {tooltip: true},
  },
  {
    title: t('views.system.user.email'),
    key: 'email',
    width: 60,
    align: 'center',
    ellipsis: {tooltip: true},
  },
  {
    title: t('views.system.user.group'),
    width: 100,
    align: 'center',
    render(row) {
      const groups = row.groups ?? []
      if (groups.length === 0) {
        return h('span', h(NTag, {type: 'info', style: {margin: '2px 3px'}}, {default: () => "No Group"}))
      }
      const MAX_VISIBLE = 1;
      const tagsToShow = groups.slice(0, MAX_VISIBLE);
      const hiddenCount = groups.length - MAX_VISIBLE;

      const triggerTags = tagsToShow.map(group =>
          h(NTag, { type: 'info', style: { marginRight: '6px' } }, { default: () => group.name })
      );

      if (hiddenCount > 0) {
        triggerTags.push(h(NTag, { type: 'default' }, { default: () => `+${hiddenCount}` }))
      }

      const allTags = groups.map(group =>
          h(NTag, { type: 'info', style: { margin: '2px' } }, { default: () => group.name })
      );

      if (hiddenCount <= 0) {
        return h('div', { style: { display: 'flex', justifyContent: 'center' } }, triggerTags)
      }

      return h(
          NPopover,
          {
            trigger: 'hover',
            placement: 'top',
            style: { maxWidth: '300px' }
          },
          {
            trigger: () => h('div', { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } }, triggerTags),
            default: () => h('div', { style: { display: 'flex', flexWrap: 'wrap', gap: '4px' } }, allTags)
          }
      )

    }
  },
  {
    title: t('common.fields.dept'),
    key: 'dept.name',
    align: 'center',
    width: 40,
    ellipsis: {tooltip: true},
  },
  {
    title: t('common.fields.status'),
    key: 'is_active',
    width: 80,
    align: 'center',
    render(row) {
      return h(NSwitch,
          { defaultValue: row.is_active, disabled: true },
          {
            checked: () => t('common.status.enabled'),
            unchecked: () => t('common.status.disabled')
          })
    },
  },
  {
    title: t('views.system.user.last_login'),
    key: 'last_login',
    align: 'center',
    width: 80,
    ellipsis: {tooltip: true},
    render(row) {
      return h(
          NButton,
          {size: 'small', type: 'text', ghost: true},
          {
            default: () => (row.last_login !== null ? formatDate(row.last_login) : "N/A"),
            icon: renderIcon('mdi:update', {size: 16}),
          }
      )
    },
  },
  {
    title: t('common.fields.operate'),
    key: 'actions',
    width: 280,
    align: 'center',
    fixed: 'right',
    render(row) {
      return [
        withDirectives(
            h(NButton,
                {
                  size: 'small',
                  type: 'primary',
                  style: 'margin-right: 8px;',
                  onClick: () => handleEdit(row)},
                {default: () => t('common.buttons.edit'), icon: renderIcon('material-symbols:edit', {size: 16})}),
            [[vPermission, 'button:user_update']]
        ),
        withDirectives(
            h(NButton, {
                  size: 'small',
                  type: 'primary',
                  style: 'margin-right: 8px;',
                  onClick: () => handleViewRoles(row)
                },
                {default: () => t('views.system.user.view_roles'), icon: renderIcon('material-symbols:visibility-outline', {size: 16})}),
            [[vPermission, 'button:user_view']]
        ),
        h(NPopconfirm, {onPositiveClick: () => handleDelete(row)},
            {
              trigger: () => withDirectives(
                  h(NButton, {size: 'small', type: 'error'},
                      {default: () => t('common.buttons.delete'), icon: renderIcon('material-symbols:delete-outline', {size: 16})}),
                  [[vPermission, 'button:user_delete']]
              ),
              default: () => t('views.system.user.confirm_delete')
            }
        ),
      ]
    },
  },
]

const userRoleColumns = [
  {
    title: t('views.system.user.role'),
    key: 'role_name',
    align: 'center',
    width: 150,
  },
  {
    title: t('views.system.user.source'),
    key: 'source',
    align: 'center',
    width: 150,
  },
  {
    title: t('views.system.user.group'),
    key: 'group_name',
    align: 'center',
    width: 150,
  },
  {
    title: t('common.fields.operate'),
    key: 'actions',
    render(row) {
      if (row.source === 'direct') {
        return h(NPopconfirm, {onPositiveClick: () => handleRemoveRole(row.uuid)},
            {
              trigger: () => withDirectives(
                  h(NButton,
                      {
                        size: 'small',
                        type: 'error'},
                      {
                        default: () => t('common.buttons.delete'),
                        icon: renderIcon('material-symbols:delete-outline',{size: 16})}),
                  [[vPermission, 'button:user_delete']]
              ),
              default: () => t('views.system.user.confirm_remove_role'),
            }
        )
      }
    }
  }
]

const validateAddUser = {
  username: [
    {
      required: true,
      message: t('views.system.user.enter_name'),
      trigger: ['input', 'blur'],
    },
  ],
  email: [
    {
      required: true,
      message: t('views.system.user.enter_email_address'),
      trigger: ['input', 'change'],
    },
    {
      trigger: ['blur'],
      validator: (rule, value, callback) => {
        const re = /^[a-zA-Z0-9_-]+@[a-zA-Z0-9_-]+(\.[a-zA-Z0-9_-]+)+$/
        if (!re.test(value)) {
          callback(t('views.system.user.invalid_email'))
          return
        }
        callback()
      },
    },
  ],
  password: [
    {
      required: true,
      message: t('views.system.user.enter_password'),
      trigger: ['input', 'blur', 'change'],
    },
  ],
  dept_ids: [
    {
      type: 'array',
      required: true,
      message: t('views.system.user.select_dept_required'),
      trigger: ['blur', 'change'],
    },
  ],
  confirmPassword: [
    {
      required: true,
      message: t('views.system.user.enter_password_again'),
      trigger: ['input'],
    },
    {
      trigger: ['blur'],
      validator: (rule, value) => {
        if (value !== currentItem.value.password) {
          return new Error(t('views.profile.message_password_confirmation_diff'));
        }
        return true;
      },
    },
  ],
}

// Department picker on the left
const nodeProps = ({option}) => {
  return {
    onClick() {
      if (queryParams.dept_uuid === option.uuid) {
        delete queryParams.dept_uuid
        handleSearch()
      } else {
        queryParams.dept_uuid=option.uuid
        handleSearch()
      }
    },
  }
}
</script>

<template>
  <NLayout has-sider wh-full>
    <NLayoutSider bordered content-style="padding: 24px;" :width="240" show-trigger="arrow-circle">
      <h1>{{ t('views.system.dept.list') }}</h1>
      <br/>
      <NTree
          block-line
          :data="departmentOptions"
          key-field="uuid"
          label-field="name"
          default-expand-all
          :node-props="nodeProps"
      />
    </NLayoutSider>
    <NLayoutContent>
      <CommonPage show-footer :title="t('views.system.user.list')">
        <template #action>
          <NButton v-permission="'button:user_create'" type="primary" @click="handleAdd">
            <TheIcon icon="material-symbols:add" :size="18" class="mr-5"/>
            {{ t('views.system.user.new') }}
          </NButton>
        </template>

        <CrudTable
            :columns="columns"
            :data="users"
            :loading="isTableLoading"
            :pagination="pagination"
            @update:page="handlePageChange"
            @update:page-size="handlePageSizeChange"
            @search="handleSearch"
            @reset="handleResetSearch"
        >
          <template #queryBar>
            <QueryBarItem :label="t('views.system.user.col_name')" :label-width="40">
              <NInput
                  v-model:value="queryParams.username"
                  clearable
                  type="text"
                  :placeholder="t('views.system.user.enter_username')"
                  @keypress.enter="handleSearch()"
              />
            </QueryBarItem>
            <QueryBarItem :label="t('views.system.user.email')" :label-width="40">
              <NInput
                  v-model:value="queryParams.email"
                  clearable
                  type="text"
                  :placeholder="t('views.system.user.enter_email')"
                  @keypress.enter="handleSearch()"
              />
            </QueryBarItem>
            <!-- ... -->
          </template>
        </CrudTable>

        <CrudModal
            v-model:visible="showCrudModal"
            :title="modalTitle"
            :loading="isSubmitting"
            @save="handleSave"
        >
          <NForm :model="currentItem" :rules="validateAddUser" label-placement="left" label-width="auto">
            <NFormItem :label="t('views.system.user.username')" path="username">
              <NInput v-model:value="currentItem.username"/>
            </NFormItem>
            <NFormItem :label="t('views.system.user.email')" path="email">
              <NInput v-model:value="currentItem.email" clearable/>
            </NFormItem>
            <NFormItem v-if="modalType === 'create'" :label="t('common.fields.password')" path="password">
              <NInput
                  v-model:value="currentItem.password"
                  show-password-on="mousedown"
                  type="password"
                  clearable
                  :placeholder="t('views.system.user.enter_password')"
              />
            </NFormItem>
            <NFormItem v-if="modalType === 'create'" :label="t('views.profile.label_confirm_password')" path="confirmPassword">
              <NInput
                  v-model:value="currentItem.confirmPassword"
                  show-password-on="mousedown"
                  type="password"
                  clearable
              />
            </NFormItem>
            <NFormItem :label="t('views.system.user.group')" path="group">
              <NSelect
                  :options="userGroupOptions"
                  v-model:value="currentItem.groups"
                  multiple
                  clearable/>
            </NFormItem>
            <NFormItem :label="t('common.fields.dept')" path="dept">
              <NTreeSelect
                  v-model:value="currentItem.dept"
                  :options="departmentOptions"
                  key-field="uuid"
                  label-field="name"
                  :placeholder="t('views.system.user.select_dept')"
                  clearable
                  default-expand-all
              ></NTreeSelect>
            </NFormItem>
            <NFormItem :label="t('common.status.disabled')" path="is_active">
              <NSwitch
                  v-model:value="currentItem.is_active"
                  :checked-value="false"
                  :unchecked-value="true"
                  :default-value="true"
              />
            </NFormItem>
          </NForm>
        </CrudModal>
        <!-- Roles dialog -->
        <NModal :show="showRoleModal"
                :title="t('views.system.user.role')"
                preset="card"
                size="large"
                style="width: 700px"
                @close="showRoleModal = false">
          <NSpace vertical :size="12">
            <NSpace :size="12">
              <NSelect
                  :options="roleOptions"
                  :consistent-menu-width="false"
                  v-model:value="currentItem.ids"
                  style="min-width: 150px"
                  filterable
                  clearable
                  multiple/>
              <NButton @click="handleAddRoles">{{ t('views.system.user.add_role') }}</NButton>
            </NSpace>
            <NDataTable :columns="userRoleColumns" :data="currentUserRoles"/>
          </NSpace>
        </NModal>

      </CommonPage>
    </NLayoutContent>
  </NLayout>
</template>