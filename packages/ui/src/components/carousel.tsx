import { useCallback, useMemo, useState } from 'react';

interface ICarouselProps {
  pagesToRender: number
  pages: (args: {
    pageNumber: number;
    pageIndex: number
    isEndPage: boolean;
    isFirstPage: boolean;
    nextPage: () => void;
    previousPage: () => void;
  }) => JSX.Element[];
}

/**
 * A Carousel component, receives an array of pages under the `pages` prop which also comes with render props for nextPage, previousPage, current page etc..
 * Sample usage:
 * ```
 * <Carousel pages={({ nextPage, previousPage }) => [<Comp1 key="page-1" previousPage={previousPage} nextPage={nextPage} />, <Comp2 key="page-2" previousPage={previousPage} nextPage={nextPage} />]} />
 * ```
 */

export const Carousel = ({ pages, pagesToRender }: ICarouselProps) => {
  const [page, setPage] = useState(0);

  const nextPage = useCallback(() => {
    if (page < pagesToRender) {
      setPage(page + 1);
    } else {
      console.log('reached last page');
    }
  }, [page, pages]);

  const previousPage = useCallback(() => {
    if (page > 0) {
      setPage(page - 1);
    }
  }, [page]);

  const isFirstPage = useMemo(() => page === 0, [page]);
  const isEndPage = useMemo(() => page === pagesToRender - 1, [page]);

  const enhancedPages = pages({
    pageNumber: page + 1,
    pageIndex: page,
    isEndPage,
    isFirstPage,
    nextPage,
    previousPage
  });

  return <div>{enhancedPages.filter((c, index) => (index === page ? c : null))}</div>;
};
