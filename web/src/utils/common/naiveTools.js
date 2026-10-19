import { isNullOrUndef } from '@/utils'

export function setupMessage(NMessage) {
  let loadingMessage = null
  class Message {
    /**
     * Rules:
     * * only one loading message is shown; a new message replaces it
     * * a loading message is never cleared automatically, only replaced; other messages clear after 2s
     */

    removeMessage(message = loadingMessage, duration = 2000) {
      setTimeout(() => {
        if (message) {
          message.destroy()
          message = null
        }
      }, duration)
    }

    showMessage(type, content, option = {}) {
      if (loadingMessage && loadingMessage.type === 'loading') {
        // Replace the loading message if one is showing
        loadingMessage.type = type
        loadingMessage.content = content

        if (type !== 'loading') {
          // Non-loading messages must be cleared automatically
          this.removeMessage(loadingMessage, option.duration)
        }
      } else {
        // No loading message showing: create one, and keep a reference if the new message is a loading message
        let message = NMessage[type](content, option)
        if (type === 'loading') {
          loadingMessage = message
        }
      }
    }

    loading(content) {
      this.showMessage('loading', content, { duration: 0 })
    }

    success(content, option = {}) {
      this.showMessage('success', content, option)
    }

    error(content, option = {}) {
      this.showMessage('error', content, option)
    }

    info(content, option = {}) {
      this.showMessage('info', content, option)
    }

    warning(content, option = {}) {
      this.showMessage('warning', content, option)
    }
  }

  return new Message()
}

export function setupDialog(NDialog, t) {
  NDialog.confirm = function (option = {}) {
    const showIcon = !isNullOrUndef(option.title)
    return NDialog[option.type || 'warning']({
      showIcon,
      positiveText: t('common.buttons.confirm'),
      negativeText: t('common.buttons.cancel'),
      onPositiveClick: option.confirm,
      onNegativeClick: option.cancel,
      onMaskClick: option.cancel,
      ...option,
    })
  }

  return NDialog
}
