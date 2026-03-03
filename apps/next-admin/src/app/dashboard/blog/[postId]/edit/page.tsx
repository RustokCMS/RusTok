import { auth } from '@/auth';
import { PageContainer } from '@/widgets/app-shell';
import { PostFormPage } from '@rustok/blog-admin';
import { redirect } from 'next/navigation';
import { Suspense } from 'react';

export const metadata = {
  title: 'Dashboard: Edit Post'
};

type PageProps = {
  params: Promise<{ postId: string }>;
};

export default async function Page(props: PageProps) {
  const session = await auth();
  if (!session) redirect('/auth/sign-in');

  const { postId } = await props.params;

  return (
    <PageContainer scrollable pageTitle='Edit Post'>
      <Suspense fallback={<div>Loading form...</div>}>
        <PostFormPage
          postId={postId}
          token={session.user.rustokToken}
          tenantSlug={session.user.tenantSlug}
          tenantId={session.user.tenantId ?? ''}
        />
      </Suspense>
    </PageContainer>
  );
}
