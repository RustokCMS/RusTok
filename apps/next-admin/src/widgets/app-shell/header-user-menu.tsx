'use client';

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger
} from '@/shared/ui/shadcn/dropdown-menu';
import { Avatar, AvatarFallback } from '@/shared/ui/shadcn/avatar';
import { IconChevronDown, IconLogout, IconUserCircle } from '@tabler/icons-react';
import { signOut, useSession } from 'next-auth/react';
import { useTranslations } from 'next-intl';
import { useRouter } from 'next/navigation';

function getInitials(value: string, fallback: string) {
  const trimmed = value.trim();
  if (!trimmed) return fallback;
  return trimmed.slice(0, 1).toUpperCase();
}

export function HeaderUserMenu() {
  const router = useRouter();
  const { data: session } = useSession();
  const tMenu = useTranslations('app.menu');
  const user = session?.user;
  const displayName = user?.name || user?.email || tMenu('defaultUser');
  const email = user?.email ?? '';
  const role = user?.role || 'user';
  const initial = getInitials(displayName, tMenu('userInitial'));

  const handleLogout = () => {
    signOut({ callbackUrl: '/auth/sign-in' });
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          type='button'
          className='flex items-center gap-2 rounded-lg p-2 transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring'
          aria-label={tMenu('defaultUser')}
        >
          <Avatar className='h-8 w-8'>
            <AvatarFallback className='bg-primary text-sm font-semibold text-primary-foreground'>
              {initial}
            </AvatarFallback>
          </Avatar>
          <div className='hidden text-left md:block'>
            <p className='text-sm font-medium leading-none text-foreground'>
              {displayName}
            </p>
            <p className='mt-1 text-xs text-muted-foreground'>{role}</p>
          </div>
          <IconChevronDown className='size-4 text-muted-foreground' />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align='end' className='w-56 rounded-lg'>
        <DropdownMenuLabel className='font-normal'>
          <div className='flex flex-col gap-1'>
            <p className='truncate text-sm font-medium text-popover-foreground'>
              {displayName}
            </p>
            {email && (
              <p className='truncate text-xs text-muted-foreground'>{email}</p>
            )}
          </div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuGroup>
          <DropdownMenuItem onClick={() => router.push('/dashboard/profile')}>
            <IconUserCircle className='mr-2 h-4 w-4' />
            {tMenu('profile')}
          </DropdownMenuItem>
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuItem
          onClick={handleLogout}
          className='text-destructive focus:text-destructive'
        >
          <IconLogout className='mr-2 h-4 w-4' />
          {tMenu('signOut')}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
