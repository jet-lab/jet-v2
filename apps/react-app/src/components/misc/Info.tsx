import { useRecoilValue } from 'recoil';
import { definitions, PreferredLanguage } from '@state/settings/localization/localization';
import { Tooltip } from 'antd';

// Info icon with a definition tooltip
export function Info(props: { term: string; children: JSX.Element }): JSX.Element {
  const preferredLanguage = useRecoilValue(PreferredLanguage);
  const termDefinition = definitions[preferredLanguage][props.term].definition;

  return <Tooltip title={termDefinition}>{props.children}</Tooltip>;
}
