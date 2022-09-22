import { notification } from 'antd';
import { openLinkInBrowser } from './ui';

// Configuration and implementation for app notifications
export const NOTIFICATION_DURATION = 7.5;
export const NOTIFICATION_PLACEMENT = 'bottomLeft';
export function notify(
  message: string,
  description: string,
  type?: 'success' | 'warning' | 'error',
  explorerLink?: string
) {
  if (type === 'success') {
    notification.success({
      message,
      description,
      duration: NOTIFICATION_DURATION,
      placement: NOTIFICATION_PLACEMENT,
      onClick: explorerLink ? () => openLinkInBrowser(explorerLink) : undefined
    });
  } else if (type === 'warning') {
    notification.warning({
      message,
      description,
      duration: NOTIFICATION_DURATION,
      placement: NOTIFICATION_PLACEMENT,
      onClick: explorerLink ? () => openLinkInBrowser(explorerLink) : undefined
    });
  } else if (type === 'error') {
    notification.error({
      message,
      description,
      duration: NOTIFICATION_DURATION,
      placement: NOTIFICATION_PLACEMENT,
      onClick: explorerLink ? () => openLinkInBrowser(explorerLink) : undefined
    });
  } else {
    notification.open({
      message,
      description,
      duration: NOTIFICATION_DURATION,
      placement: NOTIFICATION_PLACEMENT,
      onClick: explorerLink ? () => openLinkInBrowser(explorerLink) : undefined
    });
  }
}
