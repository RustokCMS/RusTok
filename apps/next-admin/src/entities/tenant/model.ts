export interface Tenant {
  id: string;
  name: string;
  slug: string;
  plan?: string;
  isActive: boolean;
}

export interface Workspace {
  id: string;
  name: string;
  tenantId: string;
  role?: string;
}
