import { getLocale } from "next-intl/server";

import SecurityView from "./view";

export default async function SecurityPage() {
  const locale = await getLocale();
  return <SecurityView locale={locale} />;
}
