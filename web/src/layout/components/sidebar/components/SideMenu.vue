<template>
  <n-menu
      ref="menu"
      class="side-menu"
      accordion
      :indent="18"
      :collapsed-icon-size="22"
      :collapsed-width="64"
      :options="menuOptions"
      :value="activeKey"
      @update:value="handleMenuSelect"
  />
</template>

<script setup>
import {useAppStore} from '@/stores'
import {renderCustomIcon, renderIcon, isExternal} from '@/utils'
import {useRoute, useRouter} from "vue-router";

const router = useRouter()
const curRoute = useRoute()
const appStore = useAppStore()

const activeKey = computed(() => curRoute.meta?.activeMenu || curRoute.name)

const menuOptions = computed(() => {
  // All defined routes
  const allRoutes = router.getRoutes();
  const routeTree = buildRouteTree(allRoutes)
  const filterRoutes = routeTree.filter(route => {
    return !route.meta?.isHidden;
  });
  return filterRoutes.map((route) => getMenuItem(route)).sort((a, b) => a.order - b.order);
});

const menu = ref(null)
watch(curRoute, async () => {
  await nextTick()
  menu.value?.showOption()
})

function getMenuItem(route) {

  const menuItem = {
    label: (route.meta && route.meta.title) || route.name,
    key: route.name,
    path: route.path,
    icon: getIcon(route.meta),
    order: route.meta?.order || 0,
  }

  const visibleChildren = route.children
      ? route.children.filter((item) => item.name && !item.meta?.isHidden)
      : []

  if (!visibleChildren.length) {
    return menuItem
  }

  // A single child route gets promoted to the parent
  if (visibleChildren.length === 1) {
    const singleRoute = visibleChildren[0]
    menuItem.label = singleRoute.meta?.title || singleRoute.name
    menuItem.key = singleRoute.name
    menuItem.path = singleRoute.path
    menuItem.icon = getIcon(singleRoute.meta)

    // If that only child has children of its own, they become the submenu
    if (singleRoute.children && singleRoute.children.length > 0) {
      menuItem.children = singleRoute.children
          .map((item) => getMenuItem(item))
          .sort((a, b) => a.order - b.order)
    }
  } else {
    // Several child routes
    menuItem.children = visibleChildren
        .map((item) => getMenuItem(item))
        .sort((a, b) => a.order - b.order)
  }

  return menuItem
}

function getIcon(meta) {
  if (meta?.customIcon) return renderCustomIcon(meta.customIcon, {size: 18})
  if (meta?.icon) return renderIcon(meta.icon, {size: 18})
  return null
}

function handleMenuSelect(key, item) {
  if (isExternal(item.path)) {
    window.open(item.path)
  } else {
    if (item.path === curRoute.path) {
      appStore.reloadPage()
    } else {
      router.push(item.path)
    }
  }
}

function buildRouteTree(routes) {
  const routeMap = new Map();
  const topLevelRoutes = [];

  for (const route of routes) {
    routeMap.set(route.path, {...route, children: []});
  }

  for (const route of routes) {
    // Find the parent route. A child path is usually the parent path plus a suffix
    // Note: we need a reliable way to find the parent; the route name works too if names are consistent
    const segments = route.path.split('/').filter(Boolean);

    if (segments.length > 1) {
      // Try to find the parent path
      const parentPath = '/' + segments.slice(0, -1).join('/');
      const parentNode = routeMap.get(parentPath);

      if (parentNode) {
        // Parent found, add this route to its children
        parentNode.children.push(routeMap.get(route.path));
      }
    } else {
      // Root or first-level path, treat as a top-level route
      topLevelRoutes.push(routeMap.get(route.path));
    }
  }
  return topLevelRoutes;
}
</script>

<style lang="scss">
.side-menu:not(.n-menu--collapsed) {
  .n-menu-item-content {
    &::before {
      left: 5px;
      right: 5px;
    }

    &.n-menu-item-content--selected,
    &:hover {
      &::before {
        border-left: 4px solid var(--primary-color);
      }
    }
  }
}
</style>
