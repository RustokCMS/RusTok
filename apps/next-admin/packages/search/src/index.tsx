'use client';

import React from 'react';

import { graphqlRequest as sharedGraphqlRequest } from '../../../src/shared/api/graphql';

type SearchAdminTab = 'overview' | 'playground' | 'analytics' | 'dictionaries';

export type SearchAdminPageProps = {
  token?: string | null;
  tenantSlug?: string | null;
  graphqlUrl?: string;
  initialTab?: SearchAdminTab;
  laggingLimit?: number;
};

type SearchAdminBootstrap = {
  availableSearchEngines: Array<{
    kind: string;
    label: string;
    providedBy: string;
    enabled: boolean;
    defaultEngine: boolean;
  }>;
  searchSettingsPreview: {
    tenantId: string | null;
    activeEngine: string;
    fallbackEngine: string;
    config: string;
    updatedAt: string;
  };
  searchDiagnostics: {
    tenantId: string;
    totalDocuments: number;
    publicDocuments: number;
    contentDocuments: number;
    productDocuments: number;
    staleDocuments: number;
    newestIndexedAt: string | null;
    oldestIndexedAt: string | null;
    maxLagSeconds: number;
    state: string;
  };
};

type SearchPreviewPayload = {
  total: number;
  tookMs: number;
  engine: string;
  items: Array<{
    id: string;
    entityType: string;
    sourceModule: string;
    title: string;
    snippet: string | null;
    score: number;
  }>;
  facets: Array<{
    name: string;
    buckets: Array<{ value: string; count: number }>;
  }>;
};

type LaggingSearchDocumentPayload = {
  documentKey: string;
  documentId: string;
  sourceModule: string;
  entityType: string;
  locale: string;
  status: string;
  title: string;
  updatedAt: string;
  indexedAt: string;
  lagSeconds: number;
};

const SEARCH_ADMIN_BOOTSTRAP_QUERY = `
  query SearchAdminBootstrap {
    availableSearchEngines { kind label providedBy enabled defaultEngine }
    searchSettingsPreview { tenantId activeEngine fallbackEngine config updatedAt }
    searchDiagnostics {
      tenantId totalDocuments publicDocuments contentDocuments productDocuments
      staleDocuments newestIndexedAt oldestIndexedAt maxLagSeconds state
    }
  }
`;

const SEARCH_PREVIEW_QUERY = `
  query SearchPreview($input: SearchPreviewInput!) {
    searchPreview(input: $input) {
      total tookMs engine
      items { id entityType sourceModule title snippet score locale payload }
      facets { name buckets { value count } }
    }
  }
`;

const SEARCH_LAGGING_DOCUMENTS_QUERY = `
  query SearchLaggingDocuments($limit: Int) {
    searchLaggingDocuments(limit: $limit) {
      documentKey documentId sourceModule entityType locale status isPublic title updatedAt indexedAt lagSeconds
    }
  }
`;

const TRIGGER_SEARCH_REBUILD_MUTATION = `
  mutation TriggerSearchRebuild($input: TriggerSearchRebuildInput!) {
    triggerSearchRebuild(input: $input) { success queued tenantId targetType targetId }
  }
`;

const tabs: Array<{ key: SearchAdminTab; label: string }> = [
  { key: 'overview', label: 'Overview' },
  { key: 'playground', label: 'Playground' },
  { key: 'analytics', label: 'Diagnostics' },
  { key: 'dictionaries', label: 'Dictionaries' }
];

function parseCsv(value: string): string[] {
  return value
    .split(',')
    .map((segment) => segment.trim())
    .filter(Boolean);
}

function optionalText(value: string): string | undefined {
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : undefined;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : 'Unexpected error';
}

async function graphqlRequest<TData>(
  query: string,
  variables: unknown,
  opts: {
    token?: string | null;
    tenantSlug?: string | null;
    graphqlUrl?: string;
  }
): Promise<TData> {
  return sharedGraphqlRequest<unknown, TData>(
    query,
    variables,
    opts.token,
    opts.tenantSlug,
    { graphqlUrl: opts.graphqlUrl }
  );
}

