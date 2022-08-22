import { useLanguage } from '../contexts/localization/localization';
import { useRpcNode } from '../contexts/rpcNode';

export function NetworkWarningBanner(): JSX.Element {
  const { dictionary } = useLanguage();
  const { degradedNetworkPerformance } = useRpcNode();

  if (degradedNetworkPerformance) {
    return (
      <div className="network-warning-banner flex-centered">
        <span className="semi-bold-text">
          {dictionary.settings.degradedNetworkPerformance}&nbsp;
          <a className="text-btn" href="https://status.solana.com/" target="_blank" rel="noopener noreferrer">
            https://status.solana.com/
          </a>
        </span>
      </div>
    );
  } else {
    return <></>;
  }
}
