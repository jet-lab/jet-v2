export const pointAtCoordinateX = (path: SVGPathElement, x: number, tolerance: number): number | undefined => {
  let length_end = path.getTotalLength(),
    length_start = 0,
    point = path.getPointAtLength((length_end + length_start) / 2), // get the middle point
    bisection_iterations_max = 40,
    bisection_iterations = 0;

  const error = 0.1;

  while (x < point.x - error || x > point.x + error) {
    point = path.getPointAtLength((length_end + length_start) / 2);
    if (x < point.x) {
      length_end = (length_start + length_end) / 2;
    } else if (x > point.x) {
      length_start = (length_start + length_end) / 2;
    }
    if (bisection_iterations_max < ++bisection_iterations) break;
  }
  if (point.x > x + tolerance || point.x < x - tolerance) {
    return undefined;
  } else {
    return point.y;
  }
};
