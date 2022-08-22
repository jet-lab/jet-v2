// Manual timeout promise to pause program execution
export function timeout(ms: number): Promise<unknown> {
  return new Promise(res => {
    setTimeout(() => res(true), ms);
  });
}

// Shorten a pubkey with ellipses
export function shortenPubkey(pubkey: string, halfLength = 4): string {
  return `${pubkey.substring(0, halfLength)}...${pubkey.substring(pubkey.length - halfLength)}`;
}

// Check if a string is a valid URL
export function isValidHttpUrl(input: string): boolean {
  let url;
  try {
    url = new URL(input);
  } catch (_) {
    return false;
  }

  return url.protocol === 'http:' || url.protocol === 'https:';
}
