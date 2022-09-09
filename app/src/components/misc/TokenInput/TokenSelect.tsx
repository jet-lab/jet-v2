import { useRecoilValue } from 'recoil';
import { Pool } from '@jet-lab/margin';
import { PoolOptions } from '../../../state/borrow/pools';
import { TokenLogo } from '../TokenLogo';
import { Select, Typography } from 'antd';

// Select component for the Token Input (to change which token user is interacting with)
export function TokenSelect(props: {
  // The currently active Pool
  tokenPool: Pool | undefined;
  // Specify what occurs when the user switches their token
  onChangeToken: (token: string) => unknown;
  // Optionally, override the styles of the token selector dropdown
  dropdownStyle?: React.CSSProperties;
}): JSX.Element {
  const poolOptions = useRecoilValue(PoolOptions);
  const { Paragraph, Text } = Typography;
  const { Option } = Select;

  return (
    <Select
      dropdownClassName="token-input-dropdown dropdown-space-between"
      dropdownStyle={props.dropdownStyle}
      value={props.tokenPool ? props.tokenPool.symbol : undefined}
      onChange={tokenSymbol => props.onChangeToken(tokenSymbol)}>
      {poolOptions.map(option => (
        <Option key={option.symbol} value={option.symbol}>
          <div className="flex-centered">
            <TokenLogo height={20} symbol={option.symbol} />
            <Paragraph>{option.symbol}</Paragraph>
          </div>
          <Text type="secondary">{option.name}</Text>
        </Option>
      ))}
    </Select>
  );
}
