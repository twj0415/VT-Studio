export interface DictionaryItemDto {
  label: string
  value: string
  disabled?: boolean
  colorToken?: string
}

export interface DictionaryDto {
  code: string
  items: DictionaryItemDto[]
}
