import Link from "next/link";

type Breadcrumb = {
  label: string;
  href?: string;
};

type BreadcrumbsProps = {
  items: Breadcrumb[];
};

export function Breadcrumbs({ items }: BreadcrumbsProps) {
  if (!items.length) {
    return null;
  }

  return (
    <nav aria-label="Breadcrumb" className="text-xs text-slate-500">
      <ol className="flex flex-wrap items-center gap-2">
        {items.map((item) => {
          const isLast = item === items.at(-1);
          const key = item.href ? `${item.label}-${item.href}` : item.label;
          return (
            <li key={key} className="flex items-center gap-2">
              {item.href && !isLast ? (
                <Link className="text-slate-500 hover:text-slate-700" href={item.href}>
                  {item.label}
                </Link>
              ) : (
                <span className="font-medium text-slate-700">{item.label}</span>
              )}
              {!isLast ? <span className="text-slate-300">/</span> : null}
            </li>
          );
        })}
      </ol>
    </nav>
  );
}
