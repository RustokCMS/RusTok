/**
 * Auth API functions — all communication goes through GraphQL.
 */

import { graphqlRequest } from './graphql';

// Types

export interface AuthUser {
  id: string;
  email: string;
  name: string | null;
  role: string;
  status: string;
  tenantSlug: string | null;
  createdAt: string;
}

export interface AuthSession {
  token: string;
  tenantSlug: string | null;
}

// Mutations

const SIGN_IN_MUTATION = `
mutation SignIn($email: String!, $password: String!, $tenantSlug: String!) {
  signIn(email: $email, password: $password, tenantSlug: $tenantSlug) {
    token
    user {
      id
      email
      name
      role
      status
      tenantSlug
      createdAt
    }
  }
}
`;

const SIGN_UP_MUTATION = `
mutation SignUp($email: String!, $password: String!, $name: String, $tenantSlug: String!) {
  signUp(email: $email, password: $password, name: $name, tenantSlug: $tenantSlug) {
    token
    user {
      id
      email
      name
      role
      status
      tenantSlug
      createdAt
    }
  }
}
`;

const SIGN_OUT_MUTATION = `
mutation SignOut {
  signOut {
    success
  }
}
`;

const CURRENT_USER_QUERY = `
query Me {
  me {
    id
    email
    name
    role
    status
    tenantSlug
    createdAt
  }
}
`;

// Responses

interface SignInResponse {
  signIn: {
    token: string;
    user: AuthUser;
  };
}

interface SignUpResponse {
  signUp: {
    token: string;
    user: AuthUser;
  };
}

interface MeResponse {
  me: AuthUser;
}

// API functions

export async function signIn(
  email: string,
  password: string,
  tenantSlug: string
): Promise<{ token: string; user: AuthUser }> {
  const data = await graphqlRequest<
    { email: string; password: string; tenantSlug: string },
    SignInResponse
  >(SIGN_IN_MUTATION, { email, password, tenantSlug });
  return data.signIn;
}

export async function signUp(
  email: string,
  password: string,
  tenantSlug: string,
  name?: string
): Promise<{ token: string; user: AuthUser }> {
  const data = await graphqlRequest<
    { email: string; password: string; tenantSlug: string; name?: string },
    SignUpResponse
  >(SIGN_UP_MUTATION, { email, password, tenantSlug, name });
  return data.signUp;
}

export async function signOut(token: string, tenantSlug?: string | null): Promise<void> {
  try {
    await graphqlRequest(SIGN_OUT_MUTATION, undefined, token, tenantSlug);
  } catch {
    // Ignore sign out errors — clear local state regardless
  }
}

export async function fetchCurrentUser(
  token: string,
  tenantSlug?: string | null
): Promise<AuthUser | null> {
  try {
    const data = await graphqlRequest<undefined, MeResponse>(
      CURRENT_USER_QUERY,
      undefined,
      token,
      tenantSlug
    );
    return data.me;
  } catch {
    return null;
  }
}
