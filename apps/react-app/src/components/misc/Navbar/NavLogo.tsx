import { useChangeView } from '@utils/ui';

// Jet logo in the Navbar
export function NavLogo(): JSX.Element {
  const changeView = useChangeView();

  return (
    <img
      onClick={() => changeView('/')}
      className="nav-logo"
      src="img/jet/jet_logo.png"
      width="100%"
      height="auto"
      alt="Jet Protocol"
    />
  );
}
