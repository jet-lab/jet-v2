import { ScaleLinear } from 'd3-scale';

export const pointAtCoordinateX = (path: SVGPathElement, x: number) => {
  let length_end = path.getTotalLength(),
    length_start = 0,
    point = path.getPointAtLength((length_end + length_start) / 2), // get the middle point
    bisection_iterations_max = 50,
    bisection_iterations = 0;

  const error = 1;

  while (x < point.x - error || x > point.x + error) {
    point = path.getPointAtLength((length_end + length_start) / 2);
    if (x < point.x) {
      length_end = (length_start + length_end) / 2;
    } else {
      length_start = (length_start + length_end) / 2;
    }
    if (bisection_iterations_max < ++bisection_iterations) break;
  }
  return point.y;
};
