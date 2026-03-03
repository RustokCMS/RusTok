import { graphqlRequest } from '@/shared/api/graphql';

export interface ModuleInfo {
  moduleSlug: string;
  name: string;
  description: string;
  version: string;
  kind: 'core' | 'optional';
  enabled: boolean;
  dependencies: string[];
}

export interface TenantModuleResult {
  moduleSlug: string;
  enabled: boolean;
  settings: string;
}

// ---------- GraphQL queries ----------

const MODULE_REGISTRY_QUERY = `
query ModuleRegistry {
  moduleRegistry {
    moduleSlug
    name
    description
    version
    kind
    enabled
    dependencies
  }
}`;

const TOGGLE_MODULE_MUTATION = `
mutation ToggleModule($moduleSlug: String!, $enabled: Boolean!) {
  toggleModule(moduleSlug: $moduleSlug, enabled: $enabled) {
    moduleSlug
    enabled
    settings
  }
}`;

// ---------- API functions ----------

export async function listModules(
  token?: string | null,
  tenantSlug?: string | null
): Promise<ModuleInfo[]> {
  const data = await graphqlRequest<
    undefined,
    { moduleRegistry: ModuleInfo[] }
  >(MODULE_REGISTRY_QUERY, undefined, token, tenantSlug);
  return data.moduleRegistry;
}

export async function toggleModule(
  moduleSlug: string,
  enabled: boolean,
  token?: string | null,
  tenantSlug?: string | null
): Promise<TenantModuleResult> {
  const data = await graphqlRequest<
    object,
    { toggleModule: TenantModuleResult }
  >(
    TOGGLE_MODULE_MUTATION,
    { moduleSlug, enabled },
    token,
    tenantSlug
  );
  return data.toggleModule;
}
