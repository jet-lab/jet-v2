import { useRecoilValue, useResetRecoilState } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { NotificationsModal as NotificationModalState } from '@state/modals/modals';
import { Modal, Typography } from 'antd';

// Modal for Dialect notifications (TODO: add Dialect here)
export function NotificationsModal() {
  const dictionary = useRecoilValue(Dictionary);
  const notificationsModalOpen = useRecoilValue(NotificationModalState);
  const resetNotificationsModalOpen = useResetRecoilState(NotificationModalState);
  const { Title } = Typography;

  if (notificationsModalOpen) {
    return (
      <Modal
        open
        className="header-modal notifications-modal"
        maskClosable={false}
        onCancel={resetNotificationsModalOpen}>
        <div className="modal-header flex-centered">
          <Title className="modal-header-title green-text">{dictionary.notificationsModal.title}</Title>
        </div>
      </Modal>
    );
  } else {
    return <></>;
  }
}
