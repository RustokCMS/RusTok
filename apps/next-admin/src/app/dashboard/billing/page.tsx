import PageContainer from '@/components/layout/page-container';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle
} from '@/components/ui/card';

export default function BillingPage() {
  return (
    <PageContainer pageTitle='Billing & Plans' pageDescription='Manage your workspace subscription'>
      <Card>
        <CardHeader>
          <CardTitle>Billing</CardTitle>
          <CardDescription>
            Billing management is handled by your workspace administrator.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <p className='text-muted-foreground text-sm'>
            Contact your administrator to manage subscription plans.
          </p>
        </CardContent>
      </Card>
    </PageContainer>
  );
}
