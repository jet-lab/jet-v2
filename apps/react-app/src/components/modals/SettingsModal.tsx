import { useEffect, useState } from 'react';
import { useRecoilState, useRecoilValue, useResetRecoilState } from 'recoil';
import { Dictionary, uiDictionary, PreferredLanguage } from '@state/settings/localization/localization';
import { SettingsModal as SettingsModalState } from '@state/modals/modals';
import {
  Explorer,
  BlockExplorer,
  blockExplorers,
  RpcNodes,
  rpcNodeOptions,
  PreferredRpcNode,
  PreferredTimeDisplay,
  timeDisplayOptions,
  PreferDayMonthYear,
  FiatCurrency,
  fiatOptions
} from '@state/settings/settings';
import { getPing } from '@utils/ui';
import { Input, Modal, Radio, Select, Typography } from 'antd';
import AngleDown from '@assets/icons/arrow-angle-down.svg';
import debounce from 'lodash.debounce';
import { useJetStore } from '@jet-lab/store';

// Modal for changing app preferences
export function SettingsModal(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const settingsModalOpen = useRecoilValue(SettingsModalState);
  const resetSettingsModalOpen = useResetRecoilState(SettingsModalState);
  // Cluster
  const { cluster, updateSetting } = useJetStore(state => ({
    cluster: state.settings.cluster,
    updateSetting: state.updateSetting
  }));
  const [clusterSetting, setClusterSetting] = useState(cluster);
  // Rpc Node
  const [rpcNodes, setRpcNodes] = useRecoilState(RpcNodes);
  const [preferredNode, setPreferredNode] = useRecoilState(PreferredRpcNode);
  const [preferredNodeSetting, setPreferredNodeSetting] = useState(preferredNode);
  const nodeIndexer = cluster === 'mainnet-beta' ? 'mainnetBeta' : 'devnet';
  const [customNodeInput, setCustomNodeInput] = useState(rpcNodes.custom[nodeIndexer]);
  const [customNodeInputError, setCustomNodeInputError] = useState('');
  // Fiat Currency
  const [fiatCurrency, setFiatCurrency] = useRecoilState(FiatCurrency);
  const [fiatCurrencySetting, setFiatCurrencySetting] = useState(fiatCurrency);
  // Explorer
  const [explorer, setExplorer] = useRecoilState(BlockExplorer);
  const [explorerSetting, setExplorerSetting] = useState(explorer);
  // Language
  const [preferredLanguage, setPreferredLanguage] = useRecoilState(PreferredLanguage);
  const [preferredLanguageSetting, setPreferredLanguageSetting] = useState(preferredLanguage);
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
    if (preferredNodeSetting === 'custom') {
      const ping = await getPing(customNodeInput);
      if (ping) {
        localStorage.setItem(`jetCustomNode-${cluster}`, customNodeInput);
        rpcNodes.custom[nodeIndexer] = customNodeInput;
        rpcNodes.custom[`${nodeIndexer}Ping`] = ping;
        setCustomNodeInputError('');
        setRpcNodes(rpcNodes);
      } else {
        setCustomNodeInputError(dictionary.settingsModal.rpcNode.errorMessages.invalidNode);
        setLoading(false);
        return;
      }
    }
    if (preferredNodeSetting !== preferredNode) {
      setPreferredNode(preferredNodeSetting);
    }
    if (clusterSetting !== cluster) {
      updateSetting('cluster', clusterSetting);
    }
    if (fiatCurrencySetting !== fiatCurrency) {
      setFiatCurrency(fiatCurrencySetting);
    }
    if (explorerSetting !== explorer) {
      setExplorer(explorerSetting);
    }
    if (preferredLanguageSetting !== preferredLanguage) {
      setPreferredLanguage(preferredLanguageSetting);
    }
    if (preferredTimeDisplaySetting !== preferredTimeDisplay) {
      setPreferredTimeDisplay(preferredTimeDisplaySetting);
    }
    if (preferDayMonthYearSetting !== preferDayMonthYear) {
      setPreferDayMonthYear(preferDayMonthYearSetting);
    }
    resetSettingsModalOpen();
    setLoading(false);
  }

  // Reset settings to their global state on cancel
  function cancelSettings() {
    setPreferredNodeSetting(preferredNode);
    setCustomNodeInput(rpcNodes.custom[nodeIndexer]);
    setCustomNodeInputError('');
    setClusterSetting(cluster);
    setFiatCurrencySetting(fiatCurrency);
    setExplorerSetting(explorer);
    setPreferredLanguageSetting(preferredLanguage);
    setPreferredTimeDisplaySetting(preferredTimeDisplay);
    setPreferDayMonthYearSetting(preferDayMonthYear);
    resetSettingsModalOpen();
  }

  // Check if anything has changes
  function checkSettingsChange() {
    if (
      customNodeInput !== rpcNodes.custom[nodeIndexer] ||
      preferredNodeSetting !== preferredNode ||
      clusterSetting !== cluster ||
      fiatCurrencySetting !== fiatCurrency ||
      explorerSetting !== explorer ||
      preferredLanguageSetting !== preferredLanguage ||
      preferredTimeDisplaySetting !== preferredTimeDisplay ||
      preferDayMonthYearSetting !== preferDayMonthYear
    ) {
      return true;
    }

    return false;
  }

  // Localize 'custom' option on mount
  useEffect(() => {
    rpcNodes.custom.name = dictionary.settingsModal.rpcNode.custom;
    setRpcNodes(rpcNodes);
  }, [dictionary.settingsModal.rpcNode.custom, rpcNodes, setRpcNodes]);

  // Returns RPC ping className for styling
  function getPingClassName(ping: number) {
    let className = 'ping-indicator-color';
    if (ping < 1000) {
      className += ' fast';
    } else if (ping < 2500) {
      className += ' slow';
    } else {
      className += ' poor';
    }

    return className;
  }

  // Renders custom node input
  function renderCustomInput() {
    let render = <></>;
    if (preferredNodeSetting === 'custom') {
      render = (
        <Input
          className={customNodeInputError ? 'error' : ''}
          value={customNodeInput}
          placeholder={dictionary.settingsModal.rpcNode.customInputPlaceholder}
          onChange={debounce(e => setCustomNodeInput(e.target.value), 300)}
          onPressEnter={() => (checkSettingsChange() ? saveSettings() : null)}
        />
      );
    }

    return render;
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
            {dictionary.settingsModal.rpcNode.title.toUpperCase()}
          </Text>
          <Select
            value={preferredNodeSetting}
            suffixIcon={<AngleDown className="jet-icon" />}
            onChange={node => setPreferredNodeSetting(node)}>
            {rpcNodeOptions.map(node => {
              const nodePing = rpcNodes[node][`${nodeIndexer}Ping`];

              return (
                <Option key={rpcNodes[node].name} value={node}>
                  {rpcNodes[node].name}
                  <div className="ping-indicator flex-centered">
                    <div className={getPingClassName(nodePing)}></div>
                    {nodePing ? nodePing + 'ms' : '(-)'}
                  </div>
                </Option>
              );
            })}
          </Select>
          {renderCustomInput()}
          <Text type="danger">{customNodeInputError}</Text>
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
            {dictionary.settingsModal.language.title.toUpperCase()}
          </Text>
          <Select
            value={preferredLanguageSetting}
            suffixIcon={<AngleDown className="jet-icon" />}
            onChange={value => setPreferredLanguageSetting(value)}>
            {Object.keys(uiDictionary).map(lang => (
              <Option key={lang} value={lang}>
                {uiDictionary[lang].language}
              </Option>
            ))}
          </Select>
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
