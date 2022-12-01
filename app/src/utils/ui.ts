import { Connection } from '@solana/web3.js';
import { useLocation, useNavigate } from 'react-router-dom';
import { blockExplorers } from '@state/settings/settings';

// Timeout for app page / theme transitions (in ms)
export const APP_TRANSITION_TIMEOUT = 500;

// Return explorer URL for a tx based on preferred block explorer
export function getExplorerUrl(
  txId: string,
  cluster: 'mainnet-beta' | 'devnet' | 'localnet',
  explorer: 'solanaExplorer' | 'solscan' | 'solanaBeach'
) {
  const baseUrl = blockExplorers[explorer].url;

  function getClusterParam() {
    if (cluster === 'localnet') {
      return `?cluster=custom&customUrl=http%3A%2F%2Flocalhost%3A8899`;
    }
    return;
  }

  return baseUrl + txId + getClusterParam();
}

// Opens a link in a new tab
export function openLinkInBrowser(url: string) {
  window.open(url, '_blank', 'noopener');
}

// Get ping time for an endpoint
export async function getPing(endpoint: string) {
  try {
    const connection = new Connection(endpoint);
    const startTime = Date.now();
    await connection.getVersion();
    const endTime = Date.now();
    return endTime - startTime;
  } catch {
    return 0;
  }
}

// Get typography type for token values
export function getTokenStyleType(tokenAmount: number | string, showDangerIfPositive?: boolean) {
  const amount = parseFloat(tokenAmount.toString());
  if (amount > 0) {
    return showDangerIfPositive ? 'danger' : 'success';
  } else if (amount < 0) {
    return 'danger';
  } else {
    return 'secondary';
  }
}

// Animated feedback when data updates
export function animateDataUpdate(animationClass: string, elementQuery: string) {
  const elements = document.querySelectorAll<HTMLElement>(elementQuery);
  if (elements.length) {
    for (const element of elements) {
      element.classList.add('animated');
      element.classList.add(animationClass);
      setTimeout(() => element.classList.remove(animationClass), 150);
      setTimeout(() => element.classList.remove('animated'), 300);
    }
  }
}

// Change view with a transition
export function useChangeView() {
  const { pathname } = useLocation();
  const navigate = useNavigate();

  // Animate view transition and navigate to new page
  return (route: string, replace = true) => {
    // If changing to the current view, do nothing
    if (route === pathname) {
      return;
    }

    navigate(route, { replace });
  };
}

// Switch from camelCase to dash-case
export function camelToDash(input: string) {
  let value = input;
  for (let i = 0; i < value.length; i++) {
    if (value[i] === value[i].toUpperCase()) {
      const letterDash = '—' + value[i].toLowerCase();
      value = value.slice(0, i) + letterDash + value.slice(i + 1);
    }
  }
  return value;
}

// Switch form dash-case to camelCase
export function dashToCamel(input: string) {
  return input.replace(/-([a-z])/g, function (g) {
    return g[1].toUpperCase();
  });
}

// Create an array of dummy data
export function createDummyArray(size: number, idString: string) {
  const dummyArray = [];
  for (let i = 0; i < size; i++) {
    const element: any = {};
    element[idString] = Math.random().toString();
    dummyArray.push(element);
  }
  return dummyArray;
}
