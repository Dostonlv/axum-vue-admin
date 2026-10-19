import i18n from '@/i18n'

const {t} = i18n.global

const Layout = () => import('@/layout/index.vue')

/*
Routes should not be configured by the backend, it causes:
1. tight coupling between frontend and backend
2. a poor frontend dev experience
3. complicated backend logic

Do this instead:
1. the frontend defines the full route table
2. mark each route's resource in meta
3. the backend provides the data:
    UI resource policies (uiPolicies)
4. the frontend filters and guards routes


icons: https://icones.js.org/
*/
export const basicRoutes = [
    {
        name: 'index',
        path: '/',
        redirect: '/workbench', // default landing page
        meta: {
            order: 0,
            isHidden: true
        },
    },
    {
        name: t('views.workbench.label_workbench'),
        path: '/workbench',
        component: Layout,
        children: [
            {
                path: '',
                component: () => import('@/views/workbench/index.vue'),
                name: t('views.workbench.label_workbench'),
                meta: {
                    title: t('views.workbench.label_workbench'),
                    icon: 'icon-park-outline:workbench',
                    affix: true,
                },
            },
        ],
        meta: {order: 1},
    },
    {
        path: '/login',
        name: 'Login',
        component: () => import('@/views/login/index.vue'),
        meta: {isHidden: true},
    },
    {
        name: 'ForGotPassword',
        path: '/forgot-password',
        component: () => import('@/views/forgot-password/index.vue'),
        meta: {
            isHidden: true,
            title: t('views.forget_password.forget_password'),
        },
    },
    {
        name: 'RestPassword',
        path: '/reset-password/:resetToken',
        component: () => import('@/views/reset-password/index.vue'),
        isHidden: true,
        props: true,
        meta: {
            isHidden: true,
            title: t('views.resetPassword.resetPassword'),
        },
    },
    {
        path: '/error-page',
        name: 'ErrorPage',
        component: Layout,
        redirect: '/error-page/404',
        meta: {isHidden: true},
        children: [
            {
                path: '401',
                name: 'ERROR-401',
                component: () => import('@/views/error-page/401.vue'),
                meta: {
                    title: '401',
                    icon: 'material-symbols:authenticator',
                },
            },
            {
                path: '403',
                name: 'ERROR-403',
                component: () => import('@/views/error-page/403.vue'),
                meta: {
                    title: '403',
                    icon: 'solar:forbidden-circle-line-duotone',
                },
            },
            {
                path: '404',
                name: 'ERROR-404',
                component: () => import('@/views/error-page/404.vue'),
                meta: {
                    title: '404',
                    icon: 'tabler:error-404',
                },
            },
            {
                path: '500',
                name: 'ERROR-500',
                component: () => import('@/views/error-page/500.vue'),
                meta: {
                    title: '500',
                    icon: 'clarity:rack-server-outline-alerted',
                },
            },
        ],
    },
    // The 404 route must come last
    {
        path: '/:pathMatch(.*)*',
        name: 'NotFound',
        redirect: '/error-page/404',
        meta: {isHidden: true},
    },
]

// Routes that need permission filtering go here
export const asyncRoutes = [
    {
        name: "SystemManagement",
        path: "/system",
        component: Layout,
        "redirect": "user",
        meta: {
            title: t('menu.system'),
            icon: "carbon:gui-management",
            order: 2,
            keepAlive: true,
            isHidden: false,
        },
        children: [
            {
                name: "UserManagement",
                path: "user",
                component: () => import('@/views/system/user/index.vue'),
                meta: {
                    title: t('menu.user'),
                    icon: "material-symbols:person-outline-rounded",
                    order: 1,
                    keepAlive: true,
                    isHidden: false,
                    resource: "menus:user_management",
                }
            },
            {
                name: "GroupManagement",
                path: "group",
                component: () => import('@/views/system/group/index.vue'),
                meta: {
                    title: t('menu.group'),
                    icon: "mdi:account-group-outline",
                    order: 2,
                    keepAlive: true,
                    isHidden: false,
                    resource: "menus:group_management",
                }
            },
            {
                name: "RoleManagement",
                path: "role",
                component: () => import('@/views/system/role/index.vue'),
                meta: {
                    title: t('menu.role'),
                    icon: "carbon:user-role",
                    order: 3,
                    keepAlive: true,
                    isHidden: false,
                    resource: "menus:role_management",
                }
            },
            {
                name: "DeptManagement",
                path: "dept",
                component: () => import('@/views/system/dept/index.vue'),
                meta: {
                    title: t('menu.dept'),
                    icon: "mingcute:department-line",
                    order: 4,
                    keepAlive: true,
                    isHidden: false,
                    resource: "menus:dept_management",
                }
            },
            {
                name: "CedarPolicyManagement",
                path: "cedar_policies",
                component: () => import('@/views/system/cedar_policies/index.vue'),
                meta: {
                    title: t('menu.cedar_policy'),
                    icon: "carbon:policy",
                    order: 5,
                    keepAlive: true,
                    isHidden: false,
                    resource: "menus:policies_management",
                }
            },
            {
                name: "CedarSchemaManagement",
                path: "cedar_schema",
                component: () => import('@/views/system/cedar_schema/index.vue'),
                meta: {
                    title: t('menu.cedar_schema'),
                    icon: "carbon:schematics",
                    order: 6,
                    keepAlive: true,
                    isHidden: false,
                    resource: "menus:policies_management",
                }
            },
            // {
            //     name: "Audit log",
            //     path: "auditlog",
            //     component: () => import('@/views/system/auditlog/index.vue'),
            //     meta: {
            //         title: "Audit log",
            //         icon: "ph:clipboard-text-bold",
            //         order: 7,
            //         keepAlive: true,
            //         isHidden: false,
            //         resource: "menus:auditlog_management",
            //     }
            // }
        ]
    },
    {
        name: t('views.profile.label_profile'),
        path: '/profile',
        component: Layout,
        children: [
            {
                path: '',
                component: () => import('@/views/profile/index.vue'),
                name: `${t('views.profile.label_profile')}Default`,
                meta: {
                    title: t('views.profile.label_profile'),
                    icon: 'user',
                    affix: true,
                },
            },
        ],
        meta: {
            order: 99,
            isHidden: true,
        },
    },
]

export const NOT_FOUND_ROUTE = {
    name: 'NotFound',
    path: '/:pathMatch(.*)*',
    redirect: '/404',
    isHidden: true,
}

export const EMPTY_ROUTE = {
    name: 'Empty',
    path: '/:pathMatch(.*)*',
    component: null,
}

// const modules = import.meta.glob('@/views/**/route.js', { eager: true })
// Object.keys(modules).forEach((key) => {
//   asyncRoutes.push(modules[key].default)
// })
