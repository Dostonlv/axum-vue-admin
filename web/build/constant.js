export const OUTPUT_DIR = 'dist'

export const PROXY_CONFIG = {
  // /**
  //  * @desc    replaces the matched prefix
  //  * @request   http://localhost:3100/api/user
  //  * @forward   http://localhost:9999/api/v1 +/user
  //  */
  // '/api': {
  //   target: 'http://localhost:9999/api/v1',
  //   changeOrigin: true,
  //   rewrite: (path) => path.replace(new RegExp('^/api'), ''),
  // },
  /**
   * @desc    keeps the matched prefix
   * @request   http://localhost:3100/api/v1/user
   * @forward   http://localhost:9999/api/v1/user
   */
  '/api/v1': {
    target: 'http://127.0.0.1:9999',
    changeOrigin: true,
  },
}
