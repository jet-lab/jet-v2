import { atom } from 'recoil';
import axios from 'axios';
import Jet_UI_EN from './languages/Jet_UI_EN.json';
import Jet_Definitions_EN from './languages/Jet_Definitions_EN.json';
import { localStorageEffect } from '../../effects/localStorageEffect';

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

// Whether user is geobanned
export const Geobanned = atom({
  key: 'geobanned',
  default: false,
  effects: [
    ({ setSelf }) => {
      axios
        .get(`https://api.ipregistry.co/?key=${process.env.REACT_APP_IP_REGISTRY}`)
        .then(({ data }) => {
          const locale = data.data;
          if (locale) {
            const countryCode = locale.location.country.code;
            if (countryCode) {
              geoBannedCountries.forEach(c => {
                if (c.code === countryCode) {
                  // If country is Ukraine, checks if first two digits
                  // of the postal code further match Crimean postal codes.
                  if (countryCode !== 'UA' || locale.location.region.code === 'UA-43') {
                    setSelf(true);
                  }
                }
              });
            }
          }
        })
        .catch(err => err);
    }
  ]
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
