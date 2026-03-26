import {
  Contract,
  TransactionBuilder,
  SorobanRpc,
  xdr,
  Address,
  nativeToScVal,
  scValToNative,
} from '@stellar/stellar-sdk';
import { CONTRACTS, SOROBAN_RPC_URL, NETWORK_PASSPHRASE } from './contracts';

const server = new SorobanRpc.Server(SOROBAN_RPC_URL);

export async function buildContractCall(
  contractId: string,
  method: string,
  args: xdr.ScVal[],
  publicKey: string,
): Promise<string> {
  const account = await server.getAccount(publicKey);
  const contract = new Contract(contractId);

  const tx = new TransactionBuilder(account, {
    fee: '10000000',
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call(method, ...args))
    .setTimeout(30)
    .build();

  const prepared = await server.prepareTransaction(tx);
  return prepared.toXDR();
}

export async function submitTransaction(signedXdr: string): Promise<any> {
  const tx = TransactionBuilder.fromXDR(signedXdr, NETWORK_PASSPHRASE);
  const response = await server.sendTransaction(tx);

  if (response.status === 'PENDING') {
    let result = await server.getTransaction(response.hash);
    while (result.status === 'NOT_FOUND') {
      await new Promise((r) => setTimeout(r, 1000));
      result = await server.getTransaction(response.hash);
    }
    return result;
  }
  return response;
}

export async function readContract(
  contractId: string,
  method: string,
  args: xdr.ScVal[],
): Promise<any> {
  const contract = new Contract(contractId);
  const tx = new TransactionBuilder(
    new (await import('@stellar/stellar-sdk')).Account(
      'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF',
      '0',
    ),
    {
      fee: '100',
      networkPassphrase: NETWORK_PASSPHRASE,
    },
  )
    .addOperation(contract.call(method, ...args))
    .setTimeout(30)
    .build();

  const sim = await server.simulateTransaction(tx);
  if (SorobanRpc.Api.isSimulationSuccess(sim) && sim.result) {
    return scValToNative(sim.result.retval);
  }
  throw new Error('Simulation failed');
}

// --- Typed helpers ---

export async function getCampaign(id: number) {
  return readContract(CONTRACTS.campaignManager, 'get_campaign', [
    nativeToScVal(id, { type: 'u32' }),
  ]);
}

export async function getUserPosition(id: number, userAddress: string) {
  return readContract(CONTRACTS.campaignManager, 'get_user_position', [
    nativeToScVal(id, { type: 'u32' }),
    new Address(userAddress).toScVal(),
  ]);
}

export async function buildCreateCampaign(
  feeUsdc: bigint,
  adapterId: bigint,
  poolAddress: string,
  lpToken: string,
  duration: number,
  targetLp: bigint,
  bonusFlash: bigint,
  minRewardFlash: bigint,
  creator: string,
) {
  return buildContractCall(CONTRACTS.campaignManager, 'create_campaign', [
    nativeToScVal(feeUsdc, { type: 'i128' }),
    nativeToScVal(adapterId, { type: 'i128' }),
    new Address(poolAddress).toScVal(),
    new Address(lpToken).toScVal(),
    nativeToScVal(duration, { type: 'u32' }),
    nativeToScVal(targetLp, { type: 'i128' }),
    nativeToScVal(bonusFlash, { type: 'i128' }),
    nativeToScVal(minRewardFlash, { type: 'i128' }),
    new Address(creator).toScVal(),
  ], creator);
}

export async function buildJoinCampaign(
  id: number,
  amount: bigint,
  token: string,
  user: string,
) {
  return buildContractCall(CONTRACTS.campaignManager, 'join_campaign', [
    nativeToScVal(id, { type: 'u32' }),
    nativeToScVal(amount, { type: 'i128' }),
    new Address(token).toScVal(),
    new Address(user).toScVal(),
  ], user);
}

export async function buildClaim(id: number, user: string) {
  return buildContractCall(CONTRACTS.campaignManager, 'claim', [
    nativeToScVal(id, { type: 'u32' }),
    new Address(user).toScVal(),
  ], user);
}
