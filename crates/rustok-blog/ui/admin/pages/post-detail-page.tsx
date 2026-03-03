import { getPost } from '../api/posts';
import type { PostDetail, GqlContentStatus } from '../api/posts';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

interface PostDetailPageProps {
  postId: string;
  token?: string | null;
  tenantSlug?: string | null;
  tenantId: string;
}

const statusVariant: Record<GqlContentStatus, 'default' | 'secondary' | 'outline'> = {
  PUBLISHED: 'default',
  DRAFT: 'secondary',
  ARCHIVED: 'outline'
};

export default async function PostDetailPage({
  postId,
  token,
  tenantSlug,
  tenantId
}: PostDetailPageProps) {
  const post: PostDetail | null = await getPost(postId, { token, tenantSlug, tenantId });

  if (!post) {
    return (
      <Card>
        <CardContent className='py-8 text-center text-muted-foreground'>
          Post not found
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className='flex items-center gap-3'>
          <CardTitle className='text-2xl'>{post.title}</CardTitle>
          <Badge variant={statusVariant[post.status] ?? 'outline'}>
            {post.status}
          </Badge>
        </div>
        <p className='text-muted-foreground text-sm'>
          {post.slug}
          {post.publishedAt && (
            <> &middot; Published {new Date(post.publishedAt).toLocaleDateString()}</>
          )}
        </p>
      </CardHeader>
      <CardContent className='space-y-4'>
        {post.excerpt && (
          <p className='text-muted-foreground italic'>{post.excerpt}</p>
        )}
        <div className='prose max-w-none whitespace-pre-wrap'>
          {post.body}
        </div>
        {post.tags.length > 0 && (
          <div className='flex flex-wrap gap-2'>
            {post.tags.map((tag) => (
              <Badge key={tag} variant='outline'>{tag}</Badge>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
