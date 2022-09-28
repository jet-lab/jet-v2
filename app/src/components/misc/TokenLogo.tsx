import { Skeleton } from 'antd';
import USDC from '../../styles/icons/cryptos/USDC.svg';
import SOL from '../../styles/icons/cryptos/SOL.svg';
import BTC from '../../styles/icons/cryptos/BTC.svg';
import SRM from '../../styles/icons/cryptos/SRM.svg';
import ETH from '../../styles/icons/cryptos/ETH.svg';
import USDT from '../../styles/icons/cryptos/USDT.svg';
import MSOL from '../../styles/icons/cryptos/MSOL.svg';

// Component to render the SVG logo of a token
export function TokenLogo(props: {
  // Token's symbol
  symbol: string | undefined;
  // Height of logo
  height: number;
  // Optional styling overrides
  style?: React.CSSProperties;
}): JSX.Element {
  const { symbol, height, style } = props;

  switch (symbol) {
    case 'USDC':
      return <USDC className="token-logo" height={height} width={height} style={style} />;
    case 'SOL':
      return <SOL className="token-logo" height={height} width={height} style={style} />;
    case 'BTC':
      return <BTC className="token-logo" height={height} width={height} style={style} />;
    case 'SRM':
      return <SRM className="token-logo" height={height} width={height} style={style} />;
    case 'ETH':
      return <ETH className="token-logo" height={height} width={height} style={style} />;
    case 'USDT':
      return <USDT className="token-logo" height={height} width={height} style={style} />;
    case 'mSOL':
      return <MSOL className="token-logo" height={height} width={height} style={style} />;
    default:
      return <Skeleton.Avatar active size={height} shape="square" style={style} />;
  }
}
