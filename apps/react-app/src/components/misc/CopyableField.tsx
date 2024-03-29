import { CopyOutlined } from '@ant-design/icons';
import { copyToClipboard } from '@utils/ui';
import { useState } from 'react';
import { formatPubkey } from '@utils/format';
import { Typography } from 'antd';

interface CopyableField {
  content: string;
}

export const CopyableField = ({ content }: CopyableField) => {
  const { Text } = Typography;

  const [copied, setCopied] = useState(false);
  const onClick = () => {
    copyToClipboard(content);
    setCopied(true);
    setTimeout(() => {
      setCopied(false);
    }, 1000);
  };
  return (
    <div className={`copiable-field ${copied ? 'copied' : ''}`} onClick={onClick}>
      <CopyOutlined />
      <Text>{formatPubkey(content)}</Text>
    </div>
  );
};
