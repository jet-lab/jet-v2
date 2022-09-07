import { useEffect, useState } from 'react';
import { useResetRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { Pool } from '@jet-lab/margin';
import { TOKEN_LIST_URL, useJupiter } from '@jup-ag/react-hook';
import { Cluster } from '../../../state/settings/settings';
import { Dictionary } from '../../../state/settings/localization/localization';
import { WalletTokens } from '../../../state/user/walletTokens';
import { CurrentPoolSymbol, Pools, CurrentPool } from '../../../state/borrow/pools';
import { CurrentMarketPair } from '../../../state/trade/market';
import { CurrentAction, TokenInputAmount, TokenInputString } from '../../../state/actions/actions';
import { formatRate } from '../../../utils/format';
import { getTokenAmountFromNumber } from '../../../utils/currency';
import { notify } from '../../../utils/notify';
import { Button, Divider, Input, Modal, Radio, Skeleton, Typography } from 'antd';
import { Info } from '../../misc/Info';
import { TokenInput } from '../../misc/TokenInput/TokenInput';
import { ReactComponent as SwapIcon } from '../../../styles/icons/function-swap.svg';
import { ReactComponent as JupiterLogo } from '../../../styles/icons/protocols/jupiter-logo.svg';

export function JupiterModal(): JSX.Element {
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const wallet = useWallet();
  const walletTokens = useRecoilValue(WalletTokens);
  const setCurrentPoolSymbol = useSetRecoilState(CurrentPoolSymbol);
  const setCurrentMarketPair = useSetRecoilState(CurrentMarketPair);
  const currentPool = useRecoilValue(CurrentPool);
  const pools = useRecoilValue(Pools);
  const currentAction = useRecoilValue(CurrentAction);
  const resetCurrentAction = useResetRecoilState(CurrentAction);
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const resetTokenInputString = useResetRecoilState(TokenInputString);
  const resetTokenInputAmount = useResetRecoilState(TokenInputAmount);
  const [sendingTransaction, setSendingTransaction] = useState(false);
  const { Title, Paragraph, Text } = Typography;

  // Jupiter setup
  const [jupiterTokens, setJupiterTokens] = useState<{ mint: PublicKey; symbol: string }[]>([]);
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
    inputMint: currentPool && currentPool.addresses.tokenMint,
    outputMint: outputToken && outputToken.addresses.tokenMint,
    slippage,
    debounceTime: 250
  });
  const { routes, exchange, loading } = jupiter;

  // Instantiate token list
  useEffect(() => {
    fetch(TOKEN_LIST_URL[cluster])
      .then(resp => resp.json())
      .then(tokens => {
        const jupiterTokens: { mint: PublicKey; symbol: string }[] = [];
        tokens.forEach((token: any) => {
          jupiterTokens.push({
            mint: new PublicKey(token.address),
            symbol: token.symbol
          });
        });
        setJupiterTokens(jupiterTokens);
      });
  }, [cluster]);

  // Jupiter swap using the best route
  async function swapBestRoute() {
    setSendingTransaction(true);
    const bestRoute = routes && routes[0];
    if (!bestRoute || !wallet || !wallet.signAllTransactions || !wallet.signTransaction) {
      return;
    }

    // Attempt swap with best route
    try {
      const swapResult = await exchange({
        wallet: {
          sendTransaction: wallet.sendTransaction,
          publicKey: wallet.publicKey,
          signAllTransactions: wallet.signAllTransactions,
          signTransaction: wallet.signTransaction
        },
        routeInfo: bestRoute
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

  // Get token symbol from an input/output mint
  function getSymbolFromMint(mint: string | PublicKey): string | undefined {
    const tokenMatch = jupiterTokens.filter(token => token.mint.toString() === mint.toString())[0];
    if (tokenMatch) {
      return tokenMatch.symbol;
    }
  }

  // Set initial outputToken
  useEffect(() => {
    if (pools && !outputToken) {
      if (currentPool?.symbol !== 'USDC') {
        setOutputToken(Object.values(pools.tokenPools).filter(pool => pool.symbol === 'USDC')[0]);
      } else {
        const poolMatch = Object.values(pools.tokenPools).filter(pool => pool.symbol !== currentPool?.symbol)[0];
        if (poolMatch) {
          setOutputToken(poolMatch);
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
          feesToken = getSymbolFromMint(marketInfo.platformFee.mint) ?? '—';
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

  // Renders the wallet balance for current pool token
  function renderInputWalletBalance() {
    let render = <></>;
    if (walletTokens && currentPool) {
      const walletBalance = walletTokens.map[currentPool.symbol].amount.tokens + ' ' + currentPool.symbol;
      render = (
        <Paragraph type="secondary" italic>
          {walletBalance}
        </Paragraph>
      );
    }

    return render;
  }

  // Renders the wallet balance for output pool token
  function renderOutputWalletBalance() {
    let render = <></>;
    if (walletTokens && outputToken) {
      const walletBalance = walletTokens.map[outputToken.symbol].amount.tokens + ' ' + outputToken.symbol;
      render = (
        <Paragraph type="secondary" italic>
          {walletBalance}
        </Paragraph>
      );
    }

    return render;
  }

  // Returns the inner text for the submit button
  function getSubmitText() {
    let text = dictionary.actions.swap.title;
    if (sendingTransaction) {
      text = dictionary.common.sending + '..';
    }

    return text;
  }

  // Handle user closing the modal
  function handleCancel() {
    // Don't close if we're sending a tx
    if (sendingTransaction) {
      return;
    }

    // Close modal and reset tokenInput
    resetCurrentAction();
    resetTokenInputString();
    resetTokenInputAmount();
  }

  if (currentAction === 'swap') {
    return (
      <Modal visible className="action-modal swap-panel header-modal" footer={null} onCancel={handleCancel}>
        <div className="modal-header flex-centered">
          <Title className="modal-header-title green-text">{dictionary.actions.swap.title}</Title>
        </div>
        <div className="swap-tokens">
          <div className="swap-section-head flex align-center justify-between">
            <Text className="small-accent-text">{dictionary.actions.swap.title.toUpperCase()}</Text>
            {renderInputWalletBalance()}
          </div>
          <TokenInput
            account={undefined}
            onChangeToken={(tokenSymbol: string) => {
              setCurrentPoolSymbol(tokenSymbol);
              // If we're not switching to USDC, also update the currentMarketPair
              if (tokenSymbol !== 'USDC') {
                setCurrentMarketPair(`${tokenSymbol}/USDC`);
              }
            }}
            onPressEnter={swapBestRoute}
            loading={sendingTransaction}
          />
          <div className="flex-centered">
            <Button
              className="function-btn swap-assets"
              shape="round"
              icon={<SwapIcon className="jet-icon" />}
              disabled={!outputToken}
              onClick={() => {
                if (outputToken) {
                  resetTokenInputString();
                  setCurrentPoolSymbol(outputToken.symbol);
                  setOutputToken(currentPool);
                }
              }}
            />
          </div>
          <div className="swap-section-head flex align-center justify-between">
            <Text className="small-accent-text">{dictionary.actions.swap.for.toUpperCase()}</Text>
            {renderOutputWalletBalance()}
          </div>
          <TokenInput
            account={undefined}
            tokenSymbol={outputToken ? outputToken.symbol : undefined}
            onChangeToken={(tokenSymbol: string) => {
              if (!pools) {
                return;
              }

              const poolMatch = Object.values(pools.tokenPools).filter(pool => pool.symbol === tokenSymbol)[0];
              if (poolMatch) {
                setOutputToken(poolMatch);
              }
            }}
            tokenValue={getTokenAmountFromNumber(
              routes ? routes[0].outAmount / 10 ** (outputToken?.decimals ?? 0) : 0,
              outputToken?.decimals ?? 0
            )}
            onPressEnter={swapBestRoute}
            loading={sendingTransaction}
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
          ) : routes && routes[0]?.marketInfos.length && !tokenInputAmount.isZero() ? (
            <div className="flex column">
              {routes[0].marketInfos.length < 2 ? (
                <>
                  <Paragraph>{routes[0].marketInfos[0].amm.label}</Paragraph>
                  <Text italic>{getSymbolFromMint(routes[0].marketInfos[0].inputMint)}</Text>
                </>
              ) : (
                <>
                  <Paragraph>
                    {routes[0].marketInfos[0].amm.label} x {routes[0].marketInfos[1].amm.label}
                  </Paragraph>
                  <Text italic>
                    {`${getSymbolFromMint(routes[0].marketInfos[0].inputMint)} > ${getSymbolFromMint(
                      routes[0].marketInfos[1].inputMint
                    )} > ${getSymbolFromMint(routes[0].marketInfos[1].outputMint)}`}
                  </Text>
                </>
              )}
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
              <Radio.Button key={percentage} value={percentage}>
                {percentage}%
              </Radio.Button>
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
            {!loading && !inputGrace && !tokenInputAmount.isZero() && outputToken && !routes
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
          {getSubmitText()}
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
