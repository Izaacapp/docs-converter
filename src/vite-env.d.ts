/// <reference types="svelte" />
/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Converter server API the app forwards PDFs to (see .env). */
  readonly VITE_CONVERTER_API_URL?: string;
}
interface ImportMeta {
  readonly env: ImportMetaEnv;
}
