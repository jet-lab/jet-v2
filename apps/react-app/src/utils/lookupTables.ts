import { useJetStore } from '@jet-lab/store';
import axios from 'axios';

export async function getAuthorityLookupTables(authority: string): Promise<string[]> {
  // const { cluster } = useJetStore(state => ({
  //   cluster: state.settings.cluster,
  // }));
  return (await axios.get<{
    authority: string,
    addresses: string[]
  }>(`http://localhost:3006/lookup/authority_addresses/${authority}`)).data.addresses
}