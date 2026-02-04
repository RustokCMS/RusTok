import { type ReactNode } from "react";
import { cn } from "@/components/utils";

interface OverviewCardProps {
  title: string;
  value: string;
  description: string;
  icon?: ReactNode;
  accent?: "indigo" | "emerald" | "amber";
}

const accents: Record<NonNullable<OverviewCardProps["accent"]>, string> = {
  indigo: "border-indigo-200 bg-indigo-50 text-indigo-700",
  emerald: "border-emerald-200 bg-emerald-50 text-emerald-700",
  amber: "border-amber-200 bg-amber-50 text-amber-700",
};

export function OverviewCard({
  title,
  value,
  description,
  icon,
  accent = "indigo",
}: OverviewCardProps) {
  return (
    <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium text-slate-500">{title}</p>
          <p className="mt-2 text-2xl font-semibold text-slate-900">{value}</p>
        </div>
        <div
          className={cn(
            "flex h-12 w-12 items-center justify-center rounded-xl border",
            accents[accent]
          )}
        >
          {icon}
        </div>
      </div>
      <p className="mt-3 text-sm text-slate-500">{description}</p>
    </section>
  );
}