export function SearchAdminPage({
  token = null,
  tenantSlug = null,
  graphqlUrl,
  initialTab = 'overview',
  laggingLimit = 25
}: SearchAdminPageProps = {}): React.JSX.Element {
  const [activeTab, setActiveTab] = React.useState<SearchAdminTab>(initialTab);
  const [bootstrap, setBootstrap] = React.useState<SearchAdminBootstrap | null>(
    null
  );
  const [bootstrapError, setBootstrapError] = React.useState<string | null>(
    null
  );
  const [bootstrapLoading, setBootstrapLoading] = React.useState(false);
  const [laggingDocuments, setLaggingDocuments] = React.useState<
    LaggingSearchDocumentPayload[]
  >([]);
  const [laggingError, setLaggingError] = React.useState<string | null>(null);
  const [laggingLoading, setLaggingLoading] = React.useState(false);
  const [refreshNonce, setRefreshNonce] = React.useState(0);
  const [query, setQuery] = React.useState('');
  const [entityTypes, setEntityTypes] = React.useState('');
  const [sourceModules, setSourceModules] = React.useState('');
  const [statuses, setStatuses] = React.useState('');
  const [preview, setPreview] = React.useState<SearchPreviewPayload | null>(
    null
  );
  const [previewError, setPreviewError] = React.useState<string | null>(null);
  const [previewBusy, setPreviewBusy] = React.useState(false);
  const [rebuildScope, setRebuildScope] = React.useState('search');
  const [rebuildTargetId, setRebuildTargetId] = React.useState('');
  const [rebuildBusy, setRebuildBusy] = React.useState(false);
  const [rebuildFeedback, setRebuildFeedback] = React.useState<string | null>(
    null
  );

  React.useEffect(() => {
    let cancelled = false;
    if (!token || !tenantSlug) {
      setBootstrap(null);
      setBootstrapError(null);
      setBootstrapLoading(false);
      return () => {
        cancelled = true;
      };
    }

    setBootstrapLoading(true);
    setBootstrapError(null);

    void graphqlRequest<SearchAdminBootstrap>(
      SEARCH_ADMIN_BOOTSTRAP_QUERY,
      undefined,
      { token, tenantSlug, graphqlUrl }
    )
      .then((data) => {
        if (!cancelled) setBootstrap(data);
      })
      .catch((error: unknown) => {
        if (!cancelled) setBootstrapError(errorMessage(error));
      })
      .finally(() => {
        if (!cancelled) setBootstrapLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [token, tenantSlug, graphqlUrl, refreshNonce]);

  React.useEffect(() => {
    let cancelled = false;
    if (!token || !tenantSlug) {
      setLaggingDocuments([]);
      setLaggingError(null);
      setLaggingLoading(false);
      return () => {
        cancelled = true;
      };
    }

    setLaggingLoading(true);
    setLaggingError(null);

    void graphqlRequest<{
      searchLaggingDocuments: LaggingSearchDocumentPayload[];
    }>(
      SEARCH_LAGGING_DOCUMENTS_QUERY,
      { limit: laggingLimit },
      { token, tenantSlug, graphqlUrl }
    )
      .then((data) => {
        if (!cancelled) setLaggingDocuments(data.searchLaggingDocuments);
      })
      .catch((error: unknown) => {
        if (!cancelled) setLaggingError(errorMessage(error));
      })
      .finally(() => {
        if (!cancelled) setLaggingLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [token, tenantSlug, graphqlUrl, laggingLimit, refreshNonce]);

  async function runPreview(
    event: React.FormEvent<HTMLFormElement>
  ): Promise<void> {
    event.preventDefault();
    if (!token || !tenantSlug) {
      setPreviewError('Search preview requires token and tenant slug.');
      return;
    }

    setPreviewBusy(true);
    setPreviewError(null);

    try {
      const data = await graphqlRequest<{
        searchPreview: SearchPreviewPayload;
      }>(
        SEARCH_PREVIEW_QUERY,
        {
          input: {
            query,
            limit: 12,
            offset: 0,
            entityTypes: parseCsv(entityTypes).length
              ? parseCsv(entityTypes)
              : undefined,
            sourceModules: parseCsv(sourceModules).length
              ? parseCsv(sourceModules)
              : undefined,
            statuses: parseCsv(statuses).length ? parseCsv(statuses) : undefined
          }
        },
        { token, tenantSlug, graphqlUrl }
      );
      setPreview(data.searchPreview);
    } catch (error: unknown) {
      setPreviewError(errorMessage(error));
    } finally {
      setPreviewBusy(false);
    }
  }

  async function queueRebuild(): Promise<void> {
    if (!token || !tenantSlug) {
      setRebuildFeedback('Scoped rebuild requires token and tenant slug.');
      return;
    }

    setRebuildBusy(true);
    setRebuildFeedback(null);

    try {
      const data = await graphqlRequest<{
        triggerSearchRebuild: { targetType: string; targetId: string | null };
      }>(
        TRIGGER_SEARCH_REBUILD_MUTATION,
        {
          input: {
            targetType: rebuildScope,
            targetId: optionalText(rebuildTargetId)
          }
        },
        { token, tenantSlug, graphqlUrl }
      );
      const payload = data.triggerSearchRebuild;
      const suffix = payload.targetId ? ` for target ${payload.targetId}` : '';
      setRebuildFeedback(
        `Rebuild queued for ${payload.targetType} scope${suffix}.`
      );
      setRefreshNonce((value) => value + 1);
    } catch (error: unknown) {
      setRebuildFeedback(
        `Failed to queue search rebuild: ${errorMessage(error)}`
      );
    } finally {
      setRebuildBusy(false);
    }
  }

  return (
    <section className='space-y-6 rounded-[24px] border border-zinc-300 bg-white p-6'>
      <header className='flex flex-wrap items-start justify-between gap-4'>
        <div className='space-y-3'>
          <div className='inline-flex rounded-full border border-zinc-300 px-3 py-1 text-[12px] tracking-[0.14em] text-zinc-500 uppercase'>
            search
          </div>
          <div>
            <h1 className='text-[28px] font-semibold text-zinc-900'>
              Search Control Plane
            </h1>
            <p className='mt-3 max-w-3xl text-sm text-zinc-600'>
              Module-owned Next admin surface for diagnostics, scoped rebuilds,
              and PostgreSQL FTS inspection.
            </p>
          </div>
        </div>
        <div className='flex flex-wrap gap-2'>
          {tabs.map((tab) => (
            <button
              key={tab.key}
              type='button'
              onClick={() => React.startTransition(() => setActiveTab(tab.key))}
              className={
                tab.key === activeTab
                  ? 'rounded-xl bg-teal-700 px-4 py-2 text-sm font-medium text-white'
                  : 'rounded-xl border border-zinc-300 px-4 py-2 text-sm font-medium text-zinc-900'
              }
            >
              {tab.label}
            </button>
          ))}
        </div>
      </header>
      {!token || !tenantSlug ? (
        <EmptyPanel
          title='Search admin package is ready for host wiring'
          body='Pass token and tenantSlug from the Next admin host to enable diagnostics, rebuilds, and FTS preview.'
        />
      ) : bootstrapLoading ? (
        <LoadingPanel label='Loading search control plane...' />
      ) : bootstrapError ? (
        <ErrorPanel
          message={`Failed to load search bootstrap data: ${bootstrapError}`}
        />
      ) : !bootstrap ? (
        <EmptyPanel
          title='No bootstrap payload'
          body='The GraphQL endpoint returned no usable search bootstrap data.'
        />
      ) : activeTab === 'playground' ? (
        <PlaygroundPanel
          query={query}
          entityTypes={entityTypes}
          sourceModules={sourceModules}
          statuses={statuses}
          preview={preview}
          previewBusy={previewBusy}
          previewError={previewError}
          onQueryChange={setQuery}
          onEntityTypesChange={setEntityTypes}
          onSourceModulesChange={setSourceModules}
          onStatusesChange={setStatuses}
          onSubmit={(event) => void runPreview(event)}
        />
      ) : activeTab === 'analytics' ? (
        <AnalyticsPanel
          diagnostics={bootstrap.searchDiagnostics}
          laggingDocuments={laggingDocuments}
          laggingError={laggingError}
          laggingLoading={laggingLoading}
        />
      ) : activeTab === 'dictionaries' ? (
        <EmptyPanel
          title='Search Dictionaries'
          body='Dictionary editors stay in a later phase. Diagnostics, scoped rebuilds, and FTS preview are already live in both Leptos and Next admin.'
        />
      ) : (
        <OverviewPanel
          bootstrap={bootstrap}
          rebuildScope={rebuildScope}
          rebuildTargetId={rebuildTargetId}
          rebuildBusy={rebuildBusy}
          rebuildFeedback={rebuildFeedback}
          onScopeChange={setRebuildScope}
          onTargetIdChange={setRebuildTargetId}
          onQueueRebuild={() => void queueRebuild()}
        />
      )}
    </section>
  );
}

function OverviewPanel(props: {
  bootstrap: SearchAdminBootstrap;
  rebuildScope: string;
  rebuildTargetId: string;
  rebuildBusy: boolean;
  rebuildFeedback: string | null;
  onScopeChange: (value: string) => void;
  onTargetIdChange: (value: string) => void;
  onQueueRebuild: () => void;
}): React.JSX.Element {
  const diagnostics = props.bootstrap.searchDiagnostics;
  return (
    <div className='space-y-6'>
      <div className='grid gap-4 md:grid-cols-2 xl:grid-cols-4'>
        <InfoCard
          title='Active engine'
          value={props.bootstrap.searchSettingsPreview.activeEngine}
          detail='Effective search settings for the current tenant.'
        />
        <InfoCard
          title='Fallback engine'
          value={props.bootstrap.searchSettingsPreview.fallbackEngine}
          detail='Used when an external engine is configured but unavailable.'
        />
        <InfoCard
          title='Available engines'
          value={String(props.bootstrap.availableSearchEngines.length)}
          detail='Only installed connectors appear in runtime selection.'
        />
        <InfoCard
          title='Updated at'
          value={props.bootstrap.searchSettingsPreview.updatedAt}
          detail='Timestamp of the effective settings record.'
        />
      </div>
      <div className='grid gap-4 md:grid-cols-2 xl:grid-cols-5'>
        <DiagnosticsCard diagnostics={diagnostics} />
        <InfoCard
          title='Documents'
          value={String(diagnostics.totalDocuments)}
          detail='Total search documents in rustok-search storage.'
        />
        <InfoCard
          title='Public docs'
          value={String(diagnostics.publicDocuments)}
          detail='Published documents visible to storefront search.'
        />
        <InfoCard
          title='Stale docs'
          value={String(diagnostics.staleDocuments)}
          detail='Documents where indexed_at lags behind source updated_at.'
        />
        <InfoCard
          title='Max lag'
          value={`${diagnostics.maxLagSeconds}s`}
          detail='Worst-case lag between source update and search projection.'
        />
      </div>
      <article className='rounded-3xl border border-zinc-200 bg-white p-6'>
        <h2 className='text-lg font-semibold text-zinc-900'>Scoped Rebuild</h2>
        <p className='mt-2 text-sm text-zinc-600'>
          Queue tenant-wide or scoped rebuilds. <code>content</code> and{' '}
          <code>product</code> rebuild the whole slice when target ID is empty,
          or a single entity when target ID is provided.
        </p>
        <div className='mt-5 grid gap-4 md:grid-cols-[14rem_minmax(0,1fr)_auto]'>
          <label className='space-y-2'>
            <span className='text-sm font-medium text-zinc-900'>Scope</span>
            <select
              value={props.rebuildScope}
              onChange={(event) => props.onScopeChange(event.target.value)}
              className='w-full rounded-xl border border-zinc-300 px-3 py-2 text-sm'
            >
              <option value='search'>search</option>
              <option value='content'>content</option>
              <option value='product'>product</option>
            </select>
          </label>
          <label className='space-y-2'>
            <span className='text-sm font-medium text-zinc-900'>
              Target ID (optional)
            </span>
            <input
              value={props.rebuildTargetId}
              onChange={(event) => props.onTargetIdChange(event.target.value)}
              placeholder='UUID for single node/product rebuild'
              className='w-full rounded-xl border border-zinc-300 px-3 py-2 text-sm'
            />
          </label>
          <div className='flex items-end'>
            <button
              type='button'
              onClick={props.onQueueRebuild}
              disabled={props.rebuildBusy}
              className='rounded-xl bg-teal-700 px-4 py-2 text-sm font-medium text-white disabled:opacity-50'
            >
              {props.rebuildBusy ? 'Queueing...' : 'Queue Rebuild'}
            </button>
          </div>
        </div>
        {props.rebuildFeedback ? (
          <div className='mt-4 rounded-2xl border border-zinc-200 bg-zinc-50 px-4 py-3 text-sm text-zinc-600'>
            {props.rebuildFeedback}
          </div>
        ) : null}
      </article>
    </div>
  );
}

function PlaygroundPanel(props: {
  query: string;
  entityTypes: string;
  sourceModules: string;
  statuses: string;
  preview: SearchPreviewPayload | null;
  previewBusy: boolean;
  previewError: string | null;
  onQueryChange: (value: string) => void;
  onEntityTypesChange: (value: string) => void;
  onSourceModulesChange: (value: string) => void;
  onStatusesChange: (value: string) => void;
  onSubmit: (event: React.FormEvent<HTMLFormElement>) => void;
}): React.JSX.Element {
  return (
    <div className='grid gap-6 xl:grid-cols-[minmax(0,22rem)_minmax(0,1fr)]'>
      <form
        className='space-y-4 rounded-3xl border border-zinc-200 bg-white p-6'
        onSubmit={props.onSubmit}
      >
        <h2 className='text-lg font-semibold text-zinc-900'>Search Preview</h2>
        <p className='text-sm text-zinc-600'>
          Runs the current PostgreSQL FTS preview path over rustok-search
          documents.
        </p>
        <Field label='Query'>
          <input
            value={props.query}
            onChange={(event) => props.onQueryChange(event.target.value)}
            className='w-full rounded-xl border border-zinc-300 px-3 py-2 text-sm'
          />
        </Field>
        <Field label='Entity types (CSV)'>
          <input
            value={props.entityTypes}
            onChange={(event) => props.onEntityTypesChange(event.target.value)}
            className='w-full rounded-xl border border-zinc-300 px-3 py-2 text-sm'
          />
        </Field>
        <Field label='Source modules (CSV)'>
          <input
            value={props.sourceModules}
            onChange={(event) =>
              props.onSourceModulesChange(event.target.value)
            }
            className='w-full rounded-xl border border-zinc-300 px-3 py-2 text-sm'
          />
        </Field>
        <Field label='Statuses (CSV)'>
          <input
            value={props.statuses}
            onChange={(event) => props.onStatusesChange(event.target.value)}
            className='w-full rounded-xl border border-zinc-300 px-3 py-2 text-sm'
          />
        </Field>
        {props.previewError ? (
          <ErrorPanel
            message={`Failed to run search preview: ${props.previewError}`}
          />
        ) : null}
        <button
          type='submit'
          disabled={props.previewBusy}
          className='w-full rounded-xl bg-teal-700 px-4 py-2 text-sm font-medium text-white disabled:opacity-50'
        >
          {props.previewBusy ? 'Running...' : 'Run FTS Preview'}
        </button>
      </form>
      {props.preview ? (
        <PreviewPanel payload={props.preview} />
      ) : (
        <EmptyPanel
          title='No preview results yet'
          body='Run a preview query to inspect FTS results, facets, and effective engine output.'
        />
      )}
    </div>
  );
}

function AnalyticsPanel(props: {
  diagnostics: SearchAdminBootstrap['searchDiagnostics'];
  laggingDocuments: LaggingSearchDocumentPayload[];
  laggingError: string | null;
  laggingLoading: boolean;
}): React.JSX.Element {
  return (
    <div className='space-y-6'>
      <div className='grid gap-4 md:grid-cols-2 xl:grid-cols-5'>
        <DiagnosticsCard diagnostics={props.diagnostics} />
        <InfoCard
          title='Lagging docs'
          value={String(props.diagnostics.staleDocuments)}
          detail='Documents where projection timestamps are behind source updates.'
        />
        <InfoCard
          title='Max lag'
          value={`${props.diagnostics.maxLagSeconds}s`}
          detail='Largest observed lag in seconds.'
        />
        <InfoCard
          title='Newest indexed'
          value={props.diagnostics.newestIndexedAt ?? 'not indexed yet'}
          detail='Most recent index write in rustok-search storage.'
        />
        <InfoCard
          title='Oldest indexed'
          value={props.diagnostics.oldestIndexedAt ?? 'not indexed yet'}
          detail='Oldest surviving indexed document timestamp.'
        />
      </div>
      <article className='rounded-3xl border border-zinc-200 bg-white p-6'>
        <h2 className='text-lg font-semibold text-zinc-900'>
          Lagging Documents
        </h2>
        <p className='mt-2 text-sm text-zinc-600'>
          Raw diagnostics for the most stale documents in search storage.
        </p>
        <div className='mt-5'>
          {props.laggingLoading ? (
            <LoadingPanel label='Loading lagging documents...' />
          ) : props.laggingError ? (
            <ErrorPanel
              message={`Failed to load lagging search diagnostics: ${props.laggingError}`}
            />
          ) : (
            <LaggingTable rows={props.laggingDocuments} />
          )}
        </div>
      </article>
    </div>
  );
}

function PreviewPanel({
  payload
}: {
  payload: SearchPreviewPayload;
}): React.JSX.Element {
  return (
    <article className='rounded-3xl border border-zinc-200 bg-white p-6'>
      <h2 className='text-lg font-semibold text-zinc-900'>Preview Results</h2>
      <p className='mt-2 text-sm text-zinc-600'>
        {payload.total} results in {payload.tookMs} ms via {payload.engine}
      </p>
      <div className='mt-5 grid gap-4 lg:grid-cols-3'>
        {payload.facets.map((facet) => (
          <article
            key={facet.name}
            className='rounded-2xl border border-zinc-200 bg-zinc-50 p-4'
          >
            <div className='text-sm font-semibold text-zinc-900 capitalize'>
              {facet.name.replaceAll('_', ' ')}
            </div>
            <div className='mt-3 flex flex-wrap gap-2'>
              {facet.buckets.map((bucket) => (
                <span
                  key={`${facet.name}-${bucket.value}`}
                  className='rounded-full border border-zinc-300 px-3 py-1 text-xs text-zinc-600'
                >
                  {bucket.value} ({bucket.count})
                </span>
              ))}
            </div>
          </article>
        ))}
      </div>
      <div className='mt-6 space-y-3'>
        {payload.items.map((item) => (
          <article
            key={item.id}
            className='rounded-2xl border border-zinc-200 bg-zinc-50 p-4'
          >
            <div className='flex flex-wrap gap-2 text-[11px] tracking-[0.16em] text-zinc-500 uppercase'>
              <span>{item.entityType}</span>
              <span>|</span>
              <span>{item.sourceModule}</span>
              <span>|</span>
              <span>score {item.score.toFixed(3)}</span>
            </div>
            <h3 className='mt-2 text-base font-semibold text-zinc-900'>
              {item.title}
            </h3>
            <p className='mt-2 text-sm text-zinc-600'>
              {item.snippet ?? 'No snippet returned.'}
            </p>
          </article>
        ))}
      </div>
    </article>
  );
}

function LaggingTable({
  rows
}: {
  rows: LaggingSearchDocumentPayload[];
}): React.JSX.Element {
  if (!rows.length)
    return (
      <EmptyPanel
        title='Projection is caught up'
        body='No lagging documents detected. Search projection is currently up to date.'
      />
    );
  return (
    <div className='overflow-hidden rounded-2xl border border-zinc-200'>
      <table className='w-full text-sm'>
        <thead className='border-b border-zinc-200 bg-zinc-50'>
          <tr>
            <Th>Title</Th>
            <Th>Type</Th>
            <Th>Locale</Th>
            <Th>Lag</Th>
            <Th>Indexed</Th>
            <Th>Updated</Th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={row.documentKey} className='border-t border-zinc-100'>
              <Td>
                <div className='font-medium text-zinc-900'>{row.title}</div>
                <div className='mt-1 text-xs text-zinc-500'>
                  {row.documentKey}
                </div>
              </Td>
              <Td>{`${row.sourceModule}/${row.entityType} (${row.status})`}</Td>
              <Td>{row.locale}</Td>
              <Td>
                <span className='rounded-full border border-amber-200 bg-amber-50 px-2.5 py-0.5 text-xs font-semibold text-amber-700'>
                  {row.lagSeconds}s
                </span>
              </Td>
              <Td>{row.indexedAt}</Td>
              <Td>{row.updatedAt}</Td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function DiagnosticsCard({
  diagnostics
}: {
  diagnostics: SearchAdminBootstrap['searchDiagnostics'];
}): React.JSX.Element {
  const badgeClass =
    diagnostics.state === 'healthy'
      ? 'border-emerald-200 bg-emerald-50 text-emerald-700'
      : diagnostics.state === 'lagging'
        ? 'border-amber-200 bg-amber-50 text-amber-700'
        : 'border-slate-200 bg-slate-50 text-slate-700';
  return (
    <article className='rounded-3xl border border-zinc-200 bg-zinc-50 p-5'>
      <div className='text-xs tracking-[0.2em] text-zinc-500 uppercase'>
        Index state
      </div>
      <div className='mt-3'>
        <span
          className={`rounded-full border px-3 py-1 text-xs font-semibold ${badgeClass}`}
        >
          {diagnostics.state}
        </span>
      </div>
      <p className='mt-3 text-sm text-zinc-600'>
        Newest indexed: {diagnostics.newestIndexedAt ?? 'not indexed yet'}
      </p>
    </article>
  );
}

function InfoCard({
  title,
  value,
  detail
}: {
  title: string;
  value: string;
  detail: string;
}): React.JSX.Element {
  return (
    <article className='rounded-3xl border border-zinc-200 bg-zinc-50 p-5'>
      <div className='text-xs tracking-[0.2em] text-zinc-500 uppercase'>
        {title}
      </div>
      <div className='mt-2 text-lg font-semibold text-zinc-900'>{value}</div>
      <p className='mt-2 text-sm text-zinc-600'>{detail}</p>
    </article>
  );
}

function Field({
  label,
  children
}: {
  label: string;
  children: React.ReactNode;
}): React.JSX.Element {
  return (
    <label className='block space-y-2'>
      <span className='text-sm font-medium text-zinc-900'>{label}</span>
      {children}
    </label>
  );
}

function Th({ children }: { children: React.ReactNode }): React.JSX.Element {
  return (
    <th className='px-4 py-3 text-left text-xs font-semibold tracking-[0.14em] text-zinc-500 uppercase'>
      {children}
    </th>
  );
}

function Td({ children }: { children: React.ReactNode }): React.JSX.Element {
  return (
    <td className='px-4 py-3 align-top text-xs text-zinc-600'>{children}</td>
  );
}

function LoadingPanel({ label }: { label: string }): React.JSX.Element {
  return (
    <div className='rounded-3xl border border-dashed border-zinc-300 bg-zinc-50 p-10 text-center text-sm text-zinc-500'>
      {label}
    </div>
  );
}

function ErrorPanel({ message }: { message: string }): React.JSX.Element {
  return (
    <div className='rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700'>
      {message}
    </div>
  );
}

function EmptyPanel({
  title,
  body
}: {
  title: string;
  body: string;
}): React.JSX.Element {
  return (
    <article className='rounded-3xl border border-dashed border-zinc-300 bg-zinc-50 p-10 text-center'>
      <h2 className='text-xl font-semibold text-zinc-900'>{title}</h2>
      <p className='mx-auto mt-3 max-w-3xl text-sm text-zinc-600'>{body}</p>
    </article>
  );
}

export default SearchAdminPage;
