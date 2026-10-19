import { createRouter, createWebHistory, createWebHashHistory } from 'vue-router'
import { setupRouterGuard } from './guard'
import {asyncRoutes, basicRoutes, EMPTY_ROUTE} from './routes'
import {hasPermission} from "@/directives/permission";
import {useUserStore} from "@/stores";

const isHash = import.meta.env.VITE_USE_HASH === 'true'



export const router = createRouter({
  history: isHash ? createWebHashHistory('/') : createWebHistory('/'),
  routes: basicRoutes,
  scrollBehavior: () => ({ left: 0, top: 0 }),
})

export async function setupRouter(app) {
  await addDynamicRoutes()
  setupRouterGuard(router)
  app.use(router)
}

export async function resetRouter() {
  const basicRouteNames = getRouteNames(basicRoutes)
  router.getRoutes().forEach((route) => {
    const name = route.name
    if (!basicRouteNames.includes(name)) {
      router.removeRoute(name)
    }
  })
}

export function addDynamicRoutes() {
  const userStore = useUserStore()
  const uiPolicies = userStore.uiPolicies || []

  // No uiPolicies
  if (!uiPolicies) {
    router.addRoute(EMPTY_ROUTE)
    return
  }
  // With uiPolicies
  console.log("uiPolicies:", uiPolicies)
  let accessibleRoutes = filterAccessibleRoutes(asyncRoutes, uiPolicies)
  accessibleRoutes.forEach((route) => {
    router.addRoute(route)
  })
  console.log(router.getRoutes())
}

export function getRouteNames(routes) {
  return routes.map((route) => getRouteName(route)).flat(1)
}

function getRouteName(route) {
  const names = [route.name]
  if (route.children && route.children.length) {
    names.push(...route.children.map((item) => getRouteName(item)).flat(1))
  }
  return names
}


function filterAccessibleRoutes(routesToFilter, uiPolicies) { //
  const accessibleRoutes = [];

  routesToFilter.forEach(route => {
    const tempRoute = { ...route };

    let hasVisibleChildren = false;
    // Filter child routes recursively
    if (tempRoute.children) {
      tempRoute.children = filterAccessibleRoutes(tempRoute.children, uiPolicies);
      if (tempRoute.children.length > 0) {
        hasVisibleChildren = true;
      }
    }

    // Decide whether to keep this route
    const resource = tempRoute.meta?.resource;
    // 1: it has visible children (so it is a parent menu)
    // 2: the user may access it directly (leaf menu items, or a parent that is also a page)
    const hasAccess = !resource || uiPolicies.includes(resource);
    console.log("resource: ", resource, hasAccess);
    if (hasVisibleChildren || hasAccess) {
      // No children left, drop the property
      if (!hasVisibleChildren) {
        delete tempRoute.children;
      }
      accessibleRoutes.push(tempRoute);
    }
  });
  return accessibleRoutes;
}