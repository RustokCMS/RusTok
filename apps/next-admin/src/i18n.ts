import { getRequestConfig } from "next-intl/server";

export const locales = ["ru", "en"] as const;
export const defaultLocale = "ru";

export default getRequestConfig(async ({ locale }) => {
  if (!locales.includes(locale as (typeof locales)[number])) {
    return { locale, messages: {} };
  }

  return {
    locale,
    messages: (await import(`../messages/${locale}.json`)).default,
  };
});
