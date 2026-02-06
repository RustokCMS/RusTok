"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { FormEvent, useState } from "react";
import { useTranslations } from "next-intl";

import { Button } from "@/components/ui/button";

type AuthResponse = { access_token: string };
type InviteAcceptResponse = { email: string; role: string };

export default function RegisterView({ locale }: { locale: string }) {
  const t = useTranslations("auth");
  const e = useTranslations("errors");
  const router = useRouter();
  const apiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:3000";
  const [tenant, setTenant] = useState("demo");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [name, setName] = useState("");
  const [inviteToken, setInviteToken] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isInviteLoading, setIsInviteLoading] = useState(false);

  const onSubmit = async (event: FormEvent) => {
    event.preventDefault();
    setError(null);
    setStatus(null);

    if (!tenant || !email || !password) {
      setError(t("errorRequired"));
      return;
    }

    setIsLoading(true);
    try {
      const response = await fetch(`${apiBaseUrl}/api/auth/register`, {
        method: "POST",
        headers: { "Content-Type": "application/json", "X-Tenant-Slug": tenant },
        body: JSON.stringify({ email, password, name: name || null }),
      });
      if (!response.ok) {
        setError(response.status === 400 ? e("auth.invalid_credentials") : e("http"));
        return;
      }

      const payload = (await response.json()) as AuthResponse;
      document.cookie = `rustok-admin-token=${payload.access_token}; path=/`;
      document.cookie = `rustok-admin-tenant=${tenant}; path=/`;
      router.push(`/${locale}`);
    } catch {
      setError(e("network"));
    } finally {
      setIsLoading(false);
    }
  };


  const onAcceptInvite = async (event: FormEvent) => {
    event.preventDefault();
    setError(null);
    setStatus(null);

    if (!tenant || !inviteToken) {
      setError(t("inviteRequired"));
      return;
    }

    setIsInviteLoading(true);
    try {
      const response = await fetch(`${apiBaseUrl}/api/auth/invite/accept`, {
        method: "POST",
        headers: { "Content-Type": "application/json", "X-Tenant-Slug": tenant },
        body: JSON.stringify({ token: inviteToken }),
      });

      if (!response.ok) {
        setError(response.status === 401 ? t("inviteExpired") : e("http"));
        return;
      }

      const payload = (await response.json()) as InviteAcceptResponse;
      setEmail(payload.email);
      setStatus(`${t("inviteAccepted")} (${payload.role})`);
    } catch {
      setError(e("network"));
    } finally {
      setIsInviteLoading(false);
    }
  };

  return (
    <main className="min-h-screen bg-slate-50">
      <section className="mx-auto max-w-2xl px-6 py-12">
        <form
          className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm"
          onSubmit={onSubmit}
        >
          <h1 className="text-2xl font-semibold">{t("registerTitle")}</h1>
          <p className="mt-2 text-sm text-slate-500">{t("registerSubtitle")}</p>
          {error ? (
            <div className="mt-4 rounded border border-rose-200 bg-rose-50 p-3 text-sm text-rose-600">
              {error}
            </div>
          ) : null}
          {status ? (
            <div className="mt-4 rounded border border-emerald-200 bg-emerald-50 p-3 text-sm text-emerald-700">
              {status}
            </div>
          ) : null}
          <div className="mt-4 grid gap-4">
            <input
              className="input input-bordered"
              placeholder="demo"
              value={tenant}
              onChange={(event) => setTenant(event.target.value)}
            />
            <input
              className="input input-bordered"
              placeholder="admin@rustok.io"
              value={email}
              onChange={(event) => setEmail(event.target.value)}
            />
            <input
              className="input input-bordered"
              placeholder={t("nameLabel")}
              value={name}
              onChange={(event) => setName(event.target.value)}
            />
            <input
              type="password"
              className="input input-bordered"
              placeholder="••••••••"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
            />
          </div>
          <Button className="mt-6 w-full" type="submit" disabled={isLoading}>
            {isLoading ? `${t("registerSubmit")}…` : t("registerSubmit")}
          </Button>
          <div className="mt-4 text-sm">
            <Link href={`/${locale}/login`} className="link link-primary">
              {t("backToLogin")}
            </Link>
          </div>
        </form>

        <form
          className="mt-6 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm"
          onSubmit={onAcceptInvite}
        >
          <h2 className="text-lg font-semibold">{t("inviteTitle")}</h2>
          <p className="mt-2 text-sm text-slate-500">{t("inviteSubtitle")}</p>
          <div className="mt-4 grid gap-4">
            <input
              className="input input-bordered"
              placeholder="INVITE-2024-ABCDE"
              value={inviteToken}
              onChange={(event) => setInviteToken(event.target.value)}
            />
          </div>
          <Button className="mt-6 w-full" type="submit" disabled={isInviteLoading}>
            {isInviteLoading ? `${t("inviteSubmit")}…` : t("inviteSubmit")}
          </Button>
        </form>
      </section>
    </main>
  );
}
