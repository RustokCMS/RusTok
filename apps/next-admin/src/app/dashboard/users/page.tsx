import PageContainer from '@/components/layout/page-container';
import { Metadata } from 'next';
import UsersView from '@/features/users/components/users-view';

export const metadata: Metadata = {
  title: 'Dashboard : Users',
  description: 'Manage users in your workspace'
};

export default function UsersPage() {
  return (
    <PageContainer
      pageTitle='Users'
      pageDescription='View and manage users in your workspace'
    >
      <UsersView />
    </PageContainer>
  );
}
