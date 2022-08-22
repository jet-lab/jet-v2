import { useState } from 'react';
import { useLanguage, uiDictionary } from '../contexts/localization/localization';
import { useSettingsModal } from '../contexts/settingsModal';
import { useRpcNode } from '../contexts/rpcNode';
import { useBlockExplorer } from '../contexts/blockExplorer';
import { isValidHttpUrl } from '../utils/utils';
import { Select, Divider, Modal, Radio } from 'antd';
import { JetInput } from '../components/JetInput';
import { GithubFilled, TwitterCircleFilled } from '@ant-design/icons';
import { useClusterSetting } from '../contexts/clusterSetting';

export function Settings(): JSX.Element {
  const { dictionary, language, changeLanguage } = useLanguage();
  const { clusterSetting, setClusterSetting } = useClusterSetting();
  const { open, setOpen } = useSettingsModal();
  const { preferredNode, ping, updateRpcNode } = useRpcNode();
  const { blockExplorers, preferredExplorer, changePreferredExplorer } = useBlockExplorer();
  const { Option } = Select;

  // RPC node input checking
  const [rpcNodeInput, setRpcNodeInput] = useState<string>('');
  const [rpcInputError, setRpcInputError] = useState<string>('');
  function checkRPC() {
    if (!rpcNodeInput || !isValidHttpUrl(rpcNodeInput)) {
      setRpcNodeInput('');
      setRpcInputError(dictionary.settings.noUrl);
      return;
    }

    setRpcInputError('');
    setRpcNodeInput('');
    updateRpcNode(rpcNodeInput);
  }

  return (
    <Modal footer={null} visible={open} onCancel={() => setOpen(false)}>
      <div className="settings">
        <h2>{dictionary.settings.title}</h2>
        <div className="setting flex align-start justify-center column">
          <span className="setting-title bold-text">{dictionary.settings.rpcNode.toUpperCase()}</span>
          <div className="rpc-info flex align-center justify-start" style={{ padding: 'var(--spacing-xs) 0' }}>
            <span>{preferredNode ?? dictionary.settings.defaultNode}</span>
            {ping > 0 && (
              <>
                <div
                  className="ping-indicator"
                  style={{
                    background: ping < 1000 ? 'var(--success)' : 'var(--danger)'
                  }}></div>
                <span className={ping < 1000 ? 'success-text' : 'danger-text'}>({ping}ms)</span>
              </>
            )}
            {preferredNode && (
              <span className="reset-rpc gradient-text semi-bold-text" onClick={() => updateRpcNode()}>
                {dictionary.settings.reset.toUpperCase()}
              </span>
            )}
          </div>
          <JetInput
            type="text"
            value={rpcNodeInput || ''}
            error={rpcInputError}
            placeholder="ex: api.devnet.solana.com/"
            onClick={() => setRpcInputError('')}
            onChange={(value: string) => setRpcNodeInput(value.toString())}
            submit={checkRPC}
          />
        </div>
        <Divider />
        <div className="setting flex align-start justify-center column">
          <span className="setting-title bold-text">{dictionary.settings.network.toUpperCase()}</span>
          <Radio.Group
            value={clusterSetting}
            onChange={(e: any) => {
              setClusterSetting(e.target.value);
              localStorage.setItem('jetCluster', e.target.value);
            }}>
            <Radio value="mainnet-beta">{dictionary.settings.mainnet}</Radio>
            <Radio value="devnet">{dictionary.settings.devnet}</Radio>
          </Radio.Group>
        </div>
        <Divider />
        <div className="setting flex align-start justify-center column">
          <span className="setting-title bold-text">{dictionary.settings.language.toUpperCase()}</span>
          <Select value={language} onChange={value => changeLanguage(value)}>
            {Object.keys(uiDictionary).map(lang => (
              <Option key={lang} value={lang}>
                {uiDictionary[lang].language}
              </Option>
            ))}
          </Select>
        </div>
        <Divider />
        <div className="setting flex align-start justify-center column">
          <span className="setting-title bold-text">{dictionary.settings.explorer.toUpperCase()}</span>
          <Select value={blockExplorers[preferredExplorer].name} onChange={value => changePreferredExplorer(value)}>
            {Object.keys(blockExplorers).map(explorer => (
              <Option key={explorer} value={explorer}>
                {blockExplorers[explorer].name}
              </Option>
            ))}
          </Select>
        </div>
        <div className="socials flex align-center justify-start">
          <TwitterCircleFilled onClick={() => window.open('https://twitter.com/jetprotocol', '_blank', 'noopener')} />
          <GithubFilled onClick={() => window.open('https://github.com/jet-lab', '_blank', 'noopener')} />
        </div>
      </div>
    </Modal>
  );
}
