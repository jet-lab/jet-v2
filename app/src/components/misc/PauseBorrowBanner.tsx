import { Alert } from 'antd';

// Banner to show user that borrows are temporarily paused
export function PauseBorrowBanner(): JSX.Element {
  function getMessage() {
    let message =
      'Due to recent market conditions surrounding several listed assets related to FTX on Jet Protocol, borrows are temporarily paused for user safety.';
    return message;
  }

  return <Alert closable className="tps-banner" type={'error'} message={getMessage()} />;
}
