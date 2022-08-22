import { useEffect, useState } from 'react';
import { useResetRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
import { PublicKey } from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';
import { Pool } from '@jet-lab/margin';
import { useJupiter } from '@jup-ag/react-hook';
import { Dictionary } from '../../../state/settings/localization/localization';
import { WalletTokens } from '../../../state/user/walletTokens';
import { CurrentAccountName, useAccountFromName, useAccountNames } from '../../../state/user/accounts';
import { CurrentPoolSymbol, Pools, CurrentPool } from '../../../state/borrow/pools';
import { CurrentMarketPair } from '../../../state/trade/market';
import { CurrentAction, TokenInputAmount, TokenInputString } from '../../../state/actions/actions';
import { useProvider } from '../../../utils/jet/provider';
import { formatRate } from '../../../utils/format';
import { getTokenAmountFromNumber } from '../../../utils/currency';
import { notify } from '../../../utils/notify';
import { Button, Divider, Input, Modal, Radio, Select, Skeleton, Typography } from 'antd';
import { Info } from '../../misc/Info';
import { TokenInput } from '../../misc/TokenInput';
import { ReactComponent as AngleDown } from '../../../styles/icons/arrow-angle-down.svg';
import { ReactComponent as SwapIcon } from '../../../styles/icons/function-swap.svg';
import { ReactComponent as JupiterLogo } from '../../../styles/icons/protocols/jupiter-logo.svg';

export function SwapModal(): JSX.Element {
  const { provider } = useProvider();
  const { connection } = provider;
  const dictionary = useRecoilValue(Dictionary);
  const wallet = useWallet();
  const walletTokens = useRecoilValue(WalletTokens);
  const accountNames = useAccountNames();
  const currentAccountName = useRecoilValue(CurrentAccountName);
  const [swapAccountName, setSwapAccountName] = useState<string>(currentAccountName ?? accountNames[0]);
  const swapAccount = useAccountFromName(swapAccountName);
  const setCurrentPoolSymbol = useSetRecoilState(CurrentPoolSymbol);
  const setCurrentMarketPair = useSetRecoilState(CurrentMarketPair);
  const currentPool = useRecoilValue(CurrentPool);
  const pools = useRecoilValue(Pools);
  const currentAction = useRecoilValue(CurrentAction);
  const resetCurrentAction = useResetRecoilState(CurrentAction);
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const resetTokenInputString = useResetRecoilState(TokenInputString);
  const [sendingTransaction, setSendingTransaction] = useState(false);
  const { Title, Paragraph, Text } = Typography;
  const { Option } = Select;

  // Jupiter setup
  const [outputToken, setOutputToken] = useState<Pool | undefined>(undefined);
  const [slippage, setSlippage] = useState(0.5);
  const [slippageInput, setSlippageInput] = useState('');
  const [inputGrace, setInputGrace] = useState(false);
  const [feesPaid, setFeesPaid] = useState<{ amount: number; feesToken: string }>({
    amount: 0,
    feesToken: ''
  });

  // Any time slippageInput updates, update slippage
  function getSlippageInput() {
    const slippage = parseFloat(slippageInput);
    if (!isNaN(slippage) && slippage > 0) {
      setSlippage(slippage);
    }
  }
  useEffect(getSlippageInput, [slippageInput]);

  // Instantiate the Jupiter hook, grab routes and exchange method
  const jupiter = useJupiter({
    amount: tokenInputAmount.lamports.toNumber(),
    inputMint: currentPool && new PublicKey(currentPool.address),
    outputMint: outputToken && new PublicKey(outputToken.address),
    slippage,
    debounceTime: 250
  });
  const { routes, exchange, loading } = jupiter;

  // Jupiter swap using the best route
  async function swapBestRoute() {
    setSendingTransaction(true);
    const bestRoute = routes && routes[0];
    if (!bestRoute || !swapAccount || !wallet || !wallet.signAllTransactions || !wallet.signTransaction) {
      return;
    }

    // Attempt swap with best route
    try {
      const swapResult = await exchange({
        wallet: {
          sendTransaction: wallet.sendTransaction,
          publicKey: swapAccount.address,
          signAllTransactions: wallet.signAllTransactions,
          signTransaction: wallet.signTransaction
        },
        routeInfo: bestRoute,
        onTransaction: async (txid: string) => {
          await connection.confirmTransaction(txid);
          return await connection.getTransaction(txid, {
            commitment: 'confirmed'
          });
        }
      });

      // Handle swap result
      if ('error' in swapResult) {
        console.error('Error:', swapResult.error);
        notify(
          dictionary.notifications.actions.failedTitle.replaceAll('{{ACTION}}', currentAction ?? ''),
          dictionary.notifications.actions.failedDescription
            .replaceAll('{{ACTION}}', currentAction ?? '')
            .replaceAll('{{ASSET}}', currentPool?.symbol ?? '')
            .replaceAll('{{AMOUNT}}', tokenInputAmount.uiTokens),
          'error'
        );
      } else if ('txid' in swapResult) {
        notify(
          dictionary.notifications.actions.successTitle.replaceAll('{{ACTION}}', currentAction ?? ''),
          dictionary.notifications.actions.successDescription
            .replaceAll('{{ACTION}}', currentAction ?? '')
            .replaceAll('{{ASSET}}', currentPool?.symbol ?? '')
            .replaceAll('{{AMOUNT}}', tokenInputAmount.uiTokens),
          'success'
        );
        resetTokenInputString();
        resetCurrentAction();
      }
    } catch (err) {
      console.error(err);
    }
    setSendingTransaction(false);
  }

  // Get Pool from an input/output mint
  function getPoolFromMint(mint: string): Pool | undefined {
    if (!pools) {
      return;
    }

    for (const pool of Object.values(pools.tokenPools)) {
      if (pool.address.toString() === mint) {
        return pool;
      }
    }
  }

  // Set initial outputToken
  useEffect(() => {
    if (pools && !outputToken) {
      for (const pool of Object.values(pools.tokenPools)) {
        if (pool.symbol !== currentPool?.symbol) {
          setOutputToken(pool);
        }
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentPool?.symbol, outputToken]);

  // Update feesPaid for best route
  useEffect(() => {
    if (currentAction !== 'swap') {
      return;
    }

    const bestRoute = routes && routes[0];
    let feesPaid = 0;
    let feesToken = '';
    if (!tokenInputAmount.isZero() && bestRoute) {
      bestRoute.getDepositAndFee().then(depositAndFee => {
        feesPaid += depositAndFee ? depositAndFee.signatureFee : 0;
        for (const marketInfo of bestRoute.marketInfos) {
          feesPaid += marketInfo.platformFee.amount + marketInfo.lpFee.amount;
          feesToken = getPoolFromMint(marketInfo.platformFee.mint)?.symbol ?? '—';
        }
        setFeesPaid({
          amount: feesPaid / 10 ** (outputToken?.decimals ?? 0),
          feesToken
        });
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tokenInputAmount, routes]);

  // On new tokenInputAmount, allow a grace period for Jupiter load before showing noRoutes message
  useEffect(() => {
    if (currentAction === 'swap' && !tokenInputAmount.isZero()) {
      setInputGrace(true);
      setTimeout(() => setInputGrace(false), 2500);
    }
  }, [currentAction, tokenInputAmount]);

  if (currentAction === 'swap') {
    return (
      <Modal
        visible
        className="action-modal swap-modal header-modal"
        footer={null}
        onCancel={() => {
          resetCurrentAction();
          resetTokenInputString();
        }}>
        <div className="modal-header flex-centered">
          <Title className="modal-header-title green-text">{dictionary.actions.swap.title}</Title>
        </div>
        <Text className="small-accent-text">{dictionary.common.account}</Text>
        <Select
          value={swapAccountName}
          suffixIcon={<AngleDown className="jet-icon" />}
          onChange={name => setSwapAccountName(name)}>
          {accountNames.map(name => (
            <Option key={name} value={name}>
              <Text>{name}</Text>
            </Option>
          ))}
        </Select>
        <Divider />
        <div className="swap-tokens">
          <div className="swap-section-head flex align-center justify-between">
            <Text className="small-accent-text">{dictionary.actions.swap.title.toUpperCase()}</Text>
            {currentPool && (
              <Paragraph type="secondary" italic>{`${
                currentPool && walletTokens
                  ? swapAccount
                    ? swapAccount.poolPositions[currentPool.symbol as string]?.depositBalance.tokens ?? 0
                    : walletTokens.map[currentPool.symbol as string]?.amount.tokens ?? 0
                  : 0
              } ${currentPool ? currentPool.symbol : '—'}`}</Paragraph>
            )}
          </div>
          <TokenInput
            account={swapAccount}
            onChangeToken={(tokenSymbol: string) => {
              setCurrentPoolSymbol(tokenSymbol);
              if (tokenSymbol !== 'USDC') {
                setCurrentMarketPair(`${tokenSymbol}/USDC`);
              }
            }}
            onPressEnter={swapBestRoute}
            loading={sendingTransaction}
          />
          <div className="flex-centered">
            <Button
              className="function-btn"
              shape="round"
              icon={<SwapIcon className="jet-icon" />}
              disabled={!outputToken}
              onClick={() => {
                if (outputToken?.symbol) {
                  resetTokenInputString();
                  setCurrentPoolSymbol(outputToken.symbol);
                  setOutputToken(currentPool);
                }
              }}
            />
          </div>
          <div className="swap-section-head flex align-center justify-between">
            <Text className="small-accent-text">{dictionary.actions.swap.for.toUpperCase()}</Text>
            {currentPool && (
              <Paragraph type="secondary" italic>{`${
                outputToken && walletTokens
                  ? swapAccount
                    ? swapAccount.poolPositions[outputToken.symbol as string]?.depositBalance.tokens ?? 0
                    : walletTokens.map[outputToken.symbol as string].amount.tokens ?? 0
                  : 0
              } ${outputToken ? outputToken.symbol : '—'}`}</Paragraph>
            )}
          </div>
          <TokenInput
            account={swapAccount}
            tokenSymbol={outputToken?.symbol}
            onChangeToken={(tokenSymbol: string) => {
              if (!pools) {
                return;
              }

              for (const pool of Object.values(pools.tokenPools)) {
                if (pool.symbol === tokenSymbol) {
                  setOutputToken(pool);
                }
              }
            }}
            tokenValue={getTokenAmountFromNumber(routes?.[0].outAmount ?? 0, outputToken?.decimals ?? 0)}
            loading={sendingTransaction}
            onPressEnter={swapBestRoute}
          />
        </div>
        <Divider />
        <Text className="small-accent-text ">{dictionary.actions.swap.bestRoute.toUpperCase()}</Text>
        <div className="swap-best-route-info flex align-center justify-between wrap">
          {loading ? (
            <div className="flex column">
              <Skeleton paragraph={false} active />
              <Skeleton paragraph={false} active />
            </div>
          ) : routes &&
            routes[0]?.marketInfos[0]?.amm &&
            routes[0]?.marketInfos[1]?.amm &&
            !tokenInputAmount.isZero() ? (
            <div className="flex column">
              <Paragraph>
                {routes[0].marketInfos[0].amm.label} x {routes[0].marketInfos[1].amm.label}
              </Paragraph>
              <Text italic>
                {`${getPoolFromMint(routes[0].marketInfos[0].inputMint.toString())?.symbol} > ${
                  getPoolFromMint(routes[0].marketInfos[1].inputMint.toString())?.symbol
                } > ${getPoolFromMint(routes[0].marketInfos[1].outputMint.toString())?.symbol}`}
              </Text>
            </div>
          ) : (
            <div className="flex column">
              <Paragraph>{'— x —'}</Paragraph>
              <Text italic>{'— > —'}</Text>
            </div>
          )}
          {loading ? (
            <Skeleton className="align-right" paragraph={false} active />
          ) : (
            <Title>
              {routes && !tokenInputAmount.isZero() ? routes[0].outAmount / 10 ** (outputToken?.decimals ?? 0) : '—'}{' '}
              {outputToken?.symbol}
            </Title>
          )}
        </div>
        <div className="swap-slippage flex column">
          <Info term="slippage">
            <Text className="small-accent-text info-element">{dictionary.actions.swap.slippage.toUpperCase()}</Text>
          </Info>
          <Radio.Group className="flex-centered" value={slippage} onChange={e => setSlippage(e.target.value)}>
            {[0.1, 0.5, 1].map(percentage => (
              <Radio.Button value={percentage}>{percentage}%</Radio.Button>
            ))}
            <div
              className={`swap-slippage-input flex-centered ${slippage.toString() === slippageInput ? 'active' : ''}`}
              onClick={getSlippageInput}>
              <Input
                type="string"
                placeholder="0.10"
                value={slippageInput}
                onChange={e => {
                  let inputString = e.target.value;
                  if (isNaN(+inputString) || +inputString < 0) {
                    inputString = '0';
                  }
                  setSlippageInput(inputString);
                }}
                onPressEnter={swapBestRoute}
              />
              <Text type="secondary" strong>
                %
              </Text>
            </div>
          </Radio.Group>
        </div>
        <div className="swap-info flex-centered column">
          <div className="flex align-center justify-between">
            <Text type="secondary">{dictionary.actions.swap.priceImpact}</Text>
            {loading ? (
              <Skeleton className="align-right" paragraph={false} active />
            ) : routes && !tokenInputAmount.isZero() ? (
              <Text>{formatRate(routes[0].priceImpactPct)}</Text>
            ) : (
              <Text>—</Text>
            )}
          </div>
          <div className="flex align-center justify-between">
            <Text type="secondary">{dictionary.actions.swap.minimumRecieved}</Text>
            {loading ? (
              <Skeleton className="align-right" paragraph={false} active />
            ) : routes && !tokenInputAmount.isZero() ? (
              <Text>
                {routes[0]?.marketInfos[1]?.minOutAmount ?? dictionary.common.notAvailable} {outputToken?.symbol}
              </Text>
            ) : (
              <Text>— {outputToken?.symbol}</Text>
            )}{' '}
          </div>
          <div className="flex align-center justify-between">
            <Text type="secondary">{dictionary.actions.swap.feesPaid}</Text>
            {loading ? (
              <Skeleton className="align-right" paragraph={false} active />
            ) : feesPaid.amount ? (
              <Text>
                {feesPaid.amount} {feesPaid.feesToken}
              </Text>
            ) : (
              <Text>—</Text>
            )}
          </div>
        </div>
        <div className="swap-error flex-centered">
          <Text type="danger">
            {!loading && !inputGrace && !tokenInputAmount.isZero() && currentPool && outputToken && !routes
              ? dictionary.actions.swap.errorMessages.noRoutes
                  .replaceAll('{{INPUT_TOKEN}}', currentPool?.symbol ?? '')
                  .replaceAll('{{OUTPUT_TOKEN}}', outputToken?.symbol ?? '')
              : ''}
          </Text>
        </div>
        <Button
          block
          disabled={
            loading ||
            sendingTransaction ||
            tokenInputAmount.isZero() ||
            !(currentPool && outputToken && routes && routes[0].outAmount)
          }
          loading={!tokenInputAmount.isZero() && (inputGrace || sendingTransaction)}
          onClick={swapBestRoute}>
          {sendingTransaction ? dictionary.common.sending + '..' : dictionary.actions[currentAction].title}
        </Button>
        <div className="powered-by flex-centered" onClick={() => window.open('https://jup.ag/', '_blank', 'noopener')}>
          <JupiterLogo />
          <Paragraph type="secondary">{`${dictionary.actions.swap.poweredBy} Jupiter`}</Paragraph>
        </div>
      </Modal>
    );
  } else {
    return <></>;
  }
}
