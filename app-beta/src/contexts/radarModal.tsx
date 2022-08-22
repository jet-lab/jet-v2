import { createContext, useContext, useState } from 'react';

// Radar modal context
interface RadarModal {
  radarOpen: boolean;
  setRadarOpen: (radarOpen: boolean) => void;
}
const RadarModalContext = createContext<RadarModal>({
  radarOpen: false,
  setRadarOpen: () => null
});

// Radar modal context provider
export function RadarModalProvider(props: { children: JSX.Element[] }): JSX.Element {
  const [radarOpen, setRadarOpen] = useState(false);
  return (
    <RadarModalContext.Provider
      value={{
        radarOpen,
        setRadarOpen
      }}>
      {props.children}
    </RadarModalContext.Provider>
  );
}

//  Radar modal hook
export const useRadarModal = () => {
  const context = useContext(RadarModalContext);
  return context;
};
