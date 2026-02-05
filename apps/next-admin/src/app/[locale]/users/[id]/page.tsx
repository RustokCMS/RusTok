import Link from "next/link";
import { cookies } from "next/headers";
import { getLocale, getTranslations } from "next-intl/server";

import { PageHeader } from "@/components/ui/page-header";
type GraphqlUser = {
  id: string;
  email: string;
  name: string | null;
  role: string;
  status: string;
  createdAt: string;
};

type GraphqlUserResponse = {
  data?: { user: GraphqlUser | null };
  errors?: Array<{ message: string }>;
};

type FetchError = {
  kind: "http" | "network" | "graphql";
  status?: number;
  message?: string;
};

const graphqlQuery = `query User($id: ID!) {
  user(id: $id) {
    id
    email
    name
    role
    status
    createdAt
  }
}`;

const apiBaseUrl =
  process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:3000";
const apiToken = process.env.ADMIN_API_TOKEN;
const tenantSlug = process.env.ADMIN_TENANT_SLUG;

const buildHeaders = () => {
  const cookieStore = cookies();
  const cookieToken = cookieStore.get("rustok-admin-token")?.value;
  const cookieTenant = cookieStore.get("rustok-admin-tenant")?.value;
  const resolvedToken = cookieToken ?? apiToken;
  const resolvedTenant = cookieTenant ?? tenantSlug;
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };

  if (resolvedToken) {
    headers.Authorization = `Bearer ${resolvedToken}`;
  }

  if (resolvedTenant) {
    headers["X-Tenant-Slug"] = resolvedTenant;
  }

  return headers;
};

async function fetchGraphqlUser(id: string) {
  try {
    const response = await fetch(`${apiBaseUrl}/api/graphql`, {
      method: "POST",
      headers: buildHeaders(),
      body: JSON.stringify({
        query: graphqlQuery,
        variables: { id },
      }),
    });

    if (!response.ok) {
      return { error: { kind: "http", status: response.status } satisfies FetchError };
    }

    const payload = (await response.json()) as GraphqlUserResponse;
    if (payload.errors?.length) {
      return {
        error: {
          kind: "graphql",
          message: payload.errors[0]?.message ?? "GraphQL error",
        } satisfies FetchError,
      };
    }

    return { data: payload.data?.user ?? null };
  } catch (error) {
    return { error: { kind: "network" } satisfies FetchError };
  }
}

type UsersDetailPageProps = {
  params: { id: string };
};

export default async function UsersDetailPage({ params }: UsersDetailPageProps) {
  const t = await getTranslations("users");
  const tErrors = await getTranslations("errors");
  const locale = await getLocale();
  const result = await fetchGraphqlUser(params.id);
  const formatError = (error: FetchError) => {
    switch (error.kind) {
      case "http":
        return error.status ? `${tErrors("http")} ${error.status}` : tErrors("http");
      case "graphql":
        return error.message
          ? `${tErrors("unknown")} ${error.message}`
          : tErrors("unknown");
      case "network":
      default:
        return tErrors("network");
    }
  };

  return (
    <main className="min-h-screen bg-slate-50">
      <section className="mx-auto max-w-5xl px-6 py-12">
        <PageHeader
          eyebrow={t("eyebrow")}
          title={t("detail.title")}
          subtitle={t("detail.subtitle")}
          breadcrumbs={[
            { label: t("back"), href: `/${locale}` },
            { label: t("title"), href: `/${locale}/users` },
            { label: t("detail.title") },
          ]}
          actions={
            <Link className="btn btn-outline" href={`/${locale}/users`}>
              {t("detail.back")}
            </Link>
          }
        />

        <section className="mt-8 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
          <h2 className="text-lg font-semibold text-slate-900">
            {t("detail.section")}
          </h2>
          {result.error ? (
            <div className="mt-4 rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-600">
              {formatError(result.error)}
            </div>
          ) : result.data ? (
            <div className="mt-4 grid gap-4 text-sm text-slate-600 md:grid-cols-2">
              <div>
                <p className="text-xs font-semibold uppercase text-slate-400">
                  {t("detail.fields.email")}
                </p>
                <p className="mt-2 text-base text-slate-900">
                  {result.data.email}
                </p>
              </div>
              <div>
                <p className="text-xs font-semibold uppercase text-slate-400">
                  {t("detail.fields.name")}
                </p>
                <p className="mt-2 text-base text-slate-900">
                  {result.data.name ?? t("rest.noName")}
                </p>
              </div>
              <div>
                <p className="text-xs font-semibold uppercase text-slate-400">
                  {t("detail.fields.role")}
                </p>
                <p className="mt-2 text-base text-slate-900">
                  {result.data.role}
                </p>
              </div>
              <div>
                <p className="text-xs font-semibold uppercase text-slate-400">
                  {t("detail.fields.status")}
                </p>
                <p className="mt-2 text-base text-slate-900">
                  {result.data.status}
                </p>
              </div>
              <div>
                <p className="text-xs font-semibold uppercase text-slate-400">
                  {t("detail.fields.createdAt")}
                </p>
                <p className="mt-2 text-base text-slate-900">
                  {result.data.createdAt}
                </p>
              </div>
              <div>
                <p className="text-xs font-semibold uppercase text-slate-400">
                  {t("detail.fields.id")}
                </p>
                <p className="mt-2 text-base text-slate-900">{result.data.id}</p>
              </div>
            </div>
          ) : (
            <div className="mt-4 rounded-xl border border-slate-200 bg-slate-50 p-4 text-sm text-slate-500">
              {t("detail.empty")}
            </div>
          )}
        </section>
      </section>
    </main>
  );
}
