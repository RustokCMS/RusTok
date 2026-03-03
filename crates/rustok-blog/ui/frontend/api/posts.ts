import { graphqlRequest } from '@/lib/graphql';

export interface PublicPostSummary {
  id: string;
  title: string;
  slug: string | null;
  excerpt: string | null;
  status: string;
  authorId: string | null;
  createdAt: string;
  publishedAt: string | null;
}

export interface PublicPostListResponse {
  items: PublicPostSummary[];
  total: number;
}

const PUBLISHED_POSTS_QUERY = `
query PublishedPosts($tenantId: UUID!, $filter: PostsFilter) {
  posts(tenantId: $tenantId, filter: $filter) {
    items { id title slug excerpt status authorId createdAt publishedAt }
    total
  }
}`;

const CURRENT_TENANT_QUERY = `
query CurrentTenant {
  currentTenant { id }
}`;

async function resolveTenantId(tenantSlug: string): Promise<string> {
  const data = await graphqlRequest<undefined, { currentTenant: { id: string } }>(
    CURRENT_TENANT_QUERY,
    undefined,
    null,
    tenantSlug
  );
  return data.currentTenant.id;
}

export async function fetchPublishedPosts(
  page = 1,
  perPage = 6
): Promise<PublicPostListResponse> {
  const tenantSlug = process.env.NEXT_PUBLIC_TENANT_SLUG ?? '';
  const tenantId = process.env.NEXT_PUBLIC_TENANT_ID
    ?? await resolveTenantId(tenantSlug);

  const data = await graphqlRequest<object, { posts: PublicPostListResponse }>(
    PUBLISHED_POSTS_QUERY,
    {
      tenantId,
      filter: {
        status: 'PUBLISHED',
        page,
        perPage
      }
    },
    null,
    tenantSlug
  );
  return data.posts;
}
