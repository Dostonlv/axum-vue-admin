import dayjs from 'dayjs'

/**
 * * Global constants defined here are added to window on start or build
 * https://vitejs.cn/config/#define
 */

// Build time
const _BUILD_TIME_ = JSON.stringify(dayjs().format('YYYY-MM-DD HH:mm:ss'))

export const viteDefine = {
  _BUILD_TIME_,
}
