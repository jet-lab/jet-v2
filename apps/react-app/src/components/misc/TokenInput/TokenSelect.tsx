import { useRecoilValue } from 'recoil';
import { PoolOption, PoolOptions, Pools } from '@state/pools/pools';
import { TokenLogo } from '../TokenLogo';
import { Select, Typography } from 'antd';
import AngleDown from '@assets/icons/arrow-angle-down.svg';
import { useJetStore } from '@jet-lab/store';
import { useMemo } from 'react';

// Select component for the Token Input (to change which token user is interacting with)
export function TokenSelect(props: {
  // Optionally, specify currently active Pool for this dropdown
  poolSymbol?: string | undefined;
  // Optionally, specify which tokens a user can choose from in the TokenSelect
  tokenOptions?: PoolOption[];
  // Optionally, specify what occurs when the user switches their token
  onChangeToken?: (token: string) => unknown;
  // Optionally, override the styles of the token selector dropdown
  dropdownStyle?: React.CSSProperties;
}): JSX.Element {
  const poolOptions = useRecoilValue(PoolOptions);
  const { Paragraph, Text } = Typography;
  const { Option } = Select;
  const pools = useRecoilValue(Pools);

  const { selectedPoolKey, selectPool } = useJetStore(state => ({
    selectedPoolKey: state.selectedPoolKey,
    selectPool: state.selectPool
  }));

  const currentPool = useMemo(
    () =>
      pools?.tokenPools && Object.values(pools?.tokenPools).find(pool => pool.address.toBase58() === selectedPoolKey),
    [selectedPoolKey, pools]
  );

  return (
    <Select
      popupClassName="token-input-dropdown dropdown-space-between"
      dropdownStyle={props.dropdownStyle}
      value={props.poolSymbol ? props.poolSymbol : currentPool?.symbol}
      onChange={tokenSymbol => {
        // If there is a specified action on a token change
        if (props.onChangeToken) {
          props.onChangeToken(tokenSymbol);
          return;
        }
        const selectedPool = pools && Object.values(pools.tokenPools).find(p => p.symbol === tokenSymbol);

        // Default to updating the currentPool
        selectedPool && selectPool(selectedPool.address.toBase58());
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
