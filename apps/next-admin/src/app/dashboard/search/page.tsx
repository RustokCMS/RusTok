import { Suspense } from 'react';

import { auth } from '@/auth';
import { PageContainer } from '@/widgets/app-shell';
import { SearchAdminPage } from '../../../../packages/search/src';

export const metadata = {
  title: 'Dashboard: Search'
};

export default async function Page() {
  const session = await auth();
  const token = session?.user?.rustokToken ?? null;
  const tenantSlug = session?.user?.tenantSlug ?? null;

  return (
    <PageContainer
      scrollable
      pageTitle='Search'
      pageDescription='Inspect search diagnostics, queue rebuilds, and run PostgreSQL FTS previews'
    >
      <Suspense fallback={<div>Loading search control plane...</div>}>
        <SearchAdminPage token={token} tenantSlug={tenantSlug} />
      </Suspense>
    </PageContainer>
  );
}
