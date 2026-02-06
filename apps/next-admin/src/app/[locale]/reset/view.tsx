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
  const [tokenExpired, setTokenExpired] = useState(false);

  const onRequest = async (event: FormEvent) => {
    event.preventDefault();
    setError(null);
    setStatus(null);
    setTokenExpired(false);

    try {
      const response = await fetch(`${apiBaseUrl}/api/auth/reset/request`, {
        method: "POST",
        headers: { "Content-Type": "application/json", "X-Tenant-Slug": tenant },
        body: JSON.stringify({ email }),
      });

      if (!response.ok) {
        setError(e("http"));
        setTokenExpired(false);
        return;
      }

      const payload = (await response.json()) as { reset_token?: string };
      if (payload.reset_token) {
        setToken(payload.reset_token);
        setStatus(t("resetTokenPreview", { token: payload.reset_token }));
      } else {
        setStatus(t("resetRequested"));
      }
    } catch {
      setError(e("network"));
      setTokenExpired(false);
    }
  };

  const onConfirm = async (event: FormEvent) => {
    event.preventDefault();
    setError(null);
    setStatus(null);
    setTokenExpired(false);

    try {
      const response = await fetch(`${apiBaseUrl}/api/auth/reset/confirm`, {
        method: "POST",
        headers: { "Content-Type": "application/json", "X-Tenant-Slug": tenant },
        body: JSON.stringify({ token, password }),
      });

      if (!response.ok) {
        if (response.status === 401) {
          setError(t("resetTokenExpired"));
          setTokenExpired(true);
          return;
        }

        setError(e("http"));
        return;
      }

      setStatus(t("passwordUpdated"));
      setTokenExpired(false);
    } catch {
      setError(e("network"));
      setTokenExpired(false);
    }
  };

  return (
    <main className="min-h-screen bg-slate-50">
      <section className="mx-auto grid max-w-4xl gap-6 px-6 py-12 lg:grid-cols-2">
        <form
          className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm"
          onSubmit={onRequest}
        >
          <h2 className="text-lg font-semibold">{t("resetRequestTitle")}</h2>
          <div className="mt-4 grid gap-3">
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
          </div>
          <Button className="mt-4 w-full" type="submit">
            {t("resetRequestSubmit")}
          </Button>
        </form>

        <form
          className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm"
          onSubmit={onConfirm}
        >
          <h2 className="text-lg font-semibold">{t("resetConfirmTitle")}</h2>
          <div className="mt-4 grid gap-3">
            <input
              className="input input-bordered"
              placeholder={t("resetTokenLabel")}
              value={token}
              onChange={(event) => setToken(event.target.value)}
            />
            <input
              type="password"
              className="input input-bordered"
              placeholder={t("newPasswordLabel")}
              value={password}
              onChange={(event) => setPassword(event.target.value)}
            />
          </div>
          <Button className="mt-4 w-full" type="submit">
            {t("resetConfirmSubmit")}
          </Button>
          {tokenExpired ? (
            <p className="mt-3 text-sm text-amber-700">{t("resetTokenExpiredRecovery")}</p>
          ) : null}
        </form>

        {status ? <p className="text-sm text-emerald-700">{status}</p> : null}
        {error ? <p className="text-sm text-rose-700">{error}</p> : null}
      </section>
    </main>
  );
}
