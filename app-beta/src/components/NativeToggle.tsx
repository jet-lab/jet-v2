import { useNativeValues } from '../contexts/nativeValues';
import { ReactComponent as CryptoIcon } from '../styles/icons/crypto_icon.svg';
import { ReactComponent as UsdIcon } from '../styles/icons/usd_icon.svg';

export function NativeToggle(): JSX.Element {
  const { nativeValues, toggleNativeValues } = useNativeValues();

  return (
    <div
      className={`native-toggle flex align-center justify-start ${nativeValues ? 'active justify-end' : ''}`}
      onClick={() => toggleNativeValues()}>
      <div className={` flex-centered ${nativeValues ? 'active' : ''}`}>
        <CryptoIcon width="20px" height="20px" />
      </div>
      <div className={`flex-centered ${!nativeValues ? 'active' : ''}`}>
        <UsdIcon width="20px" height="20px" />
      </div>
    </div>
  );
}
