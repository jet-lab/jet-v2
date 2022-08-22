import { definitions, useLanguage } from '../contexts/localization/localization';
import { Tooltip } from 'antd';
import { QuestionCircleFilled } from '@ant-design/icons';

export function Info(props: { term: string }): JSX.Element {
  const { language } = useLanguage();

  return (
    <Tooltip title={definitions[language][props.term].definition}>
      <QuestionCircleFilled className="info" />
    </Tooltip>
  );
}
