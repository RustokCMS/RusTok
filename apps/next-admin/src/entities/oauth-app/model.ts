export type AppType = 'Embedded' | 'FirstParty' | 'Mobile' | 'Service' | 'ThirdParty';

export interface OAuthApp {
  id: string;
  name: string;
  slug: string;
  description?: string;
  appType: AppType;
  clientId: string;
  redirectUris: string[];
  scopes: string[];
  grantTypes: string[];
  manifestRef?: string;
  autoCreated: boolean;
  isActive: boolean;
  activeTokenCount: number;
  lastUsedAt?: string;
  createdAt: string;
}

export interface CreateOAuthAppInput {
  name: string;
  slug: string;
  description?: string;
  appType: AppType;
  redirectUris?: string[];
  scopes: string[];
  grantTypes: string[];
}

export interface CreateOAuthAppResult {
  app: OAuthApp;
  clientSecret: string;
}
