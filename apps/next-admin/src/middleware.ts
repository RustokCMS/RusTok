import { auth } from '@/auth';
import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

export default auth((req) => {
  const { nextUrl, auth: session } = req;
  const isAuthenticated = !!session;

  // Защищённые маршруты
  if (nextUrl.pathname.startsWith('/dashboard')) {
    if (!isAuthenticated) {
      const signInUrl = new URL('/auth/sign-in', nextUrl.origin);
      signInUrl.searchParams.set('callbackUrl', nextUrl.pathname);
      return NextResponse.redirect(signInUrl);
    }
  }

  // Корневой редирект
  if (nextUrl.pathname === '/') {
    return NextResponse.redirect(
      new URL(isAuthenticated ? '/dashboard/overview' : '/auth/sign-in', nextUrl.origin)
    );
  }

  return NextResponse.next();
});

export const config = {
  matcher: ['/((?!_next/static|_next/image|favicon.ico|api/auth).*)']
};
