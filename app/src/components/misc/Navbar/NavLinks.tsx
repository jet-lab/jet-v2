import { useLocation } from 'react-router-dom';
import { useRecoilValue, useSetRecoilState } from 'recoil';
import { Dictionary } from '../../../state/settings/localization/localization';
import { NavDrawerOpen } from '../../../state/views/views';
import { SendingTransaction } from '../../../state/actions/actions';
import { useChangeView } from '../../../utils/ui';
import { Tooltip, Typography } from 'antd';
import { isDebug } from '../../../App';

type Route =
  | '/'
  | '/swaps'
  | '/accounts'
  | '/fixed-lend?debug-environment=true'
  | '/fixed-borrow?debug-environment=true';
interface Link {
  title: string;
  route: Route;
  disabled: boolean;
}

// All navigation links
export function NavLinks(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const navLinks: Link[] = [
    { title: dictionary.poolsView.title, route: '/', disabled: false },
    { title: dictionary.swapsView.title, route: '/swaps', disabled: false },
    { title: dictionary.accountsView.title, route: '/accounts', disabled: false }
  ];

  if (isDebug) {
    navLinks.push({ title: 'Lend', route: '/fixed-lend?debug-environment=true', disabled: false });
    navLinks.push({ title: 'Borrow', route: '/fixed-borrow?debug-environment=true', disabled: false });
  }

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
  const sendingTransaction = useRecoilValue(SendingTransaction);
  const { pathname } = useLocation();
  const changeView = useChangeView();
  const setDrawerOpen = useSetRecoilState(NavDrawerOpen);
  const { Text } = Typography;

  return (
    <Text
      key={link.title}
      disabled={link.disabled}
      onClick={() => {
        if (!link.disabled && !sendingTransaction) {
          changeView(link.route);
          setDrawerOpen(false);
        }
      }}
      className={`nav-link ${pathname === link.route ? 'active' : ''}`}>
      {link.title}
    </Text>
  );
}
