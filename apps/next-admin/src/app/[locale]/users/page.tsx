import Link from "next/link";
import { cookies } from "next/headers";
import { getLocale, getTranslations } from "next-intl/server";

import { PageHeader } from "@/components/ui/page-header";
type RestUser = {
  id: string;
  email: string;
  name: string | null;
  role: string;
};

type GraphqlUser = {
  id: string;
  email: string;
  name: string | null;
  role: string;
  status: string;
  createdAt: string;
};

type FetchError = {
  kind: "http" | "network" | "graphql";
  status?: number;
  message?: string;
};

type GraphqlUsersResponse = {
  data?: {
    users: {
      edges: Array<{ node: GraphqlUser }>;
      pageInfo: { totalCount: number };
    };
  };
  errors?: Array<{ message: string }>;
};

const graphqlQuery = `query Users($pagination: PaginationInput, $filter: UsersFilter, $search: String) {
  users(pagination: $pagination, filter: $filter, search: $search) {
    edges {
      node {
        id
        email
        name
        role
        status
        createdAt
      }
    }
    pageInfo {
      totalCount
    }
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

async function fetchRestUser() {
  try {
    const response = await fetch(`${apiBaseUrl}/api/auth/me`, {
      headers: buildHeaders(),
    });

    if (!response.ok) {
      return { error: { kind: "http", status: response.status } satisfies FetchError };
    }

    const data = (await response.json()) as RestUser;
    return { data };
  } catch (error) {
    return { error: { kind: "network" } satisfies FetchError };
  }
}

type UsersSearchParams = {
  page?: string;
  search?: string;
  role?: string;
  status?: string;
};

async function fetchGraphqlUsers(options: {
  page: number;
  search?: string;
  role?: string;
  status?: string;
  limit: number;
}) {
  const { page, search, role, status, limit } = options;
  const offset = Math.max(0, page - 1) * limit;
  const filter =
    role || status
      ? {
          role: role || null,
          status: status || null,
        }
      : null;
  try {
    const response = await fetch(`${apiBaseUrl}/api/graphql`, {
      method: "POST",
      headers: buildHeaders(),
      body: JSON.stringify({
        query: graphqlQuery,
        variables: {
          pagination: { offset, limit },
          filter,
          search: search || null,
        },
      }),
    });

    if (!response.ok) {
      return { error: { kind: "http", status: response.status } satisfies FetchError };
    }

    const payload = (await response.json()) as GraphqlUsersResponse;
    if (payload.errors?.length) {
      return {
        error: {
          kind: "graphql",
          message: payload.errors[0]?.message ?? "GraphQL error",
        } satisfies FetchError,
      };
    }

    return { data: payload.data?.users };
  } catch (error) {
    return { error: { kind: "network" } satisfies FetchError };
  }
}

type UsersPageProps = {
  searchParams?: UsersSearchParams;
};

export default async function UsersPage({ searchParams }: UsersPageProps) {
  const t = await getTranslations("users");
  const tErrors = await getTranslations("errors");
  const locale = await getLocale();
  const requestedPage = Number(searchParams?.page ?? 1) || 1;
  const search = searchParams?.search?.trim();
  const role = searchParams?.role?.trim();
  const status = searchParams?.status?.trim();
  const limit = 8;
  const [restResult, graphqlResult] = await Promise.all([
    fetchRestUser(),
    fetchGraphqlUsers({
      page: Math.max(1, requestedPage),
      search,
      role,
      status,
      limit,
    }),
  ]);
  const totalCount = graphqlResult.data?.pageInfo.totalCount ?? 0;
  const totalPages = Math.max(1, Math.ceil(totalCount / limit));
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
  const currentPage = Math.min(Math.max(1, requestedPage), totalPages);
  const buildQueryString = (overrides: Partial<UsersSearchParams>) => {
    const params = new URLSearchParams();
    const nextPage = overrides.page ?? String(currentPage);
    const nextSearch = overrides.search ?? search ?? "";
    const nextRole = overrides.role ?? role ?? "";
    const nextStatus = overrides.status ?? status ?? "";

    if (nextPage && nextPage !== "1") params.set("page", nextPage);
    if (nextSearch) params.set("search", nextSearch);
    if (nextRole) params.set("role", nextRole);
    if (nextStatus) params.set("status", nextStatus);

    const query = params.toString();
    return query ? `?${query}` : "";
  };

  return (
    <main className="min-h-screen bg-slate-50">
      <section className="mx-auto max-w-6xl px-6 py-12">
        <PageHeader
          eyebrow={t("eyebrow")}
          title={t("title")}
          subtitle={t("subtitle")}
          breadcrumbs={[
            { label: t("back"), href: `/${locale}` },
            { label: t("title") },
          ]}
          actions={
            <a className="btn btn-outline" href={`/${locale}`}>
              {t("back")}
            </a>
          }
        />

        <section className="mt-8 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
          <h2 className="text-lg font-semibold text-slate-900">
            {t("access.title")}
          </h2>
          <p className="mt-2 text-sm text-slate-500">
            {t("access.description")}
          </p>
          <div className="mt-4 grid gap-4 rounded-xl border border-dashed border-slate-200 bg-slate-50 p-4 text-sm text-slate-600 md:grid-cols-2">
            <div>
              <p className="font-medium">{t("access.restTitle")}</p>
              <p className="mt-1">{t("access.restHint")}</p>
            </div>
            <div>
              <p className="font-medium">{t("access.graphqlTitle")}</p>
              <p className="mt-1">{t("access.graphqlHint")}</p>
            </div>
          </div>
        </section>

        <div className="mt-10 grid gap-6 lg:grid-cols-2">
          <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
            <h3 className="text-sm font-semibold uppercase text-slate-500">
              {t("rest.title")}
            </h3>
            <p className="mt-2 text-sm text-slate-500">{t("rest.subtitle")}</p>
            {restResult.error ? (
              <div className="mt-4 rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-600">
                {formatError(restResult.error)}
              </div>
            ) : (
              <div className="mt-4 rounded-xl border border-slate-200 bg-slate-50 p-4">
                <p className="text-sm font-semibold text-slate-900">
                  {restResult.data?.email}
                </p>
                <p className="mt-1 text-sm text-slate-500">
                  {restResult.data?.name ?? t("rest.noName")}
                </p>
                <div className="mt-3 flex flex-wrap items-center gap-2 text-xs text-slate-500">
                  <span className="badge badge-outline">
                    {restResult.data?.role}
                  </span>
                  <span>{restResult.data?.id}</span>
                </div>
              </div>
            )}
          </section>

          <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
            <h3 className="text-sm font-semibold uppercase text-slate-500">
              {t("graphql.title")}
            </h3>
            <p className="mt-2 text-sm text-slate-500">
              {t("graphql.subtitle")}
            </p>
            {graphqlResult.error ? (
              <div className="mt-4 rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-600">
                {formatError(graphqlResult.error)}
              </div>
            ) : (
              <div className="mt-4 overflow-x-auto rounded-xl border border-slate-200">
                <form
                  className="border-b border-slate-100 bg-slate-50 px-4 py-3"
                  method="get"
                >
                  <div className="grid gap-3 md:grid-cols-3">
                    <label className="text-xs font-semibold uppercase text-slate-400">
                      {t("filters.search")}
                      <input
                        className="mt-2 w-full rounded-lg border border-slate-200 px-3 py-2 text-sm text-slate-900 focus:border-indigo-500 focus:outline-none"
                        defaultValue={search ?? ""}
                        name="search"
                        placeholder={t("filters.searchPlaceholder")}
                      />
                    </label>
                    <label className="text-xs font-semibold uppercase text-slate-400">
                      {t("filters.role")}
                      <select
                        className="mt-2 w-full rounded-lg border border-slate-200 px-3 py-2 text-sm text-slate-900"
                        defaultValue={role ?? ""}
                        name="role"
                      >
                        <option value="">{t("filters.rolePlaceholder")}</option>
                        <option value="SUPER_ADMIN">
                          {t("filters.roleSuperAdmin")}
                        </option>
                        <option value="ADMIN">{t("filters.roleAdmin")}</option>
                        <option value="MANAGER">{t("filters.roleManager")}</option>
                        <option value="CUSTOMER">{t("filters.roleCustomer")}</option>
                      </select>
                    </label>
                    <label className="text-xs font-semibold uppercase text-slate-400">
                      {t("filters.status")}
                      <select
                        className="mt-2 w-full rounded-lg border border-slate-200 px-3 py-2 text-sm text-slate-900"
                        defaultValue={status ?? ""}
                        name="status"
                      >
                        <option value="">
                          {t("filters.statusPlaceholder")}
                        </option>
                        <option value="ACTIVE">{t("filters.statusActive")}</option>
                        <option value="INACTIVE">
                          {t("filters.statusInactive")}
                        </option>
                        <option value="BANNED">{t("filters.statusBanned")}</option>
                      </select>
                    </label>
                  </div>
                  <div className="mt-3 flex items-center justify-between">
                    <p className="text-xs text-slate-500">
                      {t("graphql.total", { total: totalCount })}
                    </p>
                    <div className="flex gap-2">
                      <button className="btn btn-outline btn-sm" type="submit">
                        {t("filters.apply")}
                      </button>
                      <Link
                        className="btn btn-ghost btn-sm"
                        href={`/${locale}/users`}
                      >
                        {t("filters.reset")}
                      </Link>
                    </div>
                  </div>
                </form>
                <table className="min-w-full text-sm">
                  <thead className="bg-slate-50 text-left text-xs font-semibold uppercase text-slate-400">
                    <tr>
                      <th className="px-4 py-3">{t("graphql.columns.email")}</th>
                      <th className="px-4 py-3">{t("graphql.columns.name")}</th>
                      <th className="px-4 py-3">{t("graphql.columns.role")}</th>
                      <th className="px-4 py-3">{t("graphql.columns.status")}</th>
                      <th className="px-4 py-3">{t("graphql.columns.createdAt")}</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-100">
                    {graphqlResult.data?.edges.map((edge) => (
                      <tr key={edge.node.id}>
                        <td className="px-4 py-3 text-slate-900">
                          <Link
                            className="text-indigo-600 hover:text-indigo-500"
                            href={`/${locale}/users/${edge.node.id}`}
                          >
                            {edge.node.email}
                          </Link>
                        </td>
                        <td className="px-4 py-3 text-slate-500">
                          {edge.node.name ?? "—"}
                        </td>
                        <td className="px-4 py-3 text-slate-500">
                          {edge.node.role}
                        </td>
                        <td className="px-4 py-3">
                          <span className="badge badge-outline">
                            {edge.node.status}
                          </span>
                        </td>
                        <td className="px-4 py-3 text-slate-500">
                          {edge.node.createdAt}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                <div className="flex items-center justify-between border-t border-slate-100 px-4 py-3 text-xs text-slate-500">
                  <span>
                    {t("pagination.page", {
                      current: currentPage,
                      total: totalPages,
                    })}
                  </span>
                  <div className="flex gap-2">
                    <Link
                      className={`btn btn-outline btn-xs ${currentPage <= 1 ? "pointer-events-none opacity-50" : ""}`}
                      href={`/${locale}/users${buildQueryString({
                        page: String(currentPage - 1),
                      })}`}
                    >
                      {t("pagination.prev")}
                    </Link>
                    <Link
                      className={`btn btn-outline btn-xs ${currentPage >= totalPages ? "pointer-events-none opacity-50" : ""}`}
                      href={`/${locale}/users${buildQueryString({
                        page: String(currentPage + 1),
                      })}`}
                    >
                      {t("pagination.next")}
                    </Link>
                  </div>
                </div>
              </div>
            )}
          </section>
        </div>
      </section>
    </main>
  );
}
