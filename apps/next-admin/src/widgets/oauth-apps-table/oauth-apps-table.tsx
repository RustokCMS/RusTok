'use client';

import { OAuthApp, OAuthAppTypeBadge } from '@/entities/oauth-app';
import { Button } from '@/shared/ui/shadcn/button';

export function OAuthAppsTable({
  apps,
  onRotateSecret,
  onRevokeApp,
}: {
  apps: OAuthApp[];
  onRotateSecret: (app: OAuthApp) => void;
  onRevokeApp: (app: OAuthApp) => void;
}) {
  return (
    <div className="rounded-md border overflow-x-auto">
      <table className="w-full text-left text-sm whitespace-nowrap">
        <thead className="bg-muted/50 text-muted-foreground uppercase text-xs">
          <tr>
            <th className="px-4 py-3 font-medium">Name</th>
            <th className="px-4 py-3 font-medium">Type</th>
            <th className="px-4 py-3 font-medium">Client ID</th>
            <th className="px-4 py-3 font-medium">Active Tokens</th>
            <th className="px-4 py-3 font-medium text-right">Actions</th>
          </tr>
        </thead>
        <tbody className="divide-y">
          {apps.map((app) => (
            <tr key={app.id} className="hover:bg-muted/50 transition-colors">
              <td className="px-4 py-3 font-medium text-slate-900">{app.name}</td>
              <td className="px-4 py-3">
                <OAuthAppTypeBadge appType={app.appType} />
              </td>
              <td className="px-4 py-3 text-slate-500 font-mono text-xs">{app.clientId}</td>
              <td className="px-4 py-3 text-slate-500">{app.activeTokenCount}</td>
              <td className="px-4 py-3 text-right space-x-2">
                <Button variant="outline" size="sm" onClick={() => onRotateSecret(app)}>
                  Rotate Secret
                </Button>
                <Button variant="destructive" size="sm" onClick={() => onRevokeApp(app)}>
                  Revoke
                </Button>
              </td>
            </tr>
          ))}
          {apps.length === 0 && (
            <tr>
              <td colSpan={5} className="h-24 text-center text-slate-500">
                No connections found. Connect an app to get started.
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
