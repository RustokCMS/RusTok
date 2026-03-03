import { graphqlRequest } from '@/shared/api/graphql';

// ---------- Types (mirrors GraphQL schema) ----------

export type GqlContentStatus = 'DRAFT' | 'PUBLISHED' | 'ARCHIVED';

export interface PostSummary {
  id: string;
  title: string;
  slug: string | null;
  excerpt: string | null;
  status: GqlContentStatus;
  authorId: string | null;
  createdAt: string;
  publishedAt: string | null;
}

export interface PostDetail {
  id: string;
  title: string;
  slug: string | null;
  excerpt: string | null;
  body: string | null;
  status: GqlContentStatus;
  authorId: string | null;
  createdAt: string;
  publishedAt: string | null;
  tags: string[];
  featuredImageUrl: string | null;
  seoTitle: string | null;
  seoDescription: string | null;
}

export interface PostListResponse {
  items: PostSummary[];
  total: number;
}

export interface PostListQuery {
  status?: GqlContentStatus;
  authorId?: string;
  locale?: string;
  page?: number;
  perPage?: number;
}

export interface CreatePostInput {
  locale: string;
  title: string;
  body: string;
  excerpt?: string;
  slug?: string;
  publish: boolean;
  tags: string[];
  categoryId?: string;
  featuredImageUrl?: string;
  seoTitle?: string;
  seoDescription?: string;
}

export interface UpdatePostInput {
  locale?: string;
  title?: string;
  body?: string;
  excerpt?: string;
  slug?: string;
  tags?: string[];
  categoryId?: string;
  featuredImageUrl?: string;
  seoTitle?: string;
  seoDescription?: string;
}

// ---------- GraphQL queries ----------

const POSTS_QUERY = `
query Posts($tenantId: UUID!, $filter: PostsFilter) {
  posts(tenantId: $tenantId, filter: $filter) {
    items { id title slug excerpt status authorId createdAt publishedAt }
    total
  }
}`;

const POST_QUERY = `
query Post($tenantId: UUID!, $id: UUID!) {
  post(tenantId: $tenantId, id: $id) {
    id title slug excerpt body status authorId
    createdAt publishedAt tags featuredImageUrl
    seoTitle seoDescription
  }
}`;

const CREATE_POST_MUTATION = `
mutation CreatePost($tenantId: UUID!, $input: CreatePostInput!) {
  createPost(tenantId: $tenantId, input: $input)
}`;

const UPDATE_POST_MUTATION = `
mutation UpdatePost($id: UUID!, $tenantId: UUID!, $input: UpdatePostInput!) {
  updatePost(id: $id, tenantId: $tenantId, input: $input)
}`;

const DELETE_POST_MUTATION = `
mutation DeletePost($id: UUID!, $tenantId: UUID!) {
  deletePost(id: $id, tenantId: $tenantId)
}`;

const PUBLISH_POST_MUTATION = `
mutation PublishPost($id: UUID!, $tenantId: UUID!) {
  publishPost(id: $id, tenantId: $tenantId)
}`;

const UNPUBLISH_POST_MUTATION = `
mutation UnpublishPost($id: UUID!, $tenantId: UUID!) {
  unpublishPost(id: $id, tenantId: $tenantId)
}`;

// ---------- API functions ----------

interface GqlOpts {
  token?: string | null;
  tenantSlug?: string | null;
  tenantId: string;
}

export async function listPosts(
  query: PostListQuery,
  opts: GqlOpts
): Promise<PostListResponse> {
  const data = await graphqlRequest<object, { posts: PostListResponse }>(
    POSTS_QUERY,
    {
      tenantId: opts.tenantId,
      filter: {
        status: query.status ?? null,
        authorId: query.authorId ?? null,
        locale: query.locale ?? null,
        page: query.page ?? 1,
        perPage: query.perPage ?? 20
      }
    },
    opts.token,
    opts.tenantSlug
  );
  return data.posts;
}

export async function getPost(
  id: string,
  opts: GqlOpts
): Promise<PostDetail | null> {
  const data = await graphqlRequest<object, { post: PostDetail | null }>(
    POST_QUERY,
    { tenantId: opts.tenantId, id },
    opts.token,
    opts.tenantSlug
  );
  return data.post;
}

export async function createPost(
  input: CreatePostInput,
  opts: GqlOpts
): Promise<string> {
  const data = await graphqlRequest<object, { createPost: string }>(
    CREATE_POST_MUTATION,
    { tenantId: opts.tenantId, input },
    opts.token,
    opts.tenantSlug
  );
  return data.createPost;
}

export async function updatePost(
  id: string,
  input: UpdatePostInput,
  opts: GqlOpts
): Promise<boolean> {
  const data = await graphqlRequest<object, { updatePost: boolean }>(
    UPDATE_POST_MUTATION,
    { id, tenantId: opts.tenantId, input },
    opts.token,
    opts.tenantSlug
  );
  return data.updatePost;
}

export async function deletePost(
  id: string,
  opts: GqlOpts
): Promise<boolean> {
  const data = await graphqlRequest<object, { deletePost: boolean }>(
    DELETE_POST_MUTATION,
    { id, tenantId: opts.tenantId },
    opts.token,
    opts.tenantSlug
  );
  return data.deletePost;
}

export async function publishPost(
  id: string,
  opts: GqlOpts
): Promise<boolean> {
  const data = await graphqlRequest<object, { publishPost: boolean }>(
    PUBLISH_POST_MUTATION,
    { id, tenantId: opts.tenantId },
    opts.token,
    opts.tenantSlug
  );
  return data.publishPost;
}

export async function unpublishPost(
  id: string,
  opts: GqlOpts
): Promise<boolean> {
  const data = await graphqlRequest<object, { unpublishPost: boolean }>(
    UNPUBLISH_POST_MUTATION,
    { id, tenantId: opts.tenantId },
    opts.token,
    opts.tenantSlug
  );
  return data.unpublishPost;
}
