import { useRecoilValue, useRecoilState } from 'recoil';
import { CurrentPoolTokenName, PoolOption, PoolOptions } from '../../../state/pools/pools';
import { TokenLogo } from '../TokenLogo';
import { Select, Typography } from 'antd';
import AngleDown from '../../../styles/icons/arrow-angle-down.svg';

// Select component for the Token Input (to change which token user is interacting with)
export function TokenSelect(props: {
  // Optionally, specify currently active Pool for this dropdown
  poolTokenName?: string | undefined;
  // Optionally, specify which tokens a user can choose from in the TokenSelect
  tokenOptions?: PoolOption[];
  // Optionally, specify what occurs when the user switches their token
  onChangeToken?: (token: string) => unknown;
  // Optionally, override the styles of the token selector dropdown
  dropdownStyle?: React.CSSProperties;
}): JSX.Element {
  const [currentPoolTokenName, setCurrentPoolTokenName] = useRecoilState(CurrentPoolTokenName);
  const poolOptions = useRecoilValue(PoolOptions);
  const { Paragraph, Text } = Typography;
  const { Option } = Select;

  return (
    <Select
      dropdownClassName="token-input-dropdown dropdown-space-between"
      dropdownStyle={props.dropdownStyle}
      value={props.poolTokenName ? props.poolTokenName : currentPoolTokenName}
      onChange={tokenName => {
        // If there is a specified action on a token change
        if (props.onChangeToken) {
          props.onChangeToken(tokenName);
          return;
        }

        // Default to updating the currentPool
        setCurrentPoolTokenName(tokenName);
      }}>
      {(props.tokenOptions ?? poolOptions).map(option => (
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
