"use client";

import { FormEvent, useState } from "react";
import { useTranslations } from "next-intl";

import { Button } from "@/components/ui/button";

export default function ResetView({ locale: _locale }: { locale: string }) {
  const t = useTranslations("auth");
  const e = useTranslations("errors");
  const apiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:3000";
  const [tenant, setTenant] = useState("demo");
  const [email, setEmail] = useState("");
  const [token, setToken] = useState("");
  const [password, setPassword] = useState("");
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const onRequest = async (event: FormEvent) => {
    event.preventDefault();
    setError(null);
    setStatus(null);
    try {
      const response = await fetch(`${apiBaseUrl}/api/auth/reset/request`, {
        method: "POST",
        headers: { "Content-Type": "application/json", "X-Tenant-Slug": tenant },
        body: JSON.stringify({ email }),
      });
      const payload = (await response.json()) as { reset_token?: string };
      setStatus(payload.reset_token ? `Reset token: ${payload.reset_token}` : "Reset requested");
    } catch {
      setError(e("network"));
    }
  };

  const onConfirm = async (event: FormEvent) => {
    event.preventDefault();
    setError(null);
    setStatus(null);
    try {
      const response = await fetch(`${apiBaseUrl}/api/auth/reset/confirm`, {
        method: "POST",
        headers: { "Content-Type": "application/json", "X-Tenant-Slug": tenant },
        body: JSON.stringify({ token, password }),
      });
      if (!response.ok) {
        setError(e("auth.unauthorized"));
        return;
      }
      setStatus("Password updated");
    } catch {
      setError(e("network"));
    }
  };

  return (
    <main className="min-h-screen bg-slate-50">
      <section className="mx-auto grid max-w-4xl gap-6 px-6 py-12 lg:grid-cols-2">
        <form className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm" onSubmit={onRequest}>
          <h2 className="text-lg font-semibold">{t("resetLink")}</h2>
          <div className="mt-4 grid gap-3">
            <input className="input input-bordered" placeholder="demo" value={tenant} onChange={(e) => setTenant(e.target.value)} />
            <input className="input input-bordered" placeholder="admin@rustok.io" value={email} onChange={(e) => setEmail(e.target.value)} />
          </div>
          <Button className="mt-4 w-full" type="submit">Request reset</Button>
        </form>
        <form className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm" onSubmit={onConfirm}>
          <h2 className="text-lg font-semibold">Confirm reset</h2>
          <div className="mt-4 grid gap-3">
            <input className="input input-bordered" placeholder="token" value={token} onChange={(e) => setToken(e.target.value)} />
            <input type="password" className="input input-bordered" placeholder="new password" value={password} onChange={(e) => setPassword(e.target.value)} />
          </div>
          <Button className="mt-4 w-full" type="submit">Update password</Button>
        </form>
        {status ? <p className="text-sm text-emerald-700">{status}</p> : null}
        {error ? <p className="text-sm text-rose-700">{error}</p> : null}
      </section>
    </main>
  );
}
