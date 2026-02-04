import Link from "next/link";
import { getLocale, getTranslations } from "next-intl/server";

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
      return { error: `GraphQL ${response.status}` };
    }

    const payload = (await response.json()) as GraphqlUserResponse;
    if (payload.errors?.length) {
      return { error: payload.errors[0]?.message ?? "GraphQL error" };
    }

    return { data: payload.data?.user ?? null };
  } catch (error) {
    return { error: "GraphQL network error" };
  }
}

type UsersDetailPageProps = {
  params: { id: string };
};

export default async function UsersDetailPage({ params }: UsersDetailPageProps) {
  const t = await getTranslations("Users");
  const locale = await getLocale();
  const result = await fetchGraphqlUser(params.id);

  return (
    <main className="min-h-screen bg-slate-50">
      <section className="mx-auto max-w-5xl px-6 py-12">
        <header className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <p className="text-sm font-medium text-indigo-600">
              {t("eyebrow")}
            </p>
            <h1 className="mt-2 text-3xl font-semibold text-slate-900">
              {t("detail.title")}
            </h1>
            <p className="mt-2 max-w-2xl text-sm text-slate-500">
              {t("detail.subtitle")}
            </p>
          </div>
          <Link className="btn btn-outline" href={`/${locale}/users`}>
            {t("detail.back")}
          </Link>
        </header>

        <section className="mt-8 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
          <h2 className="text-lg font-semibold text-slate-900">
            {t("detail.section")}
          </h2>
          {result.error ? (
            <div className="mt-4 rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-600">
              {result.error}
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
