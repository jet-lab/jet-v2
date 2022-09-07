import { Connection } from '@solana/web3.js';
import { useLocation, useNavigate } from 'react-router-dom';
import { blockExplorers } from '../state/settings/settings';

// Timeout for app page / theme transitions (in ms)
export const APP_TRANSITION_TIMEOUT = 500;

// Return explorer URL for a tx based on preferred block explorer
export function getExplorerUrl(
  txId: string,
  cluster: 'mainnet-beta' | 'devnet',
  explorer: 'solanaExplorer' | 'solscan' | 'solanaBeach'
) {
  const baseUrl = blockExplorers[explorer].url;
  const clusterParam = cluster === 'devnet' ? '?cluster=devnet' : '';
  return baseUrl + txId + clusterParam;
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

    setTimeout(() => navigate(route, { replace }), APP_TRANSITION_TIMEOUT);
    animateViewOut();
  };
}

// Animate view in
export function animateViewOut() {
  const viewElements = document.querySelectorAll('.view-element');
  const viewElementItems = document.querySelectorAll('.view-element-item');
  viewElements.forEach(element => element.classList.add('view-element-hidden'));
  setTimeout(() => viewElementItems.forEach(item => item.classList.add('view-element-item-hidden')), 200);
}

// Animate view in
export function animateViewIn() {
  const viewElements = document.querySelectorAll('.view-element');
  const viewElementItems = document.querySelectorAll('.view-element-item');
  viewElements.forEach(element => element.classList.remove('view-element-hidden'));
  setTimeout(() => viewElementItems.forEach(item => item.classList.remove('view-element-item-hidden')), 200);
}

// Animate element out
export function animateElementOut(el: string) {
  const className = `.${camelToDash(el)}`;
  const elements = document.querySelectorAll(className.includes('row') ? `${className} .view-element` : className);
  const elementItems = document.querySelectorAll(`${className} .view-element-item`);
  elements.forEach(element => element.classList.add('view-element-transition-setup'));
  elements.forEach(element => element.classList.add('view-element-transitioning'));
  elementItems.forEach(item => item.classList.add('view-element-item-hidden'));
}

// Animate element in
export function animateElementIn(el: string) {
  const className = `.${camelToDash(el)}`;
  const elements = document.querySelectorAll(className.includes('row') ? `${className} .view-element` : className);
  const elementItems = document.querySelectorAll(`${className} .view-element-item`);
  setTimeout(() => elements.forEach(element => element.classList.remove('view-element-transition-setup')), 200);
  elements.forEach(element => element.classList.remove('view-element-transitioning'));
  elementItems.forEach(item => item.classList.remove('view-element-item-hidden'));
}

// Switch from camelCase to dash-case
export function camelToDash(input: string) {
  let value = input;
  for (let i = 0; i < value.length; i++) {
    if (value[i] === value[i].toUpperCase()) {
      const letterDash = '-' + value[i].toLowerCase();
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

// Sleep function
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// Light / dark with transition maintenance
export function toggleLightTheme(lightTheme: boolean) {
  const allElements = document.querySelectorAll<HTMLElement>('body *');
  const affectedElements: any = [];
  allElements.forEach(element => {
    const transitionProperty = window.getComputedStyle(element).transitionProperty;
    if (
      transitionProperty.includes('color') ||
      transitionProperty.includes('background-color') ||
      transitionProperty.includes('fill') ||
      transitionProperty.includes('box-shadow')
    ) {
      const transition = element.style.transition;
      element.style.transition = 'unset';
      affectedElements.push({ element, transition });
    }
  });
  [
    'primary',
    'primary-2',
    'primary-3',
    'primary-4',
    'secondary',
    'secondary-2',
    'secondary-3',
    'secondary-4',
    'secondary-5',
    'light-shadow',
    'dark-shadow',
    'drop-shadow',
    'header-background',
    'jet-green',
    'jet-green-2',
    'jet-green-3',
    'jet-blue',
    'jet-blue-2',
    'success',
    'warning',
    'danger'
  ].forEach(color => {
    document.documentElement.style.setProperty(`--${color}`, `var(--${lightTheme ? 'lt' : 'dt'}-${color})`);
  });
  setTimeout(() => {
    affectedElements.forEach((el: any) => (el.element.style.transition = el.transition));
  }, APP_TRANSITION_TIMEOUT);
}
