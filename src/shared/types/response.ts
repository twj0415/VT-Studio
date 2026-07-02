export type EmptyData = Record<string, never>;

export interface VtResponse<TData extends object = EmptyData> {
  code: 200 | 400;
  data: TData;
  msg: string;
}
