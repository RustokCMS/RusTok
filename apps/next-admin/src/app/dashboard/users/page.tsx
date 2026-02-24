import { auth } from '@/auth';
import PageContainer from '@/components/layout/page-container';
import { Metadata } from 'next';
import { redirect } from 'next/navigation';
import UsersView from '@/features/users/components/users-view';

export const metadata: Metadata = {
  title: 'Dashboard : Users',
  description: 'Manage users in your workspace'
};

export default async function UsersPage() {
  const session = await auth();
  const role = session?.user?.role;
  if (!role || role === 'CUSTOMER') {
    redirect('/dashboard/overview');
  }

  return (
    <PageContainer
      pageTitle='Users'
      pageDescription='View and manage users in your workspace'
    >
      <UsersView />
    </PageContainer>
  );
}
