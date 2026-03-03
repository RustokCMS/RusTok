import type { PostSummary, GqlContentStatus } from '../api/posts';
import { listPosts } from '../api/posts';
import { PostTable } from '../components/post-table';
import { columns } from '../components/post-table/columns';

interface PostsPageProps {
  searchParams: {
    page?: string;
    perPage?: string;
    title?: string;
    status?: string;
  };
  token?: string | null;
  tenantSlug?: string | null;
  tenantId: string;
}

export default async function PostsPage({
  searchParams,
  token,
  tenantSlug,
  tenantId
}: PostsPageProps) {
  const page = Number(searchParams.page) || 1;
  const perPage = Number(searchParams.perPage) || 20;
  const status = searchParams.status as GqlContentStatus | undefined;

  const data = await listPosts(
    { page, perPage, status },
    { token, tenantSlug, tenantId }
  );

  const posts: PostSummary[] = data.items;

  return <PostTable data={posts} totalItems={data.total} columns={columns} />;
}
