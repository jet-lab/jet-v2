import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router';
import { useRecoilState, useRecoilValue } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import { CurrentPath } from '../../state/views/views';
import { DisclaimersAccepted } from '../../state/settings/settings';
import { Dictionary } from '../../state/settings/localization/localization';
import { WalkthroughModal as WalkthroughModalState, WalkthroughCompleted } from '../../state/modals/modals';
import { sleep } from '../../utils/ui';
import { camelToDash } from '../../utils/ui';
import { Typography, Button, Divider } from 'antd';
import { ReactComponent as Copilot } from '../../styles/icons/copilot/copilot.svg';
import { ReactComponent as JetPlane } from '../../styles/icons/jet/jet_plane.svg';

export function WalkthroughModal(): JSX.Element {
  const navigate = useNavigate();
  const [currentPath, setCurrentPath] = useRecoilState(CurrentPath);
  const disclaimersAccepted = useRecoilValue(DisclaimersAccepted);
  const dictionary = useRecoilValue(Dictionary);
  const { publicKey } = useWallet();
  const [walkthroughModalOpen, setWalkthroughModalOpen] = useRecoilState(WalkthroughModalState);
  const [walkthroughCompleted, setWalkthroughCompleted] = useRecoilState(WalkthroughCompleted);
  const [currentSection, setCurrentSection] = useState<number>(0);
  const [currentSectionElement, setCurrentSectionElement] = useState<
    { element: HTMLElement; zIndex: string; boxShadow: string } | undefined
  >();
  const [offset, setOffset] = useState<{ x: number | string; y: number | string } | undefined>();
  const { Paragraph, Text } = Typography;
  const sections = [
    'welcome',
    'aboutJet',
    'walletButton',
    'accountSnapshot',
    'pairSelector',
    'orderbook',
    'recentTrades',
    'orderEntry',
    'candleStickChart',
    'pairRelatedAccount',
    'poolsTable',
    'poolDetail',
    'radar',
    'fullAccountHistory',
    'fullAccountBalance',
    'settingsBtn',
    'notificationsBtn',
    'end'
  ];

  // Continue to the next section of walkthrough, highlight elements if applicable
  async function nextSection() {
    if (!walkthroughModalOpen) {
      return;
    }

    // Prep the next section index
    const nextSection = currentSection + 1;
    setCurrentSection(nextSection);

    // Reset styling if there's a currentSectionElement
    if (currentSectionElement) {
      currentSectionElement.element.style.zIndex = currentSectionElement.zIndex;
      currentSectionElement.element.style.boxShadow = currentSectionElement.boxShadow;
    }

    // Change the page if need be
    if (nextSection > 16 && currentPath !== '/') {
      await changePage('/');
    } else if (nextSection > 12 && nextSection < 17 && currentPath !== '/accounts') {
      await changePage('/accounts');
    } else if (nextSection > 9 && nextSection < 13 && currentPath !== '/pools') {
      await changePage('/pools');
    } else if (nextSection > 2 && nextSection < 10 && currentPath !== '/') {
      await changePage('/');
    }

    // Grab nextSectionElement and style
    const nextElementClassName = `.${camelToDash(sections[nextSection])}`;
    const nextSectionElement = document.querySelector<HTMLElement>(nextElementClassName);
    if (nextSectionElement) {
      setCurrentSectionElement({
        element: nextSectionElement,
        zIndex: nextSectionElement.style.zIndex,
        boxShadow: nextSectionElement.style.boxShadow
      });
      nextSectionElement.style.boxShadow = 'unset';
      setTimeout(() => {
        nextSectionElement.style.zIndex = '101';
      }, 300);
      const rect = nextSectionElement.getBoundingClientRect();
      setOffset({
        x:
          rect.width > 500
            ? rect.left
            : rect.left > window.innerWidth / 3
            ? rect.left - 310
            : rect.left + rect.width + 10,
        y:
          rect.width > 500
            ? rect.top + window.scrollY > document.body.offsetHeight - rect.height * 2
              ? rect.top +
                window.scrollY -
                (nextSection === 8 || nextSection === 9 || nextSection === 13
                  ? window.innerHeight > 1000
                    ? 350
                    : 260
                  : window.innerHeight > 1000
                  ? 320
                  : 220)
              : rect.top + window.scrollY + 10 + rect.height
            : rect.top + window.scrollY + 10
      });

      // Scroll element into view
      document.body.style.overflowY = 'scroll';
      nextSectionElement.scrollIntoView({
        block: nextSection < 8 ? 'center' : nextSection === 8 && window.innerHeight < 1000 ? 'end' : undefined
      });
      document.body.style.overflowY = 'hidden';
    } else if (nextSection === sections.length - 1) {
      setOffset({
        x: '50%',
        y: '50%'
      });
    }
  }

  // Change page
  async function changePage(path: string) {
    window.scroll(0, 0);
    const navbar = document.querySelector<HTMLElement>('.navbar');
    const walkthroughModal = document.querySelector<HTMLElement>('.walkthrough-modal');
    if (navbar && walkthroughModal) {
      walkthroughModal.style.opacity = '0';
      const navZIndex = navbar.style.zIndex;
      navbar.style.zIndex = '101';
      await sleep(500);
      navigate(path, { replace: true });
      setCurrentPath(path);
      await sleep(500);
      navbar.style.zIndex = navZIndex;
      walkthroughModal.style.opacity = '1';
    }
  }

  // Optional modal body content
  const extraDescription = (): JSX.Element => {
    // @ts-ignore
    const extraDescription = dictionary.copilot.walkthrough[sections[currentSection]]?.extraDescription;
    if (extraDescription) {
      return <Text className="walkthrough-modal-body-description">{extraDescription}</Text>;
    } else {
      return <></>;
    }
  };
  const detail = (): JSX.Element => {
    // @ts-ignore
    const detail = dictionary.copilot.walkthrough[sections[currentSection]]?.detail;
    if (detail) {
      return <Text className="walkthrough-modal-body-detail">{detail}</Text>;
    } else {
      return <></>;
    }
  };
  const callToAction = (): JSX.Element => {
    // @ts-ignore
    const callToAction = dictionary.copilot.walkthrough[sections[currentSection]]?.callToAction;
    if (callToAction) {
      return (
        <Text className="walkthrough-modal-body-cta" italic>
          {callToAction}
        </Text>
      );
    } else {
      return <></>;
    }
  };
  const nextButton = (): JSX.Element => {
    // @ts-ignore
    const nextButtonText = dictionary.copilot.walkthrough[sections[currentSection]]?.nextButtonText;
    if (nextButtonText) {
      return (
        <Button
          className="walkthrough-modal-btn"
          onClick={() => (currentSection === sections.length - 1 ? setWalkthroughCompleted(true) : nextSection())}>
          {nextButtonText}
          <JetPlane className="jet-icon jet-plane" />
        </Button>
      );
    } else {
      return <></>;
    }
  };
  const altButton = (): JSX.Element => {
    // @ts-ignore
    const altButtonText = dictionary.copilot.walkthrough[sections[currentSection]]?.altButtonText;
    if (altButtonText) {
      return (
        <Button
          type="link"
          onClick={() => {
            if (currentSection === 2) {
              window.open('https://phantom.app/', '_blank', 'noopener');
            } else {
              window.open('https://docs.jetprotocol.io/jet-protocol/', '_blank', 'noopener');
            }
          }}>
          {altButtonText}
        </Button>
      );
    } else {
      return <></>;
    }
  };

  // Init walkthrough if it hasn't already been completed
  useEffect(() => {
    if (publicKey && disclaimersAccepted[publicKey.toBase58()] && !walkthroughCompleted && window.screen.width > 800) {
      window.scrollTo(0, 0);
      setWalkthroughModalOpen(true);
    } else {
      setWalkthroughModalOpen(false);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [disclaimersAccepted, walkthroughCompleted]);

  if (walkthroughModalOpen) {
    return (
      <div className="walkthrough-modal-wrapper">
        <div
          className="walkthrough-modal flex align-center justify-start"
          style={
            offset
              ? {
                  position: currentSection === sections.length - 1 ? 'fixed' : 'absolute',
                  transform: currentSection === sections.length - 1 ? 'translate(-50%, -50%)' : 'unset',
                  left: offset.x,
                  top: offset.y
                }
              : undefined
          }>
          <div className="walkthrough-modal-copilot flex-centered">
            <Copilot className="jet-icon" />
          </div>
          <div className="walkthrough-modal-body flex align-start justify-center column">
            <Paragraph className="walkthrough-modal-body-title" italic strong>
              {/* @ts-ignore */}
              {dictionary.copilot.walkthrough[sections[currentSection]]?.title}
            </Paragraph>
            <Divider />
            <Text className="walkthrough-modal-body-description">
              {/* @ts-ignore */}
              {dictionary.copilot.walkthrough[sections[currentSection]]?.description}
            </Text>
            {extraDescription()}
            {detail()}
            {callToAction()}
            {nextButton()}
            {altButton()}
          </div>
          <span
            className="walkthrough-modal-close"
            onClick={() => {
              setWalkthroughCompleted(true);
              if (currentSectionElement) {
                currentSectionElement.element.style.zIndex = currentSectionElement.zIndex;
                currentSectionElement.element.style.boxShadow = currentSectionElement.boxShadow;
              }
            }}></span>
        </div>
      </div>
    );
  } else {
    return <></>;
  }
}
