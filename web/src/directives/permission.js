import { useUserStore } from '@/stores'

/**
 * Check whether the current user may access a UI resource
 * @param {string} resourceIdentifier - unique resource identifier, e.g., 'menus:dashboard'
 * @returns {boolean}
 */
export function hasPermission(resourceIdentifier) {
  const user = useUserStore()

  const uiPolicies = user.uiPolicies || []
  // 1. Highest priority: boolean flag
  if (user && user.isSuperUser) {
    return true;
  }

  // Not a super admin, so run the regular permission check
  if (!user || !user.uiPolicies) {
    return false;
  }

  // 5. Check whether the user holds the permission
  return uiPolicies.includes(resourceIdentifier);
}

export default function setupPermissionDirective(app) {
  function updateElVisible(el, resourceIdentifier) {
    if (!resourceIdentifier) {
      throw new Error(`need permissions: like v-permission="['user:create', 'user:update']"`)
    }
    if (!hasPermission(resourceIdentifier)) {
      el.parentElement?.removeChild(el)
    }
  }

  const permissionDirective = {
    mounted(el, binding) {
      updateElVisible(el, binding.value)
    },
    beforeUpdate(el, binding) {
      updateElVisible(el, binding.value)
    },
  }

  app.directive('permission', permissionDirective)
}
