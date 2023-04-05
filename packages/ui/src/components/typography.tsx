interface TypographyProps {
  classNameOverride?: string;
  children?: React.ReactNode
}
// TODO change to H2 after removing antd global styles
export const Title: React.FC<TypographyProps> = ({ classNameOverride, children }) => (
  <div className={`text-3xl font-normal normal-case text-white ${classNameOverride ? classNameOverride : ''}`}>
    {children}
  </div>
);

export const SubTitle: React.FC<TypographyProps> = ({ classNameOverride, children }) => (
  <div className={`text-2xl font-normal normal-case text-white ${classNameOverride ? classNameOverride : ''}`}>
    {children}
  </div>
);


export const Paragraph: React.FC<TypographyProps> = ({ classNameOverride, children }) => (
  <div className={`font-normal normal-case text-white ${classNameOverride ? classNameOverride : ''}`}>
    {children}
  </div>
);
