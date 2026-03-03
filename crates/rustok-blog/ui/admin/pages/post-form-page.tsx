import { getPost } from '../api/posts';
import PostForm from '../components/post-form';

interface PostFormPageProps {
  postId?: string;
  token?: string | null;
  tenantSlug?: string | null;
  tenantId: string;
}

export default async function PostFormPage({
  postId,
  token,
  tenantSlug,
  tenantId
}: PostFormPageProps) {
  const initialData = postId
    ? await getPost(postId, { token, tenantSlug, tenantId })
    : null;

  return (
    <PostForm
      initialData={initialData}
      pageTitle={initialData ? 'Edit Post' : 'Create Post'}
    />
  );
}
