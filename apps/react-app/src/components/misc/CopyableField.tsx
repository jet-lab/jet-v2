import { CopyOutlined } from '@ant-design/icons';
import { copyToClipboard } from '@utils/ui';
import { useState } from 'react';
import { formatPubkey } from '@utils/format';
import { Typography } from 'antd';


interface CopyableField {
    content: string
}


export const CopyableField = ({ content }: CopyableField) => {
    const { Text } = Typography;

    const [copied, setCopied] = useState(false)
    const onClick = () => {
        copyToClipboard(content)
        setCopied(true)
        setTimeout(() => {
            setCopied(false)
        }, 1000)
    }
    return <div className={`pool-detail-body-half-section flex align-start justify-center column`}>
        <div className={`pool-details-address ${copied ? 'copied' : ''}`} onClick={onClick}>
            <CopyOutlined />
            <Text>{formatPubkey(content)}</Text>
        </div>
    </div>
}