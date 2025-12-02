import { PublicKey, Connection, Transaction } from '@solana/web3.js'
import { Program, AnchorProvider } from '@coral-xyz/anchor'

// Solana utility functions

export const PROGRAM_ID = new PublicKey(
  process.env.NEXT_PUBLIC_PROGRAM_ID || 'Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS'
)

export const RPC_URL = process.env.NEXT_PUBLIC_RPC_URL || 'https://api.devnet.solana.com'

export function getConnection(): Connection {
  return new Connection(RPC_URL, 'confirmed')
}

export async function getProgram(provider: AnchorProvider): Promise<Program> {
  // TODO: Load IDL and create program instance
  // This would require the actual IDL file from anchor build
  throw new Error('Program initialization not yet implemented')
}

export function shortenAddress(address: string, chars = 4): string {
  return `${address.slice(0, chars)}...${address.slice(-chars)}`
}

export async function confirmTransaction(
  connection: Connection,
  signature: string,
  commitment = 'confirmed'
): Promise<void> {
  const latestBlockhash = await connection.getLatestBlockhash()
  await connection.confirmTransaction({
    signature,
    blockhash: latestBlockhash.blockhash,
    lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
  }, commitment)
}

