"use client";

import { useEffect } from "react";

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

export default function LocaleSync({ locale }: LocaleSyncProps) {
  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }

    persistLocale(locale);
  }, [locale]);

  return null;
}
