import { createContext, useContext, useEffect, useState } from 'react';
import Jet_UI_EN from './languages/Jet_UI_EN.json';
import Jet_Definitions_EN from './languages/Jet_Definitions_EN.json';
import Jet_UI_ZH from './languages/Jet_UI_ZH.json';
import Jet_Definitions_ZH from './languages/Jet_Definitions_ZH.json';
import Jet_UI_KR from './languages/Jet_UI_KR.json';
import Jet_Definitions_KR from './languages/Jet_Definitions_KR.json';
import Jet_UI_RU from './languages/Jet_UI_RU.json';
import Jet_Definitions_RU from './languages/Jet_Definitions_RU.json';
import Jet_UI_TR from './languages/Jet_UI_TR.json';
import Jet_Definitions_TR from './languages/Jet_Definitions_TR.json';
import Jet_UI_DE from './languages/Jet_UI_DE.json';
import Jet_Definitions_DE from './languages/Jet_Definitions_DE.json';
import Jet_UI_IT from './languages/Jet_UI_IT.json';
import Jet_Definitions_IT from './languages/Jet_Definitions_IT.json';
import Jet_UI_SP from './languages/Jet_UI_SP.json';
import Jet_Definitions_SP from './languages/Jet_Definitions_SP.json';

// Localization context
interface Localization {
  preferredLanguage: string;
  setPreferredLanguage: (lang: string) => void;
  isGeobanned: boolean;
}
const LocalizationContext = createContext<Localization>({
  preferredLanguage: '',
  setPreferredLanguage: () => null,
  isGeobanned: false
});

// Localization context provider
export function LocalizationProvider(props: { children: JSX.Element }): JSX.Element {
  const preferredLang = localStorage.getItem('jetPreferredLanguage');
  const [preferredLanguage, setPreferredLanguage] = useState(preferredLang ?? 'en');
  const [isGeobanned, setIsGeobanned] = useState(false);

  // Get user's preferred language from browser
  useEffect(() => {
    let locale: any = null;

    // Get user's IP to determine location/geobanning
    async function getIP() {
      const ipKey = process.env.REACT_APP_IP_REGISTRY;

      try {
        const resp = await fetch(`https://api.ipregistry.co/?key=${ipKey}`, {
          method: 'GET',
          headers: { 'Content-Type': 'application/json' }
        });

        locale = await resp.json();
        const countryCode = locale.location.country.code;
        geoBannedCountries.forEach(c => {
          if (c.code === countryCode) {
            // If country is Ukraine, checks if in Crimea.
            if (countryCode !== 'UA' || isCrimea(locale)) {
              setIsGeobanned(true);
            }
          }
        });
      } catch (err) {
        console.log(err);
      }
    }

    // Check to see if user's locale is special case of Crimea
    const isCrimea = (locale: any) => {
      // Crimea region code for ipregistry is UA-43
      return locale.location.region.code === 'UA-43';
    };

    getIP();
  }, [setPreferredLanguage]);

  return (
    <LocalizationContext.Provider
      value={{
        preferredLanguage,
        setPreferredLanguage,
        isGeobanned
      }}>
      {props.children}
    </LocalizationContext.Provider>
  );
}

// Geoban check hook
export const useGeoban = () => {
  const context = useContext(LocalizationContext);
  return context.isGeobanned;
};

// Language Hook
export const useLanguage = () => {
  const { preferredLanguage, setPreferredLanguage } = useContext(LocalizationContext);
  return {
    language: preferredLanguage,
    dictionary: uiDictionary[preferredLanguage],
    changeLanguage: (lang: string) => {
      localStorage.setItem('jetPreferredLanguage', lang);
      setPreferredLanguage(lang);
    }
  };
};

// UI dictionary
export const uiDictionary: any = {
  // English
  en: Jet_UI_EN,
  // Mandarin
  zh: Jet_UI_ZH,
  // Russian
  ru: Jet_UI_RU,
  // Turkish
  tr: Jet_UI_TR,
  // Korean
  kr: Jet_UI_KR,
  // German
  de: Jet_UI_DE,
  // Italian
  it: Jet_UI_IT,
  // Spanish
  sp: Jet_UI_SP
};

// Definitions of various terminology
export const definitions: any = {
  // English
  en: Jet_Definitions_EN,
  // Mandarin
  zh: Jet_Definitions_ZH,
  // Russian
  ru: Jet_Definitions_RU,
  // Turkish
  tr: Jet_Definitions_TR,
  // Korean
  kr: Jet_Definitions_KR,
  // German
  de: Jet_Definitions_DE,
  // Italian
  it: Jet_Definitions_IT,
  // Spanish
  sp: Jet_Definitions_SP
};

// Banned countries
export const geoBannedCountries = [
  {
    country: 'Afghanistan',
    code: 'AF'
  },
  {
    country: 'Crimea (Ukraine)',
    code: 'UA'
  },
  {
    country: 'Cuba',
    code: 'CU'
  },
  {
    country: 'Democratic Republic of Congo',
    code: 'CD'
  },
  {
    country: 'Iran',
    code: 'IR'
  },
  {
    country: 'Iraq',
    code: 'IQ'
  },
  {
    country: 'Libya',
    code: 'LY'
  },
  {
    country: 'North Korea',
    code: 'KP'
  },
  {
    country: 'Sudan',
    code: 'SD'
  },
  {
    country: 'Syria',
    code: 'SY'
  },
  {
    country: 'Tajikistan',
    code: 'TJ'
  },
  {
    country: 'Venezuela',
    code: 'VE'
  }
];
