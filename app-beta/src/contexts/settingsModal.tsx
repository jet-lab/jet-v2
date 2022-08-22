import { createContext, useContext, useState } from 'react';

// Settings modal context
interface SettingsModal {
  open: boolean;
  setOpen: (open: boolean) => void;
}
const SettingsModalContext = createContext<SettingsModal>({
  open: false,
  setOpen: () => null
});

// Settings modal context provider
export function SettingsModalProvider(props: { children: any }): JSX.Element {
  const [open, setOpen] = useState(false);
  return (
    <SettingsModalContext.Provider
      value={{
        open,
        setOpen
      }}>
      {props.children}
    </SettingsModalContext.Provider>
  );
}

//  Settings modal hook
export const useSettingsModal = () => {
  const context = useContext(SettingsModalContext);
  return context;
};
