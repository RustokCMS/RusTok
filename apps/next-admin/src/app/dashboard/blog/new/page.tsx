import { auth } from '@/auth';
import { PageContainer } from '@/widgets/app-shell';
import { PostFormPage } from '@rustok/blog-admin';
import { redirect } from 'next/navigation';
import { Suspense } from 'react';

export const metadata = {
  title: 'Dashboard: New Post'
};

export default async function Page() {
  const session = await auth();
  if (!session) redirect('/auth/sign-in');

  return (
    <PageContainer scrollable pageTitle='Create Post'>
      <Suspense fallback={<div>Loading form...</div>}>
        <PostFormPage
          token={session.user.rustokToken}
          tenantSlug={session.user.tenantSlug}
          tenantId={session.user.tenantId ?? ''}
        />
      </Suspense>
    </PageContainer>
  );
}
