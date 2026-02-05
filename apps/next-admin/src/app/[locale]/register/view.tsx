"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { FormEvent, useState } from "react";
import { useTranslations } from "next-intl";

import { Button } from "@/components/ui/button";

type AuthResponse = { access_token: string };

export default function RegisterView({ locale }: { locale: string }) {
  const t = useTranslations("auth");
  const e = useTranslations("errors");
  const router = useRouter();
  const apiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:3000";
  const [tenant, setTenant] = useState("demo");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const onSubmit = async (event: FormEvent) => {
    event.preventDefault();
    setError(null);
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

  return (
    <main className="min-h-screen bg-slate-50">
      <section className="mx-auto max-w-2xl px-6 py-12">
        <form className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm" onSubmit={onSubmit}>
          <h1 className="text-2xl font-semibold">{t("title")}</h1>
          {error ? <div className="mt-4 rounded border border-rose-200 bg-rose-50 p-3 text-sm text-rose-600">{error}</div> : null}
          <div className="mt-4 grid gap-4">
            <input className="input input-bordered" placeholder="demo" value={tenant} onChange={(e) => setTenant(e.target.value)} />
            <input className="input input-bordered" placeholder="admin@rustok.io" value={email} onChange={(e) => setEmail(e.target.value)} />
            <input className="input input-bordered" placeholder="Name" value={name} onChange={(e) => setName(e.target.value)} />
            <input type="password" className="input input-bordered" placeholder="••••••••" value={password} onChange={(e) => setPassword(e.target.value)} />
          </div>
          <Button className="mt-6 w-full" type="submit" disabled={isLoading}>{isLoading ? `${t("submit")}…` : t("submit")}</Button>
          <div className="mt-4 text-sm"><Link href={`/${locale}/login`} className="link link-primary">{t("title")}</Link></div>
        </form>
      </section>
    </main>
  );
}
