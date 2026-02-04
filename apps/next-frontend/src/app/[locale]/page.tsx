import { CheckCircle2, Rocket, Sparkles } from "lucide-react";
import { getTranslations } from "next-intl/server";

import { Button } from "@/components/ui/button";

export default async function StorefrontHome() {
  const t = await getTranslations("Storefront");
  const features = t.raw("features") as string[];
  const chips = t.raw("chips") as string[];

  return (
    <main className="min-h-screen bg-white">
      <section className="mx-auto max-w-6xl px-6 py-16">
        <div className="grid gap-10 lg:grid-cols-[1.1fr_0.9fr] lg:items-center">
          <div>
            <p className="text-sm font-semibold uppercase tracking-wide text-sky-600">
              {t("eyebrow")}
            </p>
            <h1 className="mt-3 text-4xl font-semibold text-slate-900 sm:text-5xl">
              {t("title")}
            </h1>
            <p className="mt-4 text-lg text-slate-600">
              {t("subtitle")}
            </p>
            <div className="mt-6 flex flex-wrap gap-3">
              <Button size="lg">{t("primaryCta")}</Button>
              <Button size="lg" variant="outline">
                {t("secondaryCta")}
              </Button>
            </div>
            <ul className="mt-8 space-y-3 text-sm text-slate-600">
              {features.map((feature) => (
                <li key={feature} className="flex items-center gap-2">
                  <CheckCircle2 className="h-4 w-4 text-emerald-500" />
                  {feature}
                </li>
              ))}
            </ul>
          </div>
          <div className="rounded-3xl border border-slate-200 bg-slate-50 p-6 shadow-sm">
            <div className="space-y-6">
              <div className="rounded-2xl bg-white p-4 shadow-sm">
                <div className="flex items-center gap-3">
                  <div className="flex h-10 w-10 items-center justify-center rounded-full bg-sky-100 text-sky-600">
                    <Rocket className="h-5 w-5" />
                  </div>
                  <div>
                    <p className="text-sm font-semibold text-slate-900">
                      {t("cards.launchTitle")}
                    </p>
                    <p className="text-xs text-slate-500">
                      {t("cards.launchDescription")}
                    </p>
                  </div>
                </div>
              </div>
              <div className="rounded-2xl bg-white p-4 shadow-sm">
                <div className="flex items-center gap-3">
                  <div className="flex h-10 w-10 items-center justify-center rounded-full bg-amber-100 text-amber-600">
                    <Sparkles className="h-5 w-5" />
                  </div>
                  <div>
                    <p className="text-sm font-semibold text-slate-900">
                      {t("cards.flexTitle")}
                    </p>
                    <p className="text-xs text-slate-500">
                      {t("cards.flexDescription")}
                    </p>
                  </div>
                </div>
              </div>
              <div className="rounded-2xl bg-white p-4 shadow-sm">
                <p className="text-sm font-semibold text-slate-900">
                  {t("cards.stackTitle")}
                </p>
                <div className="mt-3 flex flex-wrap gap-2 text-xs">
                  {chips.map((chip) => (
                    <span
                      key={chip}
                      className="badge badge-outline border-slate-200 text-slate-600"
                    >
                      {chip}
                    </span>
                  ))}
                </div>
              </div>
              <div className="alert alert-info">
                <div>
                  <h3 className="font-semibold">{t("alertTitle")}</h3>
                  <p className="text-sm opacity-80">{t("alertDescription")}</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>
    </main>
  );
}
