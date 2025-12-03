/// <reference types="react" />
/// <reference types="react-dom" />

// Temporary JSX type shim for development without node_modules
// Remove this file after running `npm install`

declare namespace JSX {
  interface IntrinsicElements {
    [elemName: string]: any;
  }
}

declare module 'react' {
  export function useState<T>(initial: T | (() => T)): [T, (v: T | ((prev: T) => T)) => void];
  export function useEffect(effect: () => void | (() => void), deps?: any[]): void;
  export function useCallback<T extends (...args: any[]) => any>(callback: T, deps: any[]): T;
  export function useMemo<T>(factory: () => T, deps: any[]): T;
  export const Fragment: any;
  export type FC<P = {}> = (props: P) => JSX.Element | null;
  export type ChangeEvent<T> = { target: T & { value: string } };
}

declare namespace React {
  type ChangeEvent<T> = { target: T & { value: string } };
}

interface HTMLTextAreaElement {
  value: string;
}

interface HTMLSelectElement {
  value: string;
}

declare module 'next/link' {
  const Link: any;
  export default Link;
}

declare module '@solana/wallet-adapter-react' {
  export function useWallet(): { publicKey: any; connected: boolean };
  export function useConnection(): { connection: any };
}

declare module '@solana/wallet-adapter-react-ui' {
  export const WalletMultiButton: any;
}

