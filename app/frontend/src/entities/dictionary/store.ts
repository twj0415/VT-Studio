import { defineStore } from 'pinia'

import { getDictionary, listDictionaries } from './api'
import type { DictionaryDto } from './types'

export const useDictionaryStore = defineStore('dictionary', {
  state: () => ({
    dictionaries: [] as DictionaryDto[],
  }),
  actions: {
    async loadDictionaries() {
      this.dictionaries = await listDictionaries()
    },
    async loadDictionary(code: string) {
      return getDictionary(code)
    },
  },
})
