export type MediaRoot = 'project' | 'cache' | 'temp' | 'exports';

export type MediaMode = 'original' | 'thumbnail';

export type MediaThumbnailSize = 'small' | 'list' | 'detail';

export interface MediaCreateUrlPayload {
  root: MediaRoot;
  relativePath: string;
  expiresInSeconds?: number;
}

export interface MediaCreateUrlResult {
  url: string;
}

export interface MediaCreateThumbnailUrlPayload {
  root: MediaRoot;
  relativePath: string;
  size?: MediaThumbnailSize;
  expiresInSeconds?: number;
}

export interface MediaCreateThumbnailUrlResult {
  url: string;
  fallback: boolean;
}

export interface MediaResolveUrlPayload {
  url: string;
}

export interface MediaResolveUrlResult {
  root: MediaRoot;
  relativePath: string;
  mode: MediaMode;
  size?: MediaThumbnailSize;
}

export interface MediaGetOriginalUrlPayload {
  url?: string;
  root?: MediaRoot;
  relativePath?: string;
  expiresInSeconds?: number;
}

export interface MediaGetOriginalUrlResult {
  url: string;
}
