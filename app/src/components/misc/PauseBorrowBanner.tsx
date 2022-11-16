import { Alert } from 'antd';

// Banner to show user that borrows are temporarily paused
export function PauseBorrowBanner(): JSX.Element {
  function getMessage() {
    let message = 'Borrows have been re-enabled, thank you for your patience';
    return message;
  }

  return <Alert closable className="tps-banner" type={'success'} message={getMessage()} />;
}
