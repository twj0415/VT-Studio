import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import { dictRegistry } from './dict.registry'

export type DictKey = keyof typeof dictRegistry

function toDictI18nKey(key: string, value: string) {
  return `dict.${key}.${value.replace(/[^a-zA-Z0-9_]/g, '_')}`
}

export function useDictOptions<TKey extends DictKey>(key: TKey) {
  const { t, te, locale } = useI18n()

  return computed(() => {
    locale.value

    return dictRegistry[key].map((option) => {
      const i18nKey = toDictI18nKey(key, option.value)

      return {
        ...option,
        label: te(i18nKey) ? t(i18nKey) : option.label,
      }
    }) as (typeof dictRegistry)[TKey]
  })
}
