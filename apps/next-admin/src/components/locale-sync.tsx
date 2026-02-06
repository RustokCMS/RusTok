"use client";

import { useEffect } from "react";
import { usePathname, useRouter } from "next/navigation";

import { locales } from "@/i18n";

const STORAGE_KEY = "rustok-admin-locale";
type LocaleSyncProps = {
  locale: string;
};

function persistLocale(locale: string) {
  try {
    window.localStorage.setItem(STORAGE_KEY, locale);
  } catch {
    // Ignore storage errors (e.g. disabled storage).
  }
}

function isSupportedLocale(value: string | null): value is (typeof locales)[number] {
  return value ? locales.includes(value as (typeof locales)[number]) : false;
}

function replaceLocaleInPath(pathname: string, currentLocale: string, nextLocale: string) {
  const localePrefix = `/${currentLocale}`;
  if (pathname === localePrefix) {
    return `/${nextLocale}`;
  }

  if (pathname.startsWith(`${localePrefix}/`)) {
    return `/${nextLocale}${pathname.slice(localePrefix.length)}`;
  }

  return `/${nextLocale}${pathname}`;
}

export default function LocaleSync({ locale }: LocaleSyncProps) {
  const router = useRouter();
  const pathname = usePathname();

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }

    const storedLocale = window.localStorage.getItem(STORAGE_KEY);
    if (storedLocale && isSupportedLocale(storedLocale) && storedLocale !== locale) {
      const nextPath = replaceLocaleInPath(pathname, locale, storedLocale);
      router.replace(nextPath);
      return;
    }

    persistLocale(locale);
  }, [locale, pathname, router]);

  return null;
}
