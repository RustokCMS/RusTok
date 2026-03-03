import { auth } from '@/auth';
import { PageContainer } from '@/widgets/app-shell';
import { PostDetailPage } from '@rustok/blog-admin';
import { redirect } from 'next/navigation';
import { Suspense } from 'react';

export const metadata = {
  title: 'Dashboard: Post Detail'
};

type PageProps = {
  params: Promise<{ postId: string }>;
};

export default async function Page(props: PageProps) {
  const session = await auth();
  if (!session) redirect('/auth/sign-in');

  const { postId } = await props.params;

  return (
    <PageContainer scrollable pageTitle='Post Detail'>
      <Suspense fallback={<div>Loading post...</div>}>
        <PostDetailPage
          postId={postId}
          token={session.user.rustokToken}
          tenantSlug={session.user.tenantSlug}
          tenantId={session.user.tenantId ?? ''}
        />
      </Suspense>
    </PageContainer>
  );
}
