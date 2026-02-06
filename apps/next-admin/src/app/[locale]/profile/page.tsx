import { getLocale } from "next-intl/server";

import ProfileView from "./view";

export default async function ProfilePage() {
  const locale = await getLocale();
  return <ProfileView locale={locale} />;
}
