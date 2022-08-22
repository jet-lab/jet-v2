import { useTradeContext } from '../contexts/tradeContext';
import { currencyFormatter } from '../utils/currency';
import { Input } from 'antd';
import { AssetLogo } from './AssetLogo';
import { ReactComponent as ArrowIcon } from '../styles/icons/arrow_icon.svg';
import { LoadingOutlined } from '@ant-design/icons';

export function JetInput(props: {
  type: 'text' | 'number';
  value: string | number | null;
  placeholder?: string;
  currency?: boolean;
  maxInput?: number | null;
  error?: string | null;
  warning?: string | null;
  disabled?: boolean;
  disabledButton?: boolean;
  loading?: boolean;
  onClick?: () => unknown;
  onChange: (value: any) => unknown;
  submit: () => unknown;
}): JSX.Element {
  const { currentPool } = useTradeContext();

  return (
    <div className={`jet-input flex-centered ${props.disabled ? 'disabled' : ''}`}>
      <div className={`flex-centered ${props.currency ? 'currency-input' : ''}`}>
        <Input
          data-testid="jet-trade-input"
          type={props.type}
          disabled={props.disabled}
          value={props.value || ''}
          placeholder={props.placeholder}
          className={props.error ? 'error' : props.warning ? 'warning' : ''}
          onClick={() => (props.onClick ? props.onClick() : null)}
          onChange={e => {
            if (currentPool && props.maxInput && props.maxInput < e.target.valueAsNumber) {
              e.target.value = props.maxInput.toString();
              props.onChange(props.maxInput);
            } else {
              props.onChange(e.target.value);
            }
          }}
          onPressEnter={() => (props.disabled || props.error ? null : props.submit())}
        />
        {props.currency && currentPool && (
          <>
            <AssetLogo symbol={currentPool.tokenConfig?.symbol || ''} height={20} />
            <div className="asset-abbrev-usd flex align-end justify-center column">
              <span>{currentPool.tokenConfig?.symbol}</span>
              <span>
                ≈{' '}
                {currencyFormatter(
                  (Number(props.value) ?? 0) * (currentPool.tokenPrice !== undefined ? currentPool.tokenPrice : 0),
                  true,
                  2
                )}
              </span>
            </div>
          </>
        )}
      </div>
      <div
        data-testid="jet-trade-button"
        className={`input-btn flex-centered ${props.loading ? 'loading' : ''} ${
          props.disabledButton ? 'disabled' : ''
        }`}
        onClick={() => {
          if (props.loading) {
            return;
          } else if (!props.disabled && !props.disabledButton && !props.error && props.value) {
            props.submit();
          }
        }}>
        {props.loading ? <LoadingOutlined /> : <ArrowIcon width={25} />}
      </div>
    </div>
  );
}
