<script setup>
import {h, onMounted, ref, reactive, resolveDirective, withDirectives, computed} from 'vue';
import {NButton, NForm, NFormItem, NInput, NInputNumber, NPopconfirm, NTreeSelect} from 'naive-ui';
import {storeToRefs} from 'pinia';
import {useDepartmentStore} from '@/stores';
import {renderIcon} from '@/utils';
import CommonPage from '@/components/page/CommonPage.vue';
import QueryBarItem from '@/components/query-bar/QueryBarItem.vue';
import CrudModal from '@/components/table/CrudModal.vue';
import CrudTable from '@/components/table/CrudTable.vue';
import TheIcon from '@/components/icon/TheIcon.vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n({ useScope: 'global' });

const $message = window.$message;

defineOptions({name: 'DeptManagement'});


const store = useDepartmentStore();
const {departments, isLoading, departmentOptions} = storeToRefs(store);
const vPermission = resolveDirective('permission');


const showModal = ref(false);
const modalType = ref('create');
const modalTitle = computed(() => modalType.value === 'create' ? t('views.system.dept.new') : t('views.system.dept.edit'));
const currentItem = ref({}); // form data
const isSubmitting = ref(false);
const isParentDeptDisabled = ref(false); // disables the parent department picker

const queryParams = reactive({});


const loadData = () => {
  store.fetchDepartments(queryParams)
      .catch(error => $message.error(`${t('views.system.dept.load_failed')}: ${error.message}`));
};

onMounted(() => {
      loadData()
    }
);


const handleSearch = () => loadData();

const handleResetSearch = () => {
  for (const key in queryParams) {
    delete queryParams[key]
  }
  handleSearch()
}

const handleAdd = () => {
  currentItem.value = {order: 0};
  modalType.value = 'create';
  isParentDeptDisabled.value = false;
  showModal.value = true;
};

const handleEdit = (row) => {
  currentItem.value = {...row};
  modalType.value = 'edit';
  isParentDeptDisabled.value = !row.parent_id;
  showModal.value = true;
};

const handleSave = async () => {
  isSubmitting.value = true;
  try {
    if (modalType.value === 'create') {
      await store.createDepartment(currentItem.value);
      $message.success(t('common.messages.create_success'));
    } else {
      await store.updateDepartment(currentItem.value.uuid, currentItem.value);
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
    await store.deleteDepartment(row.uuid);
  } catch (error) {
    console.log(error);
  }
};


const deptRules = {
  name: [{required: true, message: t('views.system.dept.enter_name'), trigger: ['input', 'blur']}],
};

const columns = [
  {
    title: t('views.system.dept.name'),
    key: 'name',
    align: 'center',
  },
  {
    title: t('common.fields.remark'),
    key: 'desc',
    align: 'center',
  },
  {
    title: t('common.fields.operate'),
    key: 'actions',
    align: 'center',
    render(row) {
      return [
        withDirectives(
            h(NButton, {size: 'small', type: 'primary', style: 'margin-left: 8px;', onClick: () => handleEdit(row)},
                {default: () => t('common.buttons.edit'), icon: renderIcon('material-symbols:edit', {size: 16})}
            ),
            [[vPermission, 'button:dept_update']]
        ),
        withDirectives(
            h(NPopconfirm, {onPositiveClick: () => handleDelete(row)},
                {
                  trigger: () => h(
                      NButton,
                      {
                        size: 'small',
                        type: 'error',
                        style: 'margin-left: 8px;',
                      },
                      {
                        default: () => t('common.buttons.delete'),
                        icon: renderIcon('material-symbols:delete-outline', {size: 16}),
                      }
                  ),
                  default: () => t('views.system.dept.confirm_delete'),
                }
            ),
            [[vPermission, 'button:dept_delete']]
        ),

      ];
    },
  },
];
</script>

<template>
  <CommonPage show-footer :title="t('views.system.dept.list')">
    <template #action>
      <NButton v-permission="'button:dept_create'" type="primary" @click="handleAdd">
        <TheIcon icon="material-symbols:add" :size="18" class="mr-5"/>
        {{ t('views.system.dept.new') }}
      </NButton>
    </template>

    <CrudTable
        :columns="columns"
        :data="departments"
        :loading="isLoading"
        row-key="uuid"
        :pagination="false"
        @search="handleSearch"
        @reset="handleResetSearch"
    >
      <template #queryBar>
        <QueryBarItem :label="t('views.system.dept.name')" :label-width="80">
          <NInput v-model:value="queryParams.name" @keypress.enter="handleSearch"/>
        </QueryBarItem>
      </template>
    </CrudTable>

    <CrudModal
        v-model:visible="showModal"
        :title="modalTitle"
        :loading="isSubmitting"
        @save="handleSave"
    >
      <NForm :model="currentItem" :rules="deptRules" label-placement="left" label-width="auto">
        <NFormItem :label="t('views.system.dept.parent')" path="parent_id" required>
          <NTreeSelect
              v-model:value="currentItem.parent_uuid"
              :options="departmentOptions"
              key-field="id"
              label-field="name"
              :placeholder="t('views.system.dept.select_parent')"
              clearable
              default-expand-all
              :disabled="isParentDeptDisabled"
          />
        </NFormItem>
        <NFormItem :label="t('views.system.dept.name')" path="name">
          <NInput v-model:value="currentItem.name"/>
        </NFormItem>
        <NFormItem :label="t('common.fields.remark')" path="desc">
          <NInput v-model:value="currentItem.desc"/>
        </NFormItem>
        <NFormItem :label="t('common.fields.order')" path="order">
          <NInputNumber v-model:value="currentItem.order" min="0"/>
        </NFormItem>
      </NForm>
    </CrudModal>
  </CommonPage>
</template>