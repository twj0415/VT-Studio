import { dictRegistry } from '@/shared/dict/dict.registry'

import type { DictionaryDto } from './types'

const dictionaries: Record<string, DictionaryDto> = Object.fromEntries(
  Object.entries(dictRegistry).map(([code, items]) => [code, { code, items }])
)

export async function listDictionaries(): Promise<DictionaryDto[]> {
  return Object.values(dictionaries)
}

export async function getDictionary(code: string): Promise<DictionaryDto> {
  return dictionaries[code] ?? { code, items: [] }
}
