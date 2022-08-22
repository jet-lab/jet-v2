import { useRecoilValue } from 'recoil';
import { definitions, PreferredLanguage } from '../../state/settings/localization/localization';
import { Tooltip } from 'antd';

export function Info(props: { term: string; children: JSX.Element }): JSX.Element {
  const preferredLanguage = useRecoilValue(PreferredLanguage);

  return <Tooltip title={definitions[preferredLanguage][props.term].definition}>{props.children}</Tooltip>;
}
