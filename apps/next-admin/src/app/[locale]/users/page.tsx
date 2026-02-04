import { getLocale, getTranslations } from "next-intl/server";

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

type GraphqlUsersResponse = {
  data?: {
    users: {
      edges: Array<{ node: GraphqlUser }>;
      pageInfo: { totalCount: number };
    };
  };
  errors?: Array<{ message: string }>;
};

const graphqlQuery = `query Users($pagination: PaginationInput) {
  users(pagination: $pagination) {
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
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };

  if (apiToken) {
    headers.Authorization = `Bearer ${apiToken}`;
  }

  if (tenantSlug) {
    headers["X-Tenant-Slug"] = tenantSlug;
  }

  return headers;
};

async function fetchRestUser() {
  try {
    const response = await fetch(`${apiBaseUrl}/api/auth/me`, {
      headers: buildHeaders(),
    });

    if (!response.ok) {
      return { error: `REST ${response.status}` };
    }

    const data = (await response.json()) as RestUser;
    return { data };
  } catch (error) {
    return { error: "REST network error" };
  }
}

async function fetchGraphqlUsers() {
  try {
    const response = await fetch(`${apiBaseUrl}/api/graphql`, {
      method: "POST",
      headers: buildHeaders(),
      body: JSON.stringify({
        query: graphqlQuery,
        variables: { pagination: { offset: 0, limit: 8 } },
      }),
    });

    if (!response.ok) {
      return { error: `GraphQL ${response.status}` };
    }

    const payload = (await response.json()) as GraphqlUsersResponse;
    if (payload.errors?.length) {
      return { error: payload.errors[0]?.message ?? "GraphQL error" };
    }

    return { data: payload.data?.users };
  } catch (error) {
    return { error: "GraphQL network error" };
  }
}

export default async function UsersPage() {
  const t = await getTranslations("Users");
  const locale = await getLocale();
  const [restResult, graphqlResult] = await Promise.all([
    fetchRestUser(),
    fetchGraphqlUsers(),
  ]);

  return (
    <main className="min-h-screen bg-slate-50">
      <section className="mx-auto max-w-6xl px-6 py-12">
        <header className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <p className="text-sm font-medium text-indigo-600">{t("eyebrow")}</p>
            <h1 className="mt-2 text-3xl font-semibold text-slate-900">
              {t("title")}
            </h1>
            <p className="mt-2 max-w-2xl text-sm text-slate-500">
              {t("subtitle")}
            </p>
          </div>
          <a
            className="btn btn-outline"
            href={`/${locale}`}
          >
            {t("back")}
          </a>
        </header>

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
                {restResult.error}
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
                {graphqlResult.error}
              </div>
            ) : (
              <div className="mt-4 overflow-x-auto rounded-xl border border-slate-200">
                <table className="min-w-full text-sm">
                  <thead className="bg-slate-50 text-left text-xs font-semibold uppercase text-slate-400">
                    <tr>
                      <th className="px-4 py-3">{t("graphql.columns.email")}</th>
                      <th className="px-4 py-3">{t("graphql.columns.name")}</th>
                      <th className="px-4 py-3">{t("graphql.columns.role")}</th>
                      <th className="px-4 py-3">{t("graphql.columns.status")}</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-100">
                    {graphqlResult.data?.edges.map((edge) => (
                      <tr key={edge.node.id}>
                        <td className="px-4 py-3 text-slate-900">
                          {edge.node.email}
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
                      </tr>
                    ))}
                  </tbody>
                </table>
                <div className="border-t border-slate-100 px-4 py-3 text-xs text-slate-500">
                  {t("graphql.total", {
                    total: graphqlResult.data?.pageInfo.totalCount ?? 0,
                  })}
                </div>
              </div>
            )}
          </section>
        </div>
      </section>
    </main>
  );
}
