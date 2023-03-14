interface ITitleProps {
    text: string
    classNameOverride?: string
}
// TODO change to H2 after removing antd global styles
export const Title = ({ classNameOverride, text }: ITitleProps) => <div className={`text-xl text-white font-normal normal-case ${classNameOverride ? classNameOverride : ''}`}>{text}</div>