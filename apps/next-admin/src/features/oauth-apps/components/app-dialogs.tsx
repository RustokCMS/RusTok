'use client';

import { useState } from 'react';
import { OAuthApp, CreateOAuthAppResult } from '@/entities/oauth-app';
import { Button } from '@/shared/ui/shadcn/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/shared/ui/shadcn/dialog';
import { Input } from '@/shared/ui/shadcn/input';
import { Label } from '@/shared/ui/shadcn/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/shadcn/select';
import { Textarea } from '@/shared/ui/shadcn/textarea';

export function CreateAppDialog({
  open,
  onOpenChange,
  onSuccess,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: (result: CreateOAuthAppResult) => void;
}) {
  const [name, setName] = useState('');
  const [slug, setSlug] = useState('');
  const [description, setDescription] = useState('');
  const [appType, setAppType] = useState('ThirdParty');
  const [loading, setLoading] = useState(false);

  const [secret, setSecret] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);

    // MOCK: simulate creation
    setTimeout(() => {
      const mockApp: OAuthApp = {
        id: crypto.randomUUID(),
        name,
        slug,
        description,
        appType: appType as any,
        clientId: crypto.randomUUID(),
        redirectUris: [],
        scopes: [],
        grantTypes: ['authorization_code'],
        autoCreated: false,
        isActive: true,
        activeTokenCount: 0,
        createdAt: new Date().toISOString(),
      };

      setSecret('sk_live_nextjs_mock_secret_' + Math.random().toString(36).substring(7));
      onSuccess({ app: mockApp, clientSecret: 'mock' }); // Don't pass real to list, let dialog show it
      setLoading(false);
    }, 600);
  };

  const handleClose = () => {
    onOpenChange(false);
    setSecret(null);
    setName('');
    setSlug('');
    setDescription('');
    setAppType('ThirdParty');
  };

  if (secret) {
    return (
      <Dialog open={open} onOpenChange={handleClose}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="text-green-600">App Created Successfully</DialogTitle>
            <DialogDescription>
              Your new OAuth application has been created. Here is the client secret.
            </DialogDescription>
          </DialogHeader>
          <div className="rounded border bg-muted p-4 font-mono text-sm break-all">
            {secret}
          </div>
          <p className="text-sm font-medium text-destructive">
            Please store this secret safely. You will not be able to see it again.
          </p>
          <DialogFooter>
            <Button onClick={handleClose} className="w-full">
              I have saved it safely
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    );
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Create New App Connection</DialogTitle>
          <DialogDescription>
            Register a new third-party client or first-party integration.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="grid gap-4 py-4">
            <div className="grid gap-2">
              <Label htmlFor="name">App Name</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="My Integration"
                required
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="slug">Slug / Bundle ID</Label>
              <Input
                id="slug"
                value={slug}
                onChange={(e) => setSlug(e.target.value)}
                placeholder="com.example.app"
                required
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Optional description"
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="type">App Type</Label>
              <Select value={appType} onValueChange={setAppType}>
                <SelectTrigger>
                  <SelectValue placeholder="Select type" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="ThirdParty">Third Party (Integration)</SelectItem>
                  <SelectItem value="FirstParty">First Party (Internal)</SelectItem>
                  <SelectItem value="Mobile">Mobile Application</SelectItem>
                  <SelectItem value="Service">Service (Machine-to-Machine)</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" disabled={loading}>
              {loading ? 'Creating...' : 'Create App'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

export function RotateSecretDialog({
  app,
  open,
  onOpenChange,
  onSuccess,
}: {
  app: OAuthApp | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: () => void;
}) {
  const [loading, setLoading] = useState(false);
  const [secret, setSecret] = useState<string | null>(null);

  if (!app) return null;

  const handleRotate = async () => {
    setLoading(true);
    setTimeout(() => {
      setSecret('sk_live_nextjs_rotated_' + Math.random().toString(36).substring(7));
      setLoading(false);
      onSuccess();
    }, 600);
  };

  const handleClose = () => {
    onOpenChange(false);
    setSecret(null);
  };

  if (secret) {
    return (
      <Dialog open={open} onOpenChange={handleClose}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="text-green-600">Secret Rotated</DialogTitle>
            <DialogDescription>
              The old secret has been invalidated. Here is your new client secret.
            </DialogDescription>
          </DialogHeader>
          <div className="rounded border bg-muted p-4 font-mono text-sm break-all">
            {secret}
          </div>
          <p className="text-sm font-medium text-destructive">
            Please store this secret safely. You will not be able to see it again.
          </p>
          <DialogFooter>
            <Button onClick={handleClose} className="w-full">
              I have saved it safely
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    );
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Rotate Client Secret</DialogTitle>
          <DialogDescription>
            Are you sure you want to rotate the secret for <span className="font-semibold">{app.name}</span>?
            <br /><br />
            The old secret will instantly stop working. Client applications will fail to refresh tokens until updated.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button variant="destructive" onClick={handleRotate} disabled={loading}>
            {loading ? 'Rotating...' : 'Yes, Rotate Secret'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export function RevokeAppDialog({
  app,
  open,
  onOpenChange,
  onSuccess,
}: {
  app: OAuthApp | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: (appId: string) => void;
}) {
  const [loading, setLoading] = useState(false);

  if (!app) return null;

  const handleRevoke = async () => {
    setLoading(true);
    setTimeout(() => {
      setLoading(false);
      onSuccess(app.id);
      onOpenChange(false);
    }, 600);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="text-destructive">Revoke Application</DialogTitle>
          <DialogDescription>
            Are you sure you want to completely revoke <span className="font-semibold">{app.name}</span>?
            <br /><br />
            This will immediately invalidate <strong>all existing sessions and tokens</strong> for every user connected to this app.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button variant="destructive" onClick={handleRevoke} disabled={loading}>
            {loading ? 'Revoking...' : 'Revoke Application'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
