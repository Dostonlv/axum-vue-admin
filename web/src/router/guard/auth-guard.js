import {hasPermission} from "@/directives/permission";
import {useUserStore} from "@/stores";
import {addDynamicRoutes} from "@/router";

const WHITE_LIST = ['/login', '/404', '/forgot-password', /^\/reset-password\/.*/]
export function createAuthGuard(router) {
  router.beforeEach(async (to) => {
    const userStore = useUserStore();
    /** No token */
    if (!userStore.accessToken) {
      // Is the target path whitelisted?
      const isInWhiteList = WHITE_LIST.some((pattern) => {
        if (typeof pattern === 'string') {
          return pattern === to.path // plain path
        }
        return pattern.test(to.path) // regex
      })
      if (isInWhiteList) return true
      return { path: 'login', query: { ...to.query, redirect: to.path } }
    }

    if (!userStore.isDynamicRoutesAdded) { // flag prevents adding twice
      addDynamicRoutes();

      userStore.setDynamicRoutesAdded(true); // set the flag
      return { ...to, replace: true }; // navigate again so the new routes apply
    }


    /** Has token */
    if (to.path === '/login') return { path: '/' }

    // Route permission check
    // 1. Resource identifier of the target route
    const resource = to.meta.resource;

    // 2. No resource identifier means a public page (e.g. 403), let it through
    if (!resource) {
      return true;
    }

    // 3. Check whether the user may access the resource
    if (hasPermission(resource)) {
      // Allowed
      return true;
    } else {
      // Not allowed, redirect to 403
      console.warn(`Access denied for resource: ${resource}`);
      return {path: "403"};
    }
  })
}
