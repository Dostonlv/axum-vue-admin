import dayjs from 'dayjs'

/**
 * @desc  Format a date
 * @param {(Object|string|number)} time
 * @param {string} format
 * @returns {string | null}
 */
export function formatDateTime(time = undefined, format = 'YYYY-MM-DD HH:mm:ss') {
  return dayjs(time).format(format)
}

export function formatDate(date = undefined, format = 'YYYY-MM-DD') {
  return formatDateTime(date, format)
}

/**
 * @desc  Throttle
 * @param {Function} fn
 * @param {Number} wait
 * @returns {Function}
 */
export function throttle(fn, wait) {
  var context, args
  var previous = 0

  return function () {
    var now = +new Date()
    context = this
    args = arguments
    if (now - previous > wait) {
      fn.apply(context, args)
      previous = now
    }
  }
}

/**
 * @desc  Debounce
 * @param {Function} func
 * @param {number} wait
 * @param {boolean} immediate
 * @return {*}
 */
export function debounce(method, wait, immediate) {
  let timeout
  return function (...args) {
    let context = this
    if (timeout) {
      clearTimeout(timeout)
    }
    // Running immediately needs two things: immediate is true, and timeout is unset or null
    if (immediate) {
      /**
       * If there is no timer, run now and start one that resets to null after `wait` ms
       * so it can't fire again within `wait` ms of an immediate run
       */
      let callNow = !timeout
      timeout = setTimeout(() => {
        timeout = null
      }, wait)
      if (callNow) {
        method.apply(context, args)
      }
    } else {
      // If immediate is false, run after `wait` ms
      timeout = setTimeout(() => {
        /**
         * args is array-like, hence fn.apply
         * method.call(context, ...args) works too
         */
        method.apply(context, args)
      }, wait)
    }
  }
}
