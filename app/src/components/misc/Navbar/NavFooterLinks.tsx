import { useRecoilValue } from 'recoil';
import { Dictionary } from '../../../state/settings/localization/localization';

// Footer links for the mobile navigation drawer
export function NavFooterLinks(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const mobileFooterLinks = [
    { title: dictionary.termsPrivacy.termsOfService, url: 'https://www.jetprotocol.io/legal/terms-of-service' },
    { title: dictionary.termsPrivacy.privacyPolicy, url: 'https://www.jetprotocol.io/legal/privacy-policy' },
    { title: dictionary.termsPrivacy.glossary, url: 'https://docs.jetprotocol.io/jet-protocol/terms-and-definitions' }
  ];

  const footerLinks = mobileFooterLinks.map(link => (
    <a key={link.title} href={link.url} className="footer-link" rel="noopener noreferrer" target="_blank">
      {link.title}
    </a>
  ));

  return <>{footerLinks}</>;
}
