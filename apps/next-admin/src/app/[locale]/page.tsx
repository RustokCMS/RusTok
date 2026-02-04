import { Activity, Bell, Boxes, Briefcase, LineChart, Users } from "lucide-react";
import Link from "next/link";
import { getLocale, getTranslations } from "next-intl/server";

import { OverviewCard } from "@/components/overview-card";
import { Button } from "@/components/ui/button";

export default async function AdminHome() {
  const t = await getTranslations("app.dashboard");
  const locale = await getLocale();
  const quickActions = t.raw("quickActions") as Array<{
    title: string;
    description: string;
  }>;
  const quickActionIcons = [Boxes, Users, Briefcase];
  const quickActionLinks = [undefined, `/${locale}/users`, undefined];

  return (
    <main className="min-h-screen bg-slate-50">
      <section className="mx-auto max-w-6xl px-6 py-12">
        <header className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <p className="text-sm font-medium text-indigo-600">
              {t("eyebrow")}
            </p>
            <h1 className="mt-2 text-3xl font-semibold text-slate-900">
              {t("title")}
            </h1>
            <p className="mt-2 max-w-2xl text-sm text-slate-500">
              {t("subtitle")}
            </p>
          </div>
          <Button size="md" variant="primary">
            <Bell className="h-4 w-4" />
            {t("notifications")}
          </Button>
        </header>

        <div className="mt-8 alert alert-info">
          <div>
            <h3 className="font-semibold">{t("alertTitle")}</h3>
            <p className="text-sm opacity-80">{t("alertDescription")}</p>
          </div>
        </div>

        <div className="mt-10 grid gap-6 md:grid-cols-3">
          <OverviewCard
            title={t("stats.requests")}
            value={t("stats.requestsValue")}
            description={t("stats.requestsDescription")}
            icon={<LineChart className="h-5 w-5" />}
            accent="indigo"
          />
          <OverviewCard
            title={t("stats.modules")}
            value={t("stats.modulesValue")}
            description={t("stats.modulesDescription")}
            icon={<Boxes className="h-5 w-5" />}
            accent="emerald"
          />
          <OverviewCard
            title={t("stats.events")}
            value={t("stats.eventsValue")}
            description={t("stats.eventsDescription")}
            icon={<Activity className="h-5 w-5" />}
            accent="amber"
          />
        </div>

        <section className="mt-12 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-lg font-semibold text-slate-900">
                {t("quickActionsTitle")}
              </h2>
              <p className="text-sm text-slate-500">
                {t("quickActionsDescription")}
              </p>
            </div>
            <span className="badge badge-outline">MVP</span>
          </div>
          <div className="mt-6 grid gap-4 md:grid-cols-3">
            {quickActions.map((action, index) => {
              const Icon = quickActionIcons[index] ?? Briefcase;
              const href = quickActionLinks[index];
              const card = (
                <div className="rounded-xl border border-slate-200 bg-slate-50 p-4 transition hover:border-indigo-200 hover:bg-white">
                  <Icon className="h-5 w-5 text-indigo-600" />
                  <h3 className="mt-3 text-sm font-semibold text-slate-900">
                    {action.title}
                  </h3>
                  <p className="mt-2 text-sm text-slate-500">
                    {action.description}
                  </p>
                </div>
              );

              return href ? (
                <Link key={action.title} href={href}>
                  {card}
                </Link>
              ) : (
                <div key={action.title}>{card}</div>
              );
            })}
          </div>
        </section>
      </section>
    </main>
  );
}
