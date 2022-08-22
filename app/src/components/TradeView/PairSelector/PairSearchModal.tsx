import { useEffect, useState } from 'react';
import { useSetRecoilState, useResetRecoilState, useRecoilValue } from 'recoil';
import { Dictionary } from '../../../state/settings/localization/localization';
import { PairSearchModal as PairSearchModalState } from '../../../state/modals/modals';
import { CurrentPoolSymbol } from '../../../state/borrow/pools';
import { CurrentMarketPair, MarketPairs } from '../../../state/trade/market';
import { formatMarketPair } from '../../../utils/format';
import { Modal, Input, AutoComplete } from 'antd';
import { TokenLogo } from '../../misc/TokenLogo';

export function PairSearchModal() {
  const dictionary = useRecoilValue(Dictionary);
  const pairSearchModalOpen = useRecoilValue(PairSearchModalState);
  const resetPairSearchModal = useResetRecoilState(PairSearchModalState);
  const setCurrentPoolSymbol = useSetRecoilState(CurrentPoolSymbol);
  const marketPairs = useRecoilValue(MarketPairs);
  const setCurrentMarketPair = useSetRecoilState(CurrentMarketPair);
  const [pairSearchOptions, setPairSearchOptions] = useState<{ label: JSX.Element; value: string }[]>([]);
  const [filteredSearchOptions, setFilteredSearchOptions] = useState<{ label: JSX.Element; value: string }[]>([]);

  useEffect(() => {
    // Setup quick search pair options
    const searchOptions = [];
    for (const pair of marketPairs) {
      searchOptions.push({
        label: (
          <>
            <TokenLogo height={35} symbol={pair.split('/')[0]} />
            {formatMarketPair(pair) ?? '—'}
          </>
        ),
        value: pair
      });
    }
    setPairSearchOptions(searchOptions);
    setFilteredSearchOptions(searchOptions);
  }, [marketPairs]);

  // Have to manage visibility like this so it destroys correctly
  if (pairSearchModalOpen) {
    return (
      <Modal visible className="pair-search-modal" footer={null} onCancel={resetPairSearchModal}>
        <AutoComplete
          autoFocus
          dropdownClassName="xl-dropdown"
          options={filteredSearchOptions}
          onSelect={(pair: string) => {
            setCurrentMarketPair(pair.replaceAll(' ', ''));
            setCurrentPoolSymbol(pair.split('/')[0]);
            resetPairSearchModal();
          }}
          onSearch={(query: string) => {
            const filteredOptions = [];
            for (const pair of pairSearchOptions) {
              const pairStripped = pair.value.replaceAll('/', '').replaceAll(' ', '').toLowerCase();
              const queryStripped = query.replaceAll('/', '').replaceAll(' ', '').toLowerCase();
              if (pairStripped.includes(queryStripped)) {
                filteredOptions.push(pair);
              }
            }
            setFilteredSearchOptions(filteredOptions);
          }}
          children={
            <Input
              type="text"
              className="secondary-input"
              placeholder={dictionary.tradeView.pairSelector.inputPlaceholder}
              onPressEnter={() => {
                setCurrentMarketPair(filteredSearchOptions[0].value);
                setCurrentPoolSymbol(filteredSearchOptions[0].value.split('/')[0]);
                resetPairSearchModal();
              }}
            />
          }
        />
      </Modal>
    );
  } else {
    return <></>;
  }
}
