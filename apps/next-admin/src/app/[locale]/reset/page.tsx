import { getLocale } from "next-intl/server";

import ResetView from "./view";

export default async function ResetPage() {
  const locale = await getLocale();
  return <ResetView locale={locale} />;
}
