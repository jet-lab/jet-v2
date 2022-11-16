import { useRecoilState } from 'recoil';
import { TpsBanner } from '../TpsBanner';
import { PauseBorrowBanner } from '../PauseBorrowBanner';
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
      <TpsBanner />
      {/* To display a banner showing borrows as paused, uncomment the proceeding line and edit 'message' in PauseBorrowBanner.tsx */}
      <PauseBorrowBanner /> 
      
      {/* Desktop Nav */}
      <nav className="desktop flex align-center justify-between">
        <div className="nav-section flex align-center justify-start">
          <NavLogo />
        </div>
        <div className="nav-section flex-centered">
          <NavLinks />
        </div>
        <div className="nav-section flex align-center justify-end">
          <NavButtons showWalletButton />
        </div>
      </nav>
      {/* Mobile Nav */}
      <nav className="mobile flex align-center justify-between">
        <NavButtons />
        <NavLogo />
        <div
          className={`hamburger flex align-center justify-between column ${drawerOpen ? 'close' : ''}`}
          onClick={() => setDrawerOpen(!drawerOpen)}>
          <span></span>
          <span></span>
          <span></span>
        </div>
        <div className="drawer flex align-center justify-between column">
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
