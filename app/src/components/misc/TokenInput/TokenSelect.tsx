import { useRecoilValue, useRecoilState } from 'recoil';
import { CurrentPoolSymbol, PoolOptions } from '../../../state/pools/pools';
import { TokenLogo } from '../TokenLogo';
import { Select, Typography } from 'antd';
import { ReactComponent as AngleDown } from '../../../styles/icons/arrow-angle-down.svg';

// Select component for the Token Input (to change which token user is interacting with)
export function TokenSelect(props: {
  // Optionally, specify currently active Pool for this dropdown
  poolSymbol?: string | undefined;
  // Optionally, specify what occurs when the user switches their token
  onChangeToken?: (token: string) => unknown;
  // Optionally, override the styles of the token selector dropdown
  dropdownStyle?: React.CSSProperties;
}): JSX.Element {
  const [currentPoolSymbol, setCurrentPoolSymbol] = useRecoilState(CurrentPoolSymbol);
  const poolOptions = useRecoilValue(PoolOptions);
  const { Paragraph, Text } = Typography;
  const { Option } = Select;

  return (
    <Select
      dropdownClassName="token-input-dropdown dropdown-space-between"
      dropdownStyle={props.dropdownStyle}
      value={props.poolSymbol ? props.poolSymbol : currentPoolSymbol}
      onChange={tokenSymbol => {
        // If there is a specified action on a token change
        if (props.onChangeToken) {
          props.onChangeToken(tokenSymbol);
          return;
        }

        // Default to updating the currentPool
        setCurrentPoolSymbol(tokenSymbol);
      }}>
      {poolOptions.map(option => (
        <Option key={option.symbol} value={option.symbol}>
          <div className="flex-centered">
            <TokenLogo height={20} symbol={option.symbol} />
            <Paragraph className="token-symbol">{option.symbol}</Paragraph>
            <AngleDown className="jet-icon" />
          </div>
          <Text type="secondary">{option.name}</Text>
        </Option>
      ))}
    </Select>
  );
}
