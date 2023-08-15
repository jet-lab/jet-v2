import { Alert } from 'antd';
import { ReactNode } from 'react';

export function Banner(props: { message: string | ReactNode }): JSX.Element {
  return <Alert closable className="tps-banner" type="success" message={props.message} />;
}
