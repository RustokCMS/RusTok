"use client";

import { FormEvent, useMemo, useState } from "react";
import { useTranslations } from "next-intl";

import { Button } from "@/components/ui/button";
import { getClientAuth } from "../../../lib/client-auth";

export default function ProfileView({ locale: _locale }: { locale: string }) {
  const t = useTranslations("profile");
  const apiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:3000";
  const { token, tenant } = useMemo(() => getClientAuth(), []);
  const [name, setName] = useState("");
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const onSubmit = async (event: FormEvent) => {
    event.preventDefault();
    setStatus(null);
    setError(null);

    if (!token) {
      setError(t("missingToken"));
      return;
    }

    const response = await fetch(`${apiBaseUrl}/api/auth/profile`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
        "X-Tenant-Slug": tenant,
      },
      body: JSON.stringify({ name: name || null }),
    });

    if (!response.ok) {
      setError(t("updateFailed"));
      return;
    }

    setStatus(t("saved"));
  };

  return (
    <main className="min-h-screen bg-slate-50">
      <section className="mx-auto max-w-2xl px-6 py-12">
        <form
          className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm"
          onSubmit={onSubmit}
        >
          <h1 className="text-2xl font-semibold">{t("title")}</h1>
          <input
            className="input input-bordered mt-4 w-full"
            placeholder={t("namePlaceholder")}
            value={name}
            onChange={(event) => setName(event.target.value)}
          />
          <Button className="mt-4" type="submit">
            {t("save")}
          </Button>
          {status ? <p className="mt-3 text-sm text-emerald-700">{status}</p> : null}
          {error ? <p className="mt-3 text-sm text-rose-700">{error}</p> : null}
        </form>
      </section>
    </main>
  );
}
