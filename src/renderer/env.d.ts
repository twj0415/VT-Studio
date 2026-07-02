/// <reference types="vite/client" />

import type { VtStudioApi } from '@shared/contracts/preload';

declare global {
  interface Window {
    vtStudio: VtStudioApi;
  }
}
