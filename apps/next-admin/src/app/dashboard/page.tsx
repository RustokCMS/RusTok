import { auth } from '@/auth';
import { redirect } from 'next/navigation';

export default async function Dashboard() {
  const session = await auth();
  if (!session) redirect('/auth/sign-in');
  redirect('/dashboard/overview');
}
