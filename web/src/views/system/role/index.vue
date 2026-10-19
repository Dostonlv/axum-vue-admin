<script setup>
import {h, onMounted, ref, reactive, resolveDirective, withDirectives, computed} from 'vue'
import {
  NButton,
  NForm,
  NFormItem,
  NInput,
  NPopconfirm,
  NTag,
} from 'naive-ui'
import {storeToRefs} from 'pinia'
import {useRoleStore} from '@/stores'
import {formatDate, renderIcon} from '@/utils'

import CommonPage from '@/components/page/CommonPage.vue'
import QueryBarItem from '@/components/query-bar/QueryBarItem.vue'
import CrudModal from '@/components/table/CrudModal.vue'
import CrudTable from '@/components/table/CrudTable.vue'
import TheIcon from '@/components/icon/TheIcon.vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n({ useScope: 'global' })

const $message = window.$message

defineOptions({name: 'RoleManagement'})

// =============================
const roleStore = useRoleStore()
const {roles, pagination, isTableLoading} = storeToRefs(roleStore)
const vPermission = resolveDirective('permission')

const showModal = ref(false)
const modalType = ref('create') // 'create' | 'edit'
const modalTitle = computed(() => (modalType.value === 'create' ? t('views.system.role.new') : t('views.system.role.edit')))
const currentItem = ref({}) // bound to the form via v-model
const isSubmitting = ref(false)

// Search params
const queryParams = reactive({})


const loadData = () => {
  roleStore.fetchRoles({
    ...queryParams,
    page: pagination.value.page,
    pageSize: pagination.value.pageSize,
  })
      .catch(error => {
        $message.error(`${t('views.system.role.load_failed')}: ${error.message}`)
      })
}

onMounted(() => {
  loadData()
});

const handleSearch = () => {
  pagination.value.page = 1
  loadData()
}

const handleResetSearch = () => {
  for (const key in queryParams) {
    delete queryParams[key]
  }
  handleSearch()
}

const handlePageChange = (page) => {
  loadData(page, pagination.value.pageSize);
}

const handlePageSizeChange = (pageSize) => {
  loadData(1, pageSize);
}

// CRUD
const handleAdd = () => {
  currentItem.value = {}
  modalType.value = 'create'
  showModal.value = true
}

const handleEdit = (row) => {
  currentItem.value = {...row}
  modalType.value = 'edit'
  showModal.value = true
}

const handleSave = async () => {
  isSubmitting.value = true
  try {
    if (modalType.value === 'create') {
      await roleStore.createRole(currentItem.value)
      $message.success(t('common.messages.create_success'))
    } else {
      await roleStore.updateRole(currentItem.value.uuid, currentItem.value)
      $message.success(t('common.messages.update_success'))
    }
    showModal.value = false
  } catch (error) {
    $message.error(`${t('common.messages.save_failed')}: ${error.message}`)
  } finally {
    isSubmitting.value = false
  }
}

const handleDelete = async (row) => {
  try {
    await roleStore.deleteRole(row.uuid)
    $message.success(t('common.messages.delete_success'))
  } catch (error) {
    $message.error(`${t('common.messages.delete_failed')}: ${error.message}`)
  }
}

const columns = [
  {
    title: t('views.system.role.name'),
    key: 'name',
    width: 80,
    align: 'center',
    ellipsis: {tooltip: true},
    render: (row) => h(NTag, {type: 'info'}, {default: () => row.name}),
  },
  {
    title: t('views.system.role.description'),
    key: 'description',
    width: 80,
    align: 'center',
  },
  {
    title: t('common.fields.created_date'),
    key: 'created_at',
    width: 60,
    align: 'center',
    render: (row) => h('span', formatDate(row.created_at)),
  },
  {
    title: t('common.fields.operate'),
    key: 'actions',
    width: 180,
    align: 'center',
    fixed: 'right',
    render(row) {
      return [
        withDirectives(
            h(NButton, {
                  size: 'small',
                  type: 'primary',
                  style: 'margin-right: 8px;',
                  onClick: () => handleEdit(row),
                },
                {default: () => t('common.buttons.edit'), icon: renderIcon('material-symbols:edit-outline', {size: 16})}),
            [[vPermission, 'button:role_update']]
        ),
        h(NPopconfirm,
            {onPositiveClick: () => handleDelete(row)},
            {
              trigger: () => withDirectives(
                  h(NButton, {
                        size: 'small',
                        type: 'error',
                      },
                      {default: () => t('common.buttons.delete'), icon: renderIcon('material-symbols:delete-outline', {size: 16})}),
                  [[vPermission, 'button:role_delete']]
              ),
              default: () => t('views.system.role.confirm_delete'),
            }),
      ]
    },
  },
]
</script>

<template>
  <CommonPage show-footer :title="t('views.system.role.list')">
    <template #action>
      <NButton v-permission="'button:role_create'" type="primary" @click="handleAdd">
        <TheIcon icon="material-symbols:add" :size="18" class="mr-5"/>
        {{ t('views.system.role.new') }}
      </NButton>
    </template>

    <!-- CrudTable -->
    <CrudTable
        :columns="columns"
        :data="roles"
        :loading="isTableLoading"
        :pagination="pagination"
        @update:page="handlePageChange"
        @update:page-size="handlePageSizeChange"
        @reset="handleResetSearch"
        @search="handleSearch"
    >
      <template #queryBar>
        <QueryBarItem :label="t('views.system.role.name')" :label-width="50">
          <NInput
              v-model:value="queryParams.role_name"
              clearable
              type="text"
              :placeholder="t('views.system.role.enter_name')"
              @keypress.enter="handleSearch"
          />
        </QueryBarItem>
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
      >
        <NFormItem
            :label="t('views.system.role.name')"
            path="name"
            :rule="{ required: true, message: t('views.system.role.enter_name_required'), trigger: ['input', 'blur'] }"
        >
          <NInput v-model:value="currentItem.name"/>
        </NFormItem>
        <NFormItem :label="t('views.system.role.description')" path="description">
          <NInput v-model:value="currentItem.description"/>
        </NFormItem>
      </NForm>
    </CrudModal>
  </CommonPage>
</template>