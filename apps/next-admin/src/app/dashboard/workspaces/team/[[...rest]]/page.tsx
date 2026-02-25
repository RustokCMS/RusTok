import { PageContainer } from '@/widgets/app-shell';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle
} from '@/components/ui/card';

export default function TeamPage() {
  return (
    <PageContainer
      pageTitle='Team Management'
      pageDescription='Manage your workspace team members and roles.'
    >
      <Card>
        <CardHeader>
          <CardTitle>Team Members</CardTitle>
          <CardDescription>
            Team management is available through the Users section.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <p className='text-muted-foreground text-sm'>
            Visit the <a className='text-primary underline' href='/dashboard/users'>Users page</a> to manage team members.
          </p>
        </CardContent>
      </Card>
    </PageContainer>
  );
}
