import { getLocale } from "next-intl/server";

import RegisterView from "./view";

export default async function RegisterPage() {
  const locale = await getLocale();
  return <RegisterView locale={locale} />;
}
