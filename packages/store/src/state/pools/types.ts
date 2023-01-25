export interface PoolDetails {
  address: string
  symbol: string
  borrowed_tokens: number
  deposit_tokens: number
  deposit_notes: number
  accrued_until: Date
  addresses: {
    controlAuthority: string,
    depositNoteMetadata: string
    depositNoteMint: string,
    loanNoteMetadata: string
    loanNoteMint: string,
    marginPool: string
    marginPoolAdapterMetadata: string,
    tokenMetadata: string
    tokenMint: string
    vault: string
  }
}

export interface PoolsSlice {
  pools: Record<string, PoolDetails>
  updatePool: (address: string, p: Partial<PoolDetails>) => void
  setPools: (pools: Record<string, PoolDetails>) => void
}