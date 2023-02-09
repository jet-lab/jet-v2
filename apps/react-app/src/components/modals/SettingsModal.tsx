import { useState } from 'react';
import { useRecoilState, useRecoilValue, useResetRecoilState } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { SettingsModal as SettingsModalState } from '@state/modals/modals';
import {
  Explorer,
  blockExplorers,
  PreferredTimeDisplay,
  timeDisplayOptions,
  PreferDayMonthYear,
  FiatCurrency,
  fiatOptions
} from '@state/settings/settings';
import { Modal, Radio, Select, Typography } from 'antd';
import AngleDown from '@assets/icons/arrow-angle-down.svg';
import { useJetStore } from '@jet-lab/store';

// Modal for changing app preferences
export function SettingsModal(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const settingsModalOpen = useRecoilValue(SettingsModalState);
  const resetSettingsModalOpen = useResetRecoilState(SettingsModalState);
  // Cluster
  const { settings, updateSettings } = useJetStore(state => ({
    settings: state.settings,
    updateSettings: state.updateSettings
  }));
  const [clusterSetting, setClusterSetting] = useState(settings.cluster);
  // Fiat Currency
  const [fiatCurrency, setFiatCurrency] = useRecoilState(FiatCurrency);
  const [fiatCurrencySetting, setFiatCurrencySetting] = useState(fiatCurrency);
  // Explorer
  const [explorerSetting, setExplorerSetting] = useState(settings.explorer);

  // Time Display
  const [preferredTimeDisplay, setPreferredTimeDisplay] = useRecoilState(PreferredTimeDisplay);
  const [preferredTimeDisplaySetting, setPreferredTimeDisplaySetting] = useState(preferredTimeDisplay);
  const [preferDayMonthYear, setPreferDayMonthYear] = useRecoilState(PreferDayMonthYear);
  const [preferDayMonthYearSetting, setPreferDayMonthYearSetting] = useState(preferDayMonthYear);
  const [loading, setLoading] = useState(false);
  const { Title, Text } = Typography;
  const { Option } = Select;

  // Save settings to global state and localstorage
  async function saveSettings() {
    setLoading(true);
    if (fiatCurrencySetting !== fiatCurrency) {
      setFiatCurrency(fiatCurrencySetting);
    }
    if (preferredTimeDisplaySetting !== preferredTimeDisplay) {
      setPreferredTimeDisplay(preferredTimeDisplaySetting);
    }
    if (preferDayMonthYearSetting !== preferDayMonthYear) {
      setPreferDayMonthYear(preferDayMonthYearSetting);
    }
    updateSettings({
      cluster: clusterSetting,
      explorer: explorerSetting
    });
    resetSettingsModalOpen();
    setLoading(false);
  }

  // Reset settings to their global state on cancel
  function cancelSettings() {
    setClusterSetting(settings.cluster);
    setFiatCurrencySetting(fiatCurrency);
    setExplorerSetting(settings.explorer);
    setPreferredTimeDisplaySetting(preferredTimeDisplay);
    setPreferDayMonthYearSetting(preferDayMonthYear);
    resetSettingsModalOpen();
  }
  if (settingsModalOpen) {
    return (
      <Modal
        open
        className="settings-modal header-modal show-scrollbar"
        maskClosable={false}
        onCancel={cancelSettings}
        cancelText={dictionary.modals.cancel}
        onOk={saveSettings}
        okText={dictionary.settingsModal.savePreferences}
        okButtonProps={{ disabled: loading, loading }}>
        <div className="modal-header flex-centered">
          <Title className="modal-header-title green-text">{dictionary.settingsModal.title}</Title>
        </div>
        <div className="setting flex align-start justify-center column">
          <Text strong className="setting-title">
            {dictionary.settingsModal.cluster.network.toUpperCase()}
          </Text>
          <Radio.Group value={clusterSetting} onChange={e => setClusterSetting(e.target.value)}>
            <Radio value="mainnet-beta">{dictionary.settingsModal.cluster.mainnetBeta}</Radio>
            <Radio value="devnet">{dictionary.settingsModal.cluster.devnet}</Radio>
            <Radio value="localnet">{dictionary.settingsModal.cluster.localnet}</Radio>
          </Radio.Group>
        </div>
        <div className="setting flex align-start justify-center column">
          <Text strong className="setting-title">
            {dictionary.settingsModal.explorer.title.toUpperCase()}
          </Text>
          <Radio.Group
            className="flex column"
            value={explorerSetting}
            onChange={e => setExplorerSetting(e.target.value)}>
            {Object.keys(blockExplorers).map(explorer => (
              <Radio key={explorer} value={explorer}>
                {blockExplorers[explorer as Explorer].name}
              </Radio>
            ))}
          </Radio.Group>
        </div>
        <div className="setting flex align-start justify-center column">
          <Text strong className="setting-title">
            {dictionary.settingsModal.fiat.title.toUpperCase()}
          </Text>
          <Select
            value={fiatCurrencySetting}
            suffixIcon={<AngleDown className="jet-icon" />}
            onChange={value => setFiatCurrencySetting(value)}
            popupClassName="dropdown-space-between">
            {Object.keys(fiatOptions).map(abbrev => (
              <Option key={abbrev} value={abbrev}>
                {/* @ts-ignore */}
                <Text>{dictionary.settingsModal.fiat[abbrev]}</Text>
                <Text style={{ marginLeft: 10 }}>{fiatOptions[abbrev].length ? fiatOptions[abbrev] : abbrev}</Text>
              </Option>
            ))}
          </Select>
        </div>
        <div className="setting flex align-start justify-center column">
          <Text strong className="setting-title">
            {dictionary.settingsModal.timeDisplay.title.toUpperCase()}
          </Text>
          <Radio.Group
            value={preferredTimeDisplaySetting}
            onChange={e => setPreferredTimeDisplaySetting(e.target.value)}>
            {timeDisplayOptions.map(option => (
              <Radio key={option} value={option}>
                {dictionary.settingsModal.timeDisplay[option]}
              </Radio>
            ))}
          </Radio.Group>
        </div>
        <div className="setting flex align-start justify-center column">
          <Text strong className="setting-title">
            {dictionary.settingsModal.dateFormat.title.toUpperCase()}
          </Text>
          <Radio.Group value={preferDayMonthYearSetting} onChange={e => setPreferDayMonthYearSetting(e.target.value)}>
            <Radio key="dayMonthYear" value={true}>
              {dictionary.settingsModal.dateFormat.dayMonthYear}
            </Radio>
            <Radio key="monthDayYear" value={false}>
              {dictionary.settingsModal.dateFormat.monthDayYear}
            </Radio>
          </Radio.Group>
        </div>
      </Modal>
    );
  } else {
    return <></>;
  }
}
