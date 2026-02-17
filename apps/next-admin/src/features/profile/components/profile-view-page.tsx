'use client';
import PageContainer from '@/components/layout/page-container';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useSession } from 'next-auth/react';

export default function ProfileViewPage() {
  const { data: session } = useSession();
  const user = session?.user;

  return (
    <PageContainer>
      <div className='flex w-full flex-col gap-6 p-4'>
        <h1 className='text-2xl font-bold tracking-tight'>Profile</h1>
        {user ? (
          <div className='grid gap-4 md:grid-cols-2'>
            <Card>
              <CardHeader><CardTitle>Account Information</CardTitle></CardHeader>
              <CardContent className='space-y-3'>
                {[
                  { label: 'Name', value: user.name || '—' },
                  { label: 'Email', value: user.email },
                  { label: 'Role', value: user.role },
                  { label: 'Status', value: user.status },
                  { label: 'Workspace', value: user.tenantSlug || '—' },
                  { label: 'ID', value: <span className='font-mono text-xs'>{user.id}</span> }
                ].map(({ label, value }) => (
                  <div key={label}>
                    <p className='text-muted-foreground text-xs font-medium uppercase tracking-wider'>{label}</p>
                    <div className='mt-1 text-sm font-medium'>{value}</div>
                  </div>
                ))}
              </CardContent>
            </Card>
          </div>
        ) : (
          <p className='text-muted-foreground text-sm'>Loading profile...</p>
        )}
      </div>
    </PageContainer>
  );
}
