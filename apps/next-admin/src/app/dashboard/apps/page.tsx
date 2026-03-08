'use client';

import { useState } from 'react';
import { OAuthApp } from '@/entities/oauth-app';
import { CreateAppDialog, RevokeAppDialog, RotateSecretDialog } from '@/features/oauth-apps';
import { OAuthAppsTable } from '@/widgets/oauth-apps-table';
import { Button } from '@/shared/ui/shadcn/button';

export default function OAuthAppsPage() {
  const [apps, setApps] = useState<OAuthApp[]>([]);
  
  const [createOpen, setCreateOpen] = useState(false);
  const [rotateApp, setRotateApp] = useState<OAuthApp | null>(null);
  const [revokeApp, setRevokeApp] = useState<OAuthApp | null>(null);

  const handleCreateSuccess = (res: { app: OAuthApp }) => {
    setApps([...apps, res.app]);
  };

  const handleRotateSuccess = () => {
    // Usually invalidate queries here
  };

  const handleRevokeSuccess = (appId: string) => {
    setApps(apps.filter(a => a.id !== appId));
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">OAuth App Connections</h2>
          <p className="text-muted-foreground">
            Manage third-party applications, API clients, and external integrations.
          </p>
        </div>
        <Button onClick={() => setCreateOpen(true)}>Create New App</Button>
      </div>

      <OAuthAppsTable
        apps={apps}
        onRotateSecret={setRotateApp}
        onRevokeApp={setRevokeApp}
      />

      <CreateAppDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        onSuccess={handleCreateSuccess}
      />
      
      <RotateSecretDialog
        app={rotateApp}
        open={!!rotateApp}
        onOpenChange={(open) => !open && setRotateApp(null)}
        onSuccess={handleRotateSuccess}
      />

      <RevokeAppDialog
        app={revokeApp}
        open={!!revokeApp}
        onOpenChange={(open) => !open && setRevokeApp(null)}
        onSuccess={handleRevokeSuccess}
      />
    </div>
  );
}
