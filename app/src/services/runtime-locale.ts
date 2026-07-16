const LOCALE_STORAGE_KEY = 'chuchen-terminal.locale'

export type RuntimeLocale = 'zh-CN' | 'en-US'

export function getRuntimeLocale(): RuntimeLocale {
  if (typeof window === 'undefined' || !window.localStorage) {
    return 'zh-CN'
  }

  return window.localStorage.getItem(LOCALE_STORAGE_KEY) === 'en-US' ? 'en-US' : 'zh-CN'
}

export function pickLocaleText(zh: string, en: string, locale = getRuntimeLocale()) {
  return locale === 'en-US' ? en : zh
}
