'use client';
import PageContainer from '@/components/layout/page-container';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useSession } from 'next-auth/react';
import { BadgeCheck } from 'lucide-react';

export default function ExclusivePage() {
  const { data: session } = useSession();
  const user = session?.user;

  return (
    <PageContainer>
      <div className='space-y-6'>
        <div>
          <h1 className='flex items-center gap-2 text-3xl font-bold tracking-tight'>
            <BadgeCheck className='h-7 w-7 text-green-600' />Admin Area
          </h1>
          <p className='text-muted-foreground'>Welcome, <span className='font-semibold'>{user?.name || user?.email}</span>!</p>
        </div>
        <Card>
          <CardHeader>
            <CardTitle>RusTok Admin Panel</CardTitle>
            <CardDescription>You are signed in as {user?.role}.</CardDescription>
          </CardHeader>
          <CardContent><div className='text-lg'>Have a wonderful day!</div></CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
