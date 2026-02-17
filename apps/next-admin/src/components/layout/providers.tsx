'use client';
import React from 'react';
import { SessionProvider } from 'next-auth/react';
import { ActiveThemeProvider } from '../themes/active-theme';

export default function Providers({
  activeThemeValue,
  children
}: {
  activeThemeValue: string;
  children: React.ReactNode;
}) {
  return (
    <ActiveThemeProvider initialTheme={activeThemeValue}>
      <SessionProvider>
        {children}
      </SessionProvider>
    </ActiveThemeProvider>
  );
}
