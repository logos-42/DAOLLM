import { useWallet, useConnection } from '@solana/wallet-adapter-react'
import { useCallback } from 'react'
import { PublicKey, Transaction } from '@solana/web3.js'

export function useSolana() {
  const { publicKey, sendTransaction, signTransaction } = useWallet()
  const { connection } = useConnection()

  const sendAndConfirmTransaction = useCallback(
    async (transaction: Transaction) => {
      if (!publicKey || !sendTransaction) {
        throw new Error('Wallet not connected')
      }

      const signature = await sendTransaction(transaction, connection)
      await connection.confirmTransaction(signature, 'confirmed')
      return signature
    },
    [publicKey, sendTransaction, connection]
  )

  return {
    publicKey,
    connection,
    sendAndConfirmTransaction,
    isConnected: !!publicKey,
  }
}

