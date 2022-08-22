import { Skeleton } from 'antd';
import { ReactComponent as USDC } from '../../styles/icons/cryptos/USDC.svg';
import { ReactComponent as SOL } from '../../styles/icons/cryptos/SOL.svg';
import { ReactComponent as BTC } from '../../styles/icons/cryptos/BTC.svg';
import { ReactComponent as SRM } from '../../styles/icons/cryptos/SRM.svg';
import { ReactComponent as ETH } from '../../styles/icons/cryptos/ETH.svg';
import { ReactComponent as USDT } from '../../styles/icons/cryptos/USDT.svg';
import { ReactComponent as MSOL } from '../../styles/icons/cryptos/MSOL.svg';

export function TokenLogo(props: {
  symbol: string | undefined;
  height: number;
  style?: React.CSSProperties;
}): JSX.Element {
  const { symbol, height } = props;

  if (symbol === 'USDC') {
    return <USDC className="token-logo" height={height} width={height} style={props.style} />;
  } else if (symbol === 'SOL') {
    return <SOL className="token-logo" height={height} width={height} style={props.style} />;
  } else if (symbol === 'BTC') {
    return <BTC className="token-logo" height={height} width={height} style={props.style} />;
  } else if (symbol === 'SRM' || symbol === 'MSRM') {
    return <SRM className="token-logo" height={height} width={height} style={props.style} />;
  } else if (symbol === 'ETH') {
    return <ETH className="token-logo" height={height} width={height} style={props.style} />;
  } else if (symbol === 'USDT') {
    return <USDT className="token-logo" height={height} width={height} style={props.style} />;
  } else if (symbol === 'mSOL') {
    return <MSOL className="token-logo" height={height} width={height} style={props.style} />;
  } else {
    return <Skeleton.Avatar active size={height} shape="square" style={props.style} />;
  }
}
