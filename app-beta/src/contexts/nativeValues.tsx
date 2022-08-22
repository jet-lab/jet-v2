import { createContext, useContext, useState } from 'react';

// Native vs USD context
interface NativeValues {
  nativeValues: boolean;
  setNativeValues: (native: boolean) => void;
}
const NativeValuesContext = createContext<NativeValues>({
  nativeValues: true,
  setNativeValues: () => null
});

// Native vs USD context provider
export function NativeValuesProvider(props: { children: JSX.Element }): JSX.Element {
  const [nativeValues, setNativeValues] = useState(true);

  return (
    <NativeValuesContext.Provider
      value={{
        nativeValues,
        setNativeValues
      }}>
      {props.children}
    </NativeValuesContext.Provider>
  );
}

// Native vs USD hook
export const useNativeValues = () => {
  const { nativeValues, setNativeValues } = useContext(NativeValuesContext);

  return {
    nativeValues,
    toggleNativeValues: () => {
      setNativeValues(!nativeValues);
    }
  };
};
