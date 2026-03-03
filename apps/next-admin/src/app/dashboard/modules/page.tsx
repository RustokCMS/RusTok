import { auth } from '@/auth';
import { PageContainer } from '@/widgets/app-shell';
import { ModulesList } from '@/features/modules/components/modules-list';
import { listModules } from '@/features/modules/api';
import { redirect } from 'next/navigation';
import { Suspense } from 'react';

export const metadata = {
  title: 'Dashboard: Modules'
};

export default async function Page() {
  const session = await auth();
  if (!session) redirect('/auth/sign-in');

  const modules = await listModules(
    session.user.rustokToken,
    session.user.tenantSlug
  );

  return (
    <PageContainer
      scrollable
      pageTitle='Modules'
      pageDescription='Manage platform modules. Core modules are always active and cannot be disabled.'
    >
      <Suspense fallback={<div>Loading modules...</div>}>
        <ModulesList modules={modules} />
      </Suspense>
    </PageContainer>
  );
}
