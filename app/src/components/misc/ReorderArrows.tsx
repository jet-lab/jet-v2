import ArrowLeft from '../../assets/icons/arrow-left.svg';
import ArrowRight from '../../assets/icons/arrow-right.svg';
import ArrowUp from '../../assets/icons/arrow-up.svg';
import ArrowDown from '../../assets/icons/arrow-down.svg';

// Arrow components for user to customize their UI
export function ReorderArrows(props: {
  // Component key so we know what component to reorder
  component: string;
  // The global order state we're dealing with
  order: string[];
  // The reordering method for the global state
  setOrder: (order: string[]) => void;
  // Whether we want to present the arrows vertically (if component is full-width)
  vertical?: boolean;
}) {
  // Updates global state
  function reorderComponent(backInOrder: boolean) {
    // Clone the global order of components
    const orderClone = JSON.parse(JSON.stringify(props.order));
    // Find the component's current position in array
    const currentPosition = orderClone.indexOf(props.component);
    let newPosition = currentPosition;
    // If moving back in the order, decrement the position
    if (backInOrder && currentPosition > 0) {
      newPosition--;
      // If moving forward in the order, increment the position
    } else if (!backInOrder && currentPosition < orderClone.length - 1) {
      newPosition++;
    }
    // Reorder the array based on the new position
    orderClone.splice(newPosition, 0, orderClone.splice(currentPosition, 1)[0]);
    // Update the global state with new order
    props.setOrder(orderClone);
  }

  // Return whether a component is at the beginning or end of array (and only show the arrow they can move from)
  function getHiddenArrowClass() {
    let className = '';
    // If component can only increase its position in array
    if (props.order[0] === props.component) {
      className = 'reorder-arrows-up-only';
      // If component can only decrease its position in array
    } else if (props.order[props.order.length - 1] === props.component) {
      className = 'reorder-arrows-down-only';
    }

    return className;
  }

  // Render the arrow to DECREASE component's position in the global array
  function renderDecreaseArrow() {
    const reorderBackwards = () => reorderComponent(true);
    let render = <ArrowLeft className="jet-icon" onClick={reorderBackwards} />;
    // If we want to present arrow vertically
    if (props.vertical) {
      render = <ArrowUp className="jet-icon" onClick={() => reorderComponent(true)} />;
    }

    return render;
  }

  // Render the arrow to INCREASE component's position in the global array
  function renderIncreaseArrow() {
    const reorderForwards = () => reorderComponent(false);
    let render = <ArrowRight className="jet-icon" onClick={reorderForwards} />;
    // If we want to present arrow vertically
    if (props.vertical) {
      render = <ArrowDown className="jet-icon" onClick={reorderForwards} />;
    }

    return render;
  }

  return (
    <div
      className={`reorder-arrows ${
        props.vertical ? 'reorder-arrows-vertical column' : ''
      } ${getHiddenArrowClass()} flex align-center justify-between`}>
      {renderDecreaseArrow()}
      {renderIncreaseArrow()}
    </div>
  );
}
