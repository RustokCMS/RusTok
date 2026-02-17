'use client';
import { useMemo } from 'react';
import { useSession } from 'next-auth/react';
import type { NavItem } from '@/types';

export function useFilteredNavItems(items: NavItem[]) {
  const { data: session } = useSession();
  const role = session?.user?.role;

  return useMemo(() => {
    return items
      .filter((item) => {
        if (!item.access) return true;
        if (item.access.requireOrg) return false;
        if (item.access.role && role?.toLowerCase() !== item.access.role.toLowerCase()) return false;
        return true;
      })
      .map((item) => ({
        ...item,
        items: item.items?.filter((child) => {
          if (!child.access) return true;
          if (child.access.requireOrg) return false;
          if (child.access.role && role?.toLowerCase() !== child.access.role.toLowerCase()) return false;
          return true;
        }) ?? []
      }));
  }, [items, role]);
}
