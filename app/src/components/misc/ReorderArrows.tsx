import { animateElementOut, animateElementIn } from '../../utils/ui';
import { ReactComponent as ArrowLeft } from '../../styles/icons/arrow-left.svg';
import { ReactComponent as ArrowRight } from '../../styles/icons/arrow-right.svg';
import { ReactComponent as ArrowUp } from '../../styles/icons/arrow-up.svg';
import { ReactComponent as ArrowDown } from '../../styles/icons/arrow-down.svg';

export function ReorderArrows(props: {
  component: string;
  order: string[];
  setOrder: (order: string[]) => void;
  vertical?: boolean;
}) {
  function reorderComponent(back: boolean) {
    const nextComponent =
      props.order[back ? props.order.indexOf(props.component) - 1 : props.order.indexOf(props.component) + 1];
    animateElementOut(props.component);
    setTimeout(() => animateElementOut(nextComponent), 150);
    setTimeout(() => {
      const orderClone = JSON.parse(JSON.stringify(props.order));
      const currentPosition = orderClone.indexOf(props.component);
      let newPosition = currentPosition;
      if (back && currentPosition > 0) {
        newPosition--;
      } else if (!back && currentPosition < orderClone.length - 1) {
        newPosition++;
      }
      orderClone.splice(newPosition, 0, orderClone.splice(currentPosition, 1)[0]);
      props.setOrder(orderClone);
      animateElementIn(props.component);
      setTimeout(() => animateElementIn(nextComponent), 150);
    }, 400);
  }

  return (
    <div className="view-element-item view-element-item-hidden">
      <div
        className={`reorder-arrows ${props.vertical ? 'reorder-arrows-vertical column' : ''} ${
          props.order[0] === props.component
            ? 'reorder-arrows-up-only'
            : props.order[props.order.length - 1] === props.component
            ? 'reorder-arrows-down-only'
            : ''
        } flex align-center justify-between`}>
        {props.vertical ? (
          <ArrowUp className="jet-icon" onClick={() => reorderComponent(true)} />
        ) : (
          <ArrowLeft className="jet-icon" onClick={() => reorderComponent(true)} />
        )}
        {props.vertical ? (
          <ArrowDown className="jet-icon" onClick={() => reorderComponent(false)} />
        ) : (
          <ArrowRight className="jet-icon" onClick={() => reorderComponent(false)} />
        )}
      </div>
    </div>
  );
}
