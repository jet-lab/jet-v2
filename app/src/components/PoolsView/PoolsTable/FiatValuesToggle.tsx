import { useRecoilState } from 'recoil';
import { FiatValues } from '../../../state/settings/settings';
import { ReactComponent as CryptoIcon } from '../../../styles/icons/crypto-icon.svg';
import { ReactComponent as UsdIcon } from '../../../styles/icons/usd-icon.svg';

export function FiatValuesToggle(): JSX.Element {
  const [fiatValues, setFiatValues] = useRecoilState(FiatValues);

  return (
    <div
      className={`fiat-toggle flex align-center justify-start ${!fiatValues ? 'active justify-end' : ''}`}
      onClick={() => setFiatValues(fiatValues)}>
      <div className={`fiat-toggle-half flex-centered ${!fiatValues ? 'active' : ''}`}>
        <CryptoIcon className="jet-icon" width="20px" height="20px" />
      </div>
      <div className={`fiat-toggle-half flex-centered ${fiatValues ? 'active' : ''}`}>
        <UsdIcon className="jet-icon" width="20px" height="20px" />
      </div>
    </div>
  );
}
