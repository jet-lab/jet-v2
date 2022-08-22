import { Skeleton } from 'antd';
import { ReactComponent as USDC } from '../styles/icons/cryptos/USDC.svg';
import { ReactComponent as SOL } from '../styles/icons/cryptos/SOL.svg';
import { ReactComponent as BTC } from '../styles/icons/cryptos/BTC.svg';
import { ReactComponent as ETH } from '../styles/icons/cryptos/ETH.svg';
import { ReactComponent as SRM } from '../styles/icons/cryptos/SRM.svg';
import { ReactComponent as USDT } from '../styles/icons/cryptos/USDT.svg';
import { ReactComponent as MSOL } from '../styles/icons/cryptos/MSOL.svg';

export function AssetLogo(props: { symbol: string; height: number; style?: React.CSSProperties }): JSX.Element {
  const { symbol, height } = props;

  if (symbol === 'USDC') {
    return <USDC className="asset-logo" height={height} width={height} style={props.style} />;
  } else if (symbol === 'SOL') {
    return <SOL className="asset-logo" height={height} width={height} style={props.style} />;
  } else if (symbol === 'BTC') {
    return <BTC className="asset-logo" height={height} width={height} style={props.style} />;
  } else if (symbol === 'SRM') {
    return <SRM className="asset-logo" height={height} width={height} style={props.style} />;
  } else if (symbol === 'ETH') {
    return <ETH className="asset-logo" height={height} width={height} style={props.style} />;
  } else if (symbol === 'USDT') {
    return <USDT className="asset-logo" height={height} width={height} style={props.style} />;
  } else if (symbol === 'mSOL') {
    return <MSOL className="asset-logo" height={height} width={height} style={props.style} />;
  } else {
    return <Skeleton.Avatar className="asset-logo" active size={height} shape="square" style={props.style} />;
  }
}
