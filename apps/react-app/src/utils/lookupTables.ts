import { useJetStore } from '@jet-lab/store';
import axios from 'axios';

export async function getAuthorityLookupTables(authority: string): Promise<LookupTable[]> {
  const { cluster } = useJetStore.getState().settings;
  const apiEndpoint =
    cluster === 'mainnet-beta'
      ? process.env.REACT_APP_DATA_API
      : cluster === 'devnet'
        ? process.env.REACT_APP_DEV_DATA_API
        : cluster === 'localnet'
          ? process.env.REACT_APP_LOCAL_DATA_API
          : undefined;
  const data = (await axios.get<{
    authority: string,
    tables: {
      address: string,
      data: number[]
    }[]
  }>(`${apiEndpoint || 'http://localhost:3002'}/lookup/authority_addresses/${authority}`)).data;
  return data.tables.map(address => {
    return {
      address: address.address,
      data: Uint8Array.from(address.data)
    }
  })
}

export async function checkUpgradeLookupRegistry(airspace: string, authority: string, payer: string): Promise<LookupRegistryInstructions> {
  const { cluster } = useJetStore.getState().settings;
  const apiEndpoint =
    cluster === 'mainnet-beta'
      ? process.env.REACT_APP_DATA_API
      : cluster === 'devnet'
        ? process.env.REACT_APP_DEV_DATA_API
        : cluster === 'localnet'
          ? process.env.REACT_APP_LOCAL_DATA_API
          : undefined;
  const data = (await axios.post<LookupRegistryInstructions>(`${apiEndpoint || 'http://localhost:3002'}/lookup/upgrade_registry`, {
    airspace, authority, payer
  })).data;
  return data
}