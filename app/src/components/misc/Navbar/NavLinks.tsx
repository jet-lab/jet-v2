import { useLocation } from 'react-router-dom';
import { useRecoilValue } from 'recoil';
import { Dictionary } from '../../../state/settings/localization/localization';
import { useChangeView } from '../../../utils/ui';
import { Tooltip, Typography } from 'antd';

type Route = '/' | '/trade' | '/swaps' | '/accounts';
interface Link {
  title: string;
  route: Route;
  disabled: boolean;
}

// All navigation links
export function NavLinks(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const navLinks: Link[] = [
    { title: dictionary.tradeView.title, route: '/trade', disabled: true },
    { title: dictionary.poolsView.title, route: '/', disabled: false },
    { title: dictionary.swapsView.title, route: '/swaps', disabled: false },
    { title: dictionary.accountsView.title, route: '/accounts', disabled: false }
  ];

  const navLinkComponents = navLinks.map(link => {
    let navLink = NavLink(link);

    // If link is disabled, wrap in a tooltip for "coming soon" text
    if (link.disabled) {
      navLink = (
        <Tooltip key={link.title} title={dictionary.common.comingSoon}>
          {navLink}
        </Tooltip>
      );
    }

    return navLink;
  });

  return <>{navLinkComponents}</>;
}

// One navigation link
function NavLink(link: Link): JSX.Element {
  const { pathname } = useLocation();
  const changeView = useChangeView();
  const { Text } = Typography;

  return (
    <Text
      key={link.title}
      disabled={link.disabled}
      onClick={() => (!link.disabled ? changeView(link.route) : null)}
      className={`nav-link ${pathname === link.route ? 'active' : ''}`}>
      {link.title}
    </Text>
  );
}
