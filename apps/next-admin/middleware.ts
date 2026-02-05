import createMiddleware from "next-intl/middleware";
import { NextResponse, type NextRequest } from "next/server";

import { defaultLocale, locales } from "./src/i18n";

const intlMiddleware = createMiddleware({
  locales,
  defaultLocale,
});

export default function middleware(request: NextRequest) {
  const response = intlMiddleware(request);
  const { pathname } = request.nextUrl;
  const [, locale] = pathname.split("/");
  const isLocaleRoute = locales.includes(locale as (typeof locales)[number]);
  const isPublicRoute = pathname.endsWith("/login") || pathname.endsWith("/register") || pathname.endsWith("/reset");
  const token = request.cookies.get("rustok-admin-token")?.value;

  if (isLocaleRoute && !isPublicRoute && !token) {
    const loginUrl = request.nextUrl.clone();
    loginUrl.pathname = `/${locale}/login`;
    return NextResponse.redirect(loginUrl);
  }

  return response;
}

export const config = {
  matcher: ["/", "/(ru|en)/:path*"],
};
