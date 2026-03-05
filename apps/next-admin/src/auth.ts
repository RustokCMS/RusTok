import NextAuth from 'next-auth';
import Credentials from 'next-auth/providers/credentials';
import { signIn as rustokSignIn, fetchCurrentTenant } from '@/shared/api/auth-api';

export const { handlers, signIn, signOut, auth } = NextAuth({
  providers: [
    Credentials({
      credentials: {
        email: { label: 'Email', type: 'email' },
        password: { label: 'Password', type: 'password' },
        tenantSlug: { label: 'Workspace', type: 'text' }
      },
      authorize: async (credentials) => {
        if (!credentials?.email || !credentials?.password || !credentials?.tenantSlug) {
          return null;
        }
        try {
          const result = await rustokSignIn(
            credentials.email as string,
            credentials.password as string,
            credentials.tenantSlug as string
          );

          // Fetch tenantId via currentTenant query
          let tenantId: string | null = null;
          const tenant = await fetchCurrentTenant(
            result.accessToken,
            credentials.tenantSlug as string
          );
          if (tenant) {
            tenantId = tenant.id;
          }

          return {
            id: result.user.id,
            email: result.user.email,
            name: result.user.name,
            role: result.user.role,
            status: result.user.status,
            tenantSlug: credentials.tenantSlug as string,
            tenantId,
            rustokToken: result.accessToken
          };
        } catch {
          return null;
        }
      }
    })
  ],
  callbacks: {
    jwt({ token, user }) {
      if (user) {
        token.id = user.id ?? '';
        token.role = user.role;
        token.status = user.status;
        token.tenantSlug = user.tenantSlug;
        token.tenantId = user.tenantId;
        token.rustokToken = user.rustokToken;
      }
      return token;
    },
    session({ session, token }) {
      session.user.id = token.id;
      session.user.role = token.role;
      session.user.status = token.status;
      session.user.tenantSlug = token.tenantSlug;
      session.user.tenantId = token.tenantId;
      session.user.rustokToken = token.rustokToken;
      return session;
    }
  },
  pages: {
    signIn: '/auth/sign-in'
  },
  session: {
    strategy: 'jwt',
    maxAge: 7 * 24 * 60 * 60
  }
});
