import { useRecoilState } from 'recoil';
import { TpsBanner } from '../TpsBanner';
import { Banner } from '../Banner';
import { WalletButton } from '../WalletButton';
import { NavLogo } from './NavLogo';
import { NavLinks } from './NavLinks';
import { NavButtons } from './NavButtons';
import { NavFooterLinks } from './NavFooterLinks';
import { NavDrawerOpen } from '@state/views/views';

// The Navigation Bar for the application
export function Navbar(): JSX.Element {
  const [drawerOpen, setDrawerOpen] = useRecoilState(NavDrawerOpen);
  return (
    <div className={`navbar-container flex-centered column ${drawerOpen ? 'drawer-open' : ''}`}>
      <Banner
        message={
          <p>
            Jet Protocol is shutting down.{' '}
            <u>
              <a href="https://forum.jetprotocol.io/t/community-update-jet-protocol-holdings-llc-is-shutting-down/1560" target="_blank"> Visit the forum</a>
            </u>{' '}
            for more detail.
          </p>
        }
      />
      <TpsBanner />
      {/* Desktop Nav */}
      <nav className="desktop align-center flex justify-between">
        <div className="nav-section align-center flex justify-start">
          <NavLogo />
        </div>
        <div className="nav-section flex-centered">
          <NavLinks />
        </div>
        <div className="nav-section align-center flex justify-end">
          <NavButtons showWalletButton />
        </div>
      </nav>
      {/* Mobile Nav */}
      <nav className="mobile align-center flex justify-between">
        <NavButtons />
        <NavLogo />
        <div
          className={`hamburger align-center column flex justify-between ${drawerOpen ? 'close' : ''}`}
          onClick={() => setDrawerOpen(!drawerOpen)}>
          <span></span>
          <span></span>
          <span></span>
        </div>
        <div className="drawer align-center column flex justify-between">
          <div className="drawer-top flex-centered column">
            <NavLinks />
            <WalletButton mobile />
          </div>
          <div className="drawer-bottom flex-centered column">
            <NavFooterLinks />
          </div>
        </div>
      </nav>
    </div>
  );
}
