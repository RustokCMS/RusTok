import type { PublicPostSummary } from '../api/posts';

export function PostCard({ post }: { post: PublicPostSummary }) {
  const date = post.publishedAt
    ? new Date(post.publishedAt).toLocaleDateString()
    : null;

  return (
    <article className='rounded-2xl border border-border bg-card p-5 shadow-sm transition-shadow hover:shadow-md'>
      <h3 className='text-lg font-semibold text-card-foreground'>
        {post.title}
      </h3>
      {post.excerpt && (
        <p className='mt-2 line-clamp-3 text-sm text-muted-foreground'>
          {post.excerpt}
        </p>
      )}
      <div className='mt-3 flex items-center gap-2 text-xs text-muted-foreground'>
        {date && <span>{date}</span>}
      </div>
    </article>
  );
}
