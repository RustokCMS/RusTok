"use client";

import { FormEvent, useMemo, useState } from "react";
import { useTranslations } from "next-intl";

import { Button } from "@/components/ui/button";
import { getClientAuth } from "../../../lib/client-auth";

type SessionItem = {
  id: string;
  ip_address?: string;
  user_agent?: string;
  current: boolean;
  created_at: string;
};

export default function SecurityView({ locale: _locale }: { locale: string }) {
  const t = useTranslations("security");
  const apiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:3000";
  const { token, tenant } = useMemo(() => getClientAuth(), []);
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
    if (!token) {
      setStatus(t("missingToken"));
      return;
    }

    const response = await fetch(`${apiBaseUrl}/api/auth/sessions`, { headers });
    if (response.ok) {
      const payload = (await response.json()) as { sessions: SessionItem[] };
      setSessions(payload.sessions);
      return;
    }

    setStatus(t("revokeFailed"));
  };

  const loadHistory = async () => {
    if (!token) {
      setStatus(t("missingToken"));
      return;
    }

    const response = await fetch(`${apiBaseUrl}/api/auth/history`, { headers });
    if (response.ok) {
      const payload = (await response.json()) as { sessions: SessionItem[] };
      setHistory(payload.sessions);
      return;
    }

    setStatus(t("revokeFailed"));
  };

  const onChangePassword = async (event: FormEvent) => {
    event.preventDefault();
    if (!token) {
      setStatus(t("missingToken"));
      return;
    }

    const response = await fetch(`${apiBaseUrl}/api/auth/change-password`, {
      method: "POST",
      headers,
      body: JSON.stringify({
        current_password: currentPassword,
        new_password: newPassword,
      }),
    });
    setStatus(response.ok ? t("passwordUpdated") : t("passwordFailed"));
  };

  const onRevokeAll = async () => {
    if (!token) {
      setStatus(t("missingToken"));
      return;
    }

    const response = await fetch(`${apiBaseUrl}/api/auth/sessions/revoke-all`, {
      method: "POST",
      headers,
      body: "{}",
    });
    setStatus(response.ok ? t("revoked") : t("revokeFailed"));
    await loadSessions();
  };

  return (
    <main className="min-h-screen bg-slate-50">
      <section className="mx-auto max-w-4xl px-6 py-12">
        <h1 className="text-2xl font-semibold">{t("title")}</h1>
        <div className="mt-4 flex gap-3">
          <Button type="button" onClick={loadSessions}>
            {t("loadSessions")}
          </Button>
          <Button type="button" onClick={loadHistory}>
            {t("loadHistory")}
          </Button>
          <Button type="button" onClick={onRevokeAll}>
            {t("signOutAll")}
          </Button>
        </div>

        <form
          className="mt-6 rounded-xl border bg-white p-4"
          onSubmit={onChangePassword}
        >
          <h2 className="font-medium">{t("changePasswordTitle")}</h2>
          <div className="mt-3 grid gap-3 md:grid-cols-2">
            <input
              type="password"
              className="input input-bordered"
              placeholder={t("currentPassword")}
              value={currentPassword}
              onChange={(event) => setCurrentPassword(event.target.value)}
            />
            <input
              type="password"
              className="input input-bordered"
              placeholder={t("newPassword")}
              value={newPassword}
              onChange={(event) => setNewPassword(event.target.value)}
            />
          </div>
          <Button className="mt-3" type="submit">
            {t("changePassword")}
          </Button>
        </form>

        <div className="mt-6 grid gap-4 md:grid-cols-2">
          <div className="rounded-xl border bg-white p-4">
            <h3 className="font-medium">{t("activeSessions")}</h3>
            <ul className="mt-2 space-y-2 text-sm">
              {sessions.map((item) => (
                <li key={item.id} className="rounded border p-2">
                  <div>{item.user_agent ?? t("unknownDevice")}</div>
                  <div>{item.ip_address ?? t("unknownIp")}</div>
                  <div>{item.current ? t("current") : t("other")}</div>
                </li>
              ))}
            </ul>
          </div>
          <div className="rounded-xl border bg-white p-4">
            <h3 className="font-medium">{t("loginHistory")}</h3>
            <ul className="mt-2 space-y-2 text-sm">
              {history.map((item) => (
                <li key={item.id} className="rounded border p-2">
                  <div>{item.user_agent ?? t("unknownDevice")}</div>
                  <div>{item.ip_address ?? t("unknownIp")}</div>
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
