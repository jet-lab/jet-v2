import { atom, selector } from 'recoil';
import axios from 'axios';
import Jet_UI_EN from './languages/Jet_UI_EN.json';
import Jet_Definitions_EN from './languages/Jet_Definitions_EN.json';
import { localStorageEffect } from '../../effects/localStorageEffect';
import { WalletPublicKey } from '@state/user/accounts';

// UI Localization Dictionary
export const Dictionary = atom({
  key: 'dictionary',
  default: Jet_UI_EN
});

// Preferred language
export const PreferredLanguage = atom({
  key: 'preferredLanguage',
  default: 'en',
  effects: [localStorageEffect('jetAppPreferredLanguage')]
});

interface GeoLocation {
  ip: string;
  type: string;
  location: {
    city: string;
    continent: {
      code: string;
      name: string;
    };
    country: {
      area: number;
      borders: string[];
      calling_code: string;
      capital: string;
      code: string;
      name: string;
    };
    in_eu: boolean;
    region: {
      code: string;
      name: string;
    };
  };
  security: {
    is_abuser: boolean;
    is_anonymous: boolean;
    is_attacker: boolean;
    is_bogon: boolean;
    is_cloud_provider: boolean;
    is_proxy: boolean;
    is_relay: boolean;
    is_threat: boolean;
    is_tor: boolean;
    is_tor_exit: boolean;
    is_vpn: boolean;
  };
  time_zone: {
    id: string;
  };
  user_agent: {
    header: string;
    name: string;
    os: {
      name: string;
      type: string;
    };
    version: string;
  };
}

// Whether user is geobanned
export const Geobanned = selector<boolean>({
  key: 'geobanned',
  get: async ({ get }) => {
    const data = await axios
      .get(`https://api.ipregistry.co/?key=${process.env.REACT_APP_IP_REGISTRY}`)
      .then(({ data }: { data: GeoLocation }) => data);
    {
      const countryCode = data.location.country.code;
      const tz = data.time_zone.id;
      const wallet = get(WalletPublicKey);
      if (wallet && process.env.REACT_APP_ALLOWED_WALLETS?.includes(wallet)) {
        // is whitelisted
        return false;
      }

      if (geoBannedCountries.map(country => country.code).includes(countryCode)) {
        // Is in a geobanned country
        return true;
      }

      if (Object.values(data.security).some(v => v)) {
        // is masking traffic
        return true;
      }

      if (tz !== Intl.DateTimeFormat().resolvedOptions().timeZone) {
        // Timezone is different from that of the IP resolved one
        return true;
      }

      return false;
    }
  }
});

// UI dictionary
export const uiDictionary: any = {
  // English
  en: Jet_UI_EN
};

// Definitions of various terminology
export const definitions: any = {
  // English
  en: Jet_Definitions_EN
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
    country: 'Russia',
    code: 'RU'
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
    country: 'Ukraine',
    code: 'UA'
  },
  {
    country: 'United States',
    code: 'US'
  },
  {
    country: 'Venezuela',
    code: 'VE'
  },
  {
    country: 'United Kingdom',
    code: 'GB'
  }
];
