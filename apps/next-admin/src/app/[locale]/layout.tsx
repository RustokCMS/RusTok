import type { ReactNode } from "react";
import { setRequestLocale } from "next-intl/server";

import LocaleSync from "@/components/locale-sync";

export default async function LocaleLayout({
  children,
  params,
}: {
  children: ReactNode;
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;

  setRequestLocale(locale);

  return (
    <>
      <LocaleSync locale={locale} />
      {children}
    </>
  );
}
