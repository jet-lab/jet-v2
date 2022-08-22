import { createContext, useContext, useState } from 'react';

// Initializing app context
interface Init {
  initFailed: boolean;
  setInitFailed: (failed: boolean) => void;
}
const InitContext = createContext<Init>({
  initFailed: false,
  setInitFailed: () => null
});

// Initializing app context provider
export function InitProvider(props: { children: JSX.Element }): JSX.Element {
  const [initFailed, setInitFailed] = useState(false);
  return (
    <InitContext.Provider
      value={{
        initFailed,
        setInitFailed
      }}>
      {props.children}
    </InitContext.Provider>
  );
}

// Init failed hook
export const useInitFailed = () => {
  const context = useContext(InitContext);
  return context;
};
