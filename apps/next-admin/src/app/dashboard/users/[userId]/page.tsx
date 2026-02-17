import PageContainer from '@/components/layout/page-container';
import { Metadata } from 'next';
import UserDetailView from '@/features/users/components/user-detail-view';

export const metadata: Metadata = {
  title: 'Dashboard : User Detail',
  description: 'View user details'
};

export default function UserDetailPage({ params }: { params: { userId: string } }) {
  return (
    <PageContainer
      pageTitle='User Detail'
      pageDescription='View and manage user information'
    >
      <UserDetailView userId={params.userId} />
    </PageContainer>
  );
}
