import { resolve } from 'path'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { NaiveUiResolver } from 'unplugin-vue-components/resolvers'
import { FileSystemIconLoader } from 'unplugin-icons/loaders'
import IconsResolver from 'unplugin-icons/resolver'

/**
 * * unplugin-icons plugin, auto-imports iconify icons
 * usage: https://github.com/antfu/unplugin-icons
 * Icon sets: https://icones.js.org/
 */
import Icons from 'unplugin-icons/vite'
import { createSvgIconsPlugin } from 'vite-plugin-svg-icons'

import { getSrcPath } from '../utils'

const customIconPath = resolve(getSrcPath(), 'assets/svg')

export default [
  AutoImport({
    imports: ['vue', 'vue-router'],
    dts: false,
  }),
  Icons({
    compiler: 'vue3',
    customCollections: {
      custom: FileSystemIconLoader(customIconPath),
    },
    scale: 1,
    defaultClass: 'inline-block',
  }),
  Components({
    resolvers: [
      NaiveUiResolver(),
      IconsResolver({ customCollections: ['custom'], componentPrefix: 'icon' }),
    ],
    dts: false,
  }),
  createSvgIconsPlugin({
    iconDirs: [customIconPath],
    symbolId: 'icon-custom-[dir]-[name]',
    inject: 'body-last',
    customDomId: '__CUSTOM_SVG_ICON__',
  }),
]
