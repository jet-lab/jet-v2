import { useNavigate } from 'react-router-dom';
// Jet logo in the Navbar
export function NavLogo(): JSX.Element {
  const navigate = useNavigate();

  return (
    <img
      onClick={() => navigate('/')}
      className="nav-logo"
      src="img/jet/jet_logo.png"
      width="100%"
      height="auto"
      alt="Jet Protocol"
    />
  );
}
