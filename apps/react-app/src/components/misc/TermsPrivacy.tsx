import { useRecoilValue } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { Typography } from 'antd';

// Terms of Use, Privacy Policy, Glossary, and Audit Reports links
export function TermsPrivacy(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const { Paragraph } = Typography;

  return (
    <div className="terms-privacy flex-centered">
      <a
        href="https://docs.jetprotocol.io/jet-protocol/audit-reports"
        target="_blank"
        rel="noopener noreferrer">
        <Paragraph className="text-btn">{dictionary.termsPrivacy.audits}</Paragraph>
      </a>
      <a href="https://www.jetprotocol.io/legal/terms-of-service" target="_blank" rel="noopener noreferrer">
        <Paragraph className="text-btn">{dictionary.termsPrivacy.termsOfService}</Paragraph>
      </a>
      <a href="https://www.jetprotocol.io/legal/privacy-policy" target="_blank" rel="noopener noreferrer">
        <Paragraph className="text-btn">{dictionary.termsPrivacy.privacyPolicy}</Paragraph>
      </a>
      <a
        href="https://docs.jetprotocol.io/jet-protocol/terms-and-definitions"
        target="_blank"
        rel="noopener noreferrer">
        <Paragraph className="text-btn">{dictionary.termsPrivacy.glossary}</Paragraph>
      </a>
    </div>
  );
}
