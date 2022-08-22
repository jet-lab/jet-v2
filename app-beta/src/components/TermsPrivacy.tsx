import { useLanguage } from '../contexts/localization/localization';

export function TermsPrivacy(): JSX.Element {
  const { dictionary } = useLanguage();

  return (
    <div className="terms-privacy flex-centered">
      <a href="https://www.jetprotocol.io/legal/terms-of-service" target="_blank" rel="noopener noreferrer">
        <span className="text-btn">{dictionary.termsPrivacy.termsOfService}</span>
      </a>
      <a href="https://www.jetprotocol.io/legal/privacy-policy" target="_blank" rel="noopener noreferrer">
        <span className="text-btn">{dictionary.termsPrivacy.privacyPolicy}</span>
      </a>
      <a
        href="https://docs.jetprotocol.io/jet-protocol/terms-and-definitions"
        target="_blank"
        rel="noopener noreferrer">
        <span className="text-btn">{dictionary.termsPrivacy.glossary}</span>
      </a>
      <a href="https://v1.jetprotocol.io/" target="_blank" rel="noopener noreferrer">
        <span className="text-btn">Jet V1</span>
      </a>
    </div>
  );
}
