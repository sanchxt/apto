// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    // interface PageData {}
    // interface PageState {}
    // interface Platform {}
  }

  interface Window {
    __TAURI__?: {
      window: {
        appWindow: {
          close: () => Promise<void>;
          minimize: () => Promise<void>;
          maximize: () => Promise<void>;
          unmaximize: () => Promise<void>;
          isMaximized: () => Promise<boolean>;
        };
      };
    };
  }
}

export {};
