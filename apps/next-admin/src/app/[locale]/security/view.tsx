"use client";

import { FormEvent, useMemo, useState } from "react";

import { Button } from "@/components/ui/button";

type SessionItem = {
  id: string;
  ip_address?: string;
  user_agent?: string;
  current: boolean;
  created_at: string;
};

function getCookieValue(name: string) {
  const pair = document.cookie.split("; ").find((row) => row.startsWith(`${name}=`));
  return pair ? decodeURIComponent(pair.split("=")[1]) : undefined;
}

export default function SecurityView({ locale: _locale }: { locale: string }) {
  const apiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:3000";
  const token = useMemo(() => getCookieValue("rustok-admin-token"), []);
  const tenant = useMemo(() => getCookieValue("rustok-admin-tenant") ?? "demo", []);
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [sessions, setSessions] = useState<SessionItem[]>([]);
  const [history, setHistory] = useState<SessionItem[]>([]);
  const [status, setStatus] = useState<string | null>(null);

  const headers = {
    "Content-Type": "application/json",
    Authorization: `Bearer ${token ?? ""}`,
    "X-Tenant-Slug": tenant,
  };

  const loadSessions = async () => {
    if (!token) return;
    const response = await fetch(`${apiBaseUrl}/api/auth/sessions`, { headers });
    if (response.ok) {
      const payload = (await response.json()) as { sessions: SessionItem[] };
      setSessions(payload.sessions);
    }
  };

  const loadHistory = async () => {
    if (!token) return;
    const response = await fetch(`${apiBaseUrl}/api/auth/history`, { headers });
    if (response.ok) {
      const payload = (await response.json()) as { sessions: SessionItem[] };
      setHistory(payload.sessions);
    }
  };

  const onChangePassword = async (event: FormEvent) => {
    event.preventDefault();
    if (!token) return;
    const response = await fetch(`${apiBaseUrl}/api/auth/change-password`, {
      method: "POST",
      headers,
      body: JSON.stringify({ current_password: currentPassword, new_password: newPassword }),
    });
    setStatus(response.ok ? "Password updated" : "Failed to update password");
  };

  const onRevokeAll = async () => {
    if (!token) return;
    const response = await fetch(`${apiBaseUrl}/api/auth/sessions/revoke-all`, {
      method: "POST",
      headers,
      body: "{}",
    });
    setStatus(response.ok ? "Other sessions revoked" : "Failed to revoke sessions");
    await loadSessions();
  };

  return (
    <main className="min-h-screen bg-slate-50">
      <section className="mx-auto max-w-4xl px-6 py-12">
        <h1 className="text-2xl font-semibold">Security</h1>
        <div className="mt-4 flex gap-3">
          <Button type="button" onClick={loadSessions}>Load sessions</Button>
          <Button type="button" onClick={loadHistory}>Load history</Button>
          <Button type="button" onClick={onRevokeAll}>Sign out all</Button>
        </div>

        <form className="mt-6 rounded-xl border bg-white p-4" onSubmit={onChangePassword}>
          <h2 className="font-medium">Change password</h2>
          <div className="mt-3 grid gap-3 md:grid-cols-2">
            <input type="password" className="input input-bordered" placeholder="Current password" value={currentPassword} onChange={(e) => setCurrentPassword(e.target.value)} />
            <input type="password" className="input input-bordered" placeholder="New password" value={newPassword} onChange={(e) => setNewPassword(e.target.value)} />
          </div>
          <Button className="mt-3" type="submit">Update password</Button>
        </form>

        <div className="mt-6 grid gap-4 md:grid-cols-2">
          <div className="rounded-xl border bg-white p-4">
            <h3 className="font-medium">Active sessions</h3>
            <ul className="mt-2 space-y-2 text-sm">
              {sessions.map((item) => (
                <li key={item.id} className="rounded border p-2">
                  <div>{item.user_agent ?? "Unknown device"}</div>
                  <div>{item.ip_address ?? "Unknown IP"}</div>
                  <div>{item.current ? "Current" : "Other"}</div>
                </li>
              ))}
            </ul>
          </div>
          <div className="rounded-xl border bg-white p-4">
            <h3 className="font-medium">Login history</h3>
            <ul className="mt-2 space-y-2 text-sm">
              {history.map((item) => (
                <li key={item.id} className="rounded border p-2">
                  <div>{item.user_agent ?? "Unknown device"}</div>
                  <div>{item.ip_address ?? "Unknown IP"}</div>
                  <div>{new Date(item.created_at).toLocaleString()}</div>
                </li>
              ))}
            </ul>
          </div>
        </div>
        {status ? <p className="mt-4 text-sm text-slate-700">{status}</p> : null}
      </section>
    </main>
  );
}
