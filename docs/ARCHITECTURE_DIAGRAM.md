# RusToK Architecture Diagrams

> Visual representation of the RusToK architecture

---

## System Architecture Overview

```mermaid
graph TB
    subgraph "Client Layer"
        Admin[Admin UI<br/>Leptos CSR]
        Store[Storefront<br/>Leptos SSR]
        Next[Next.js App]
    end
    
    subgraph "API Layer"
        GQL[GraphQL<br/>async-graphql]
        REST[REST APIs<br/>Axum + utoipa]
        Health[Health Checks]
    end
    
    subgraph "Service Layer"
        TM[Tenant Middleware]
        Auth[Auth Service]
        RBAC[RBAC]
    end
    
    subgraph "Domain Modules"
        Content[Content Module<br/>nodes, bodies]
        Commerce[Commerce Module<br/>products, orders]
        Blog[Blog Module<br/>wrapper]
        Forum[Forum Module<br/>wrapper]
        Pages[Pages Module<br/>wrapper]
    end
    
    subgraph "Infrastructure"
        Core[rustok-core<br/>events, registry]
        Outbox[rustok-outbox<br/>reliable events]
        Index[rustok-index<br/>CQRS read model]
        Iggy[rustok-iggy<br/>streaming]
        Telem[rustok-telemetry]
    end
    
    subgraph "Data Layer"
        PG[(PostgreSQL<br/>Write Model)]
        Redis[(Redis<br/>Cache)]
        Search[(Search<br/>FTS/Tantivy)]
    end
    
    Admin --> GQL
    Store --> GQL
    Next --> GQL
    Admin --> REST
    
    GQL --> TM
    REST --> TM
    
    TM --> Auth
    TM --> RBAC
    
    Auth --> Content
    Auth --> Commerce
    
    Content --> Core
    Commerce --> Core
    Blog --> Content
    Forum --> Content
    Pages --> Content
    
    Core --> Outbox
    Outbox --> PG
    Core --> Index
    
    Content --> PG
    Commerce --> PG
    Index --> Search
    
    TM --> Redis
    
    style Admin fill:#e1f5ff
    style Store fill:#e1f5ff
    style GQL fill:#fff4e6
    style REST fill:#fff4e6
    style Content fill:#f3e5f5
    style Commerce fill:#f3e5f5
    style Core fill:#e8f5e9
    style PG fill:#ffebee
```

---

## Event Flow Architecture

```mermaid
sequenceDiagram
    participant Client
    participant API
    participant Service
    participant DB
    participant EventBus
    participant Indexer
    participant ReadModel
    
    Client->>API: Create Product Request
    API->>Service: create_product(input)
    
    Service->>DB: BEGIN TRANSACTION
    Service->>DB: INSERT product
    Service->>DB: INSERT variants
    
    Service->>EventBus: publish_in_tx(ProductCreated)
    EventBus->>DB: INSERT sys_events
    
    Service->>DB: COMMIT
    
    Note over DB,EventBus: Transactional!<br/>Events saved atomically
    
    EventBus-->>Indexer: ProductCreated event
    Indexer->>ReadModel: UPDATE index_products
    
    API-->>Client: ProductResponse
    
    Client->>API: Search Products
    API->>ReadModel: SELECT from index_products
    ReadModel-->>API: Results
    API-->>Client: SearchResponse
```

---

## Module Dependency Graph

```mermaid
graph TD
    subgraph "Core Infrastructure"
        Core[rustok-core]
        Outbox[rustok-outbox]
        Telem[rustok-telemetry]
    end
    
    subgraph "Domain Layer"
        Content[rustok-content]
        Commerce[rustok-commerce]
        Tenant[rustok-tenant]
        RBAC[rustok-rbac]
    end
    
    subgraph "Wrapper Layer"
        Blog[rustok-blog]
        Forum[rustok-forum]
        Pages[rustok-pages]
    end
    
    subgraph "Infrastructure Layer"
        Index[rustok-index]
        Iggy[rustok-iggy]
    end
    
    Blog --> Content
    Forum --> Content
    Pages --> Content
    
    Content --> Core
    Content --> Outbox
    Commerce --> Core
    Commerce --> Outbox
    Tenant --> Core
    RBAC --> Core
    
    Index --> Core
    Iggy --> Core
    
    Outbox --> Core
    
    style Core fill:#4caf50,color:#fff
    style Content fill:#2196f3,color:#fff
    style Commerce fill:#2196f3,color:#fff
    style Blog fill:#9c27b0,color:#fff
    style Forum fill:#9c27b0,color:#fff
    
    classDef critical fill:#f44336,color:#fff
    classDef important fill:#ff9800,color:#fff
```

---

## CQRS Pattern Implementation

```mermaid
graph LR
    subgraph "Write Side"
        API[API Request]
        Service[Domain Service]
        WriteDB[(Normalized Tables)]
    end
    
    subgraph "Event Bus"
        Events[Domain Events]
        Outbox[Outbox Pattern]
    end
    
    subgraph "Read Side"
        Indexer[Event Handlers]
        ReadDB[(Denormalized Tables)]
        Search[(Search Index)]
    end
    
    API --> Service
    Service --> WriteDB
    Service --> Events
    Events --> Outbox
    Outbox --> WriteDB
    
    Events --> Indexer
    Indexer --> ReadDB
    Indexer --> Search
    
    Query[Query Request] --> ReadDB
    Query --> Search
    
    style WriteDB fill:#ffcdd2
    style ReadDB fill:#c8e6c9
    style Events fill:#fff9c4
```

---

## Tenant Resolution Flow

```mermaid
flowchart TD
    Start[HTTP Request] --> Extract{Extract Identifier}
    
    Extract -->|subdomain| Host[Extract from Host]
    Extract -->|header| Header[Extract from Header]
    Extract -->|path| Path[Extract from Path]
    
    Host --> Validate{Validate & Sanitize}
    Header --> Validate
    Path --> Validate
    
    Validate -->|invalid| Reject[Return 400]
    Validate -->|valid| Cache{Check Cache}
    
    Cache -->|hit| Return[Return TenantContext]
    Cache -->|miss| DB[(Load from DB)]
    
    DB --> Found{Tenant Found?}
    Found -->|yes| Store[Store in Cache]
    Found -->|no| NegCache[Store in Negative Cache]
    
    Store --> Return
    NegCache --> Reject2[Return 404]
    
    style Validate fill:#fff9c4
    style Cache fill:#c8e6c9
    style DB fill:#ffcdd2
```

---

## Security Architecture

```mermaid
graph TB
    subgraph "Request Layer"
        Req[HTTP Request]
    end
    
    subgraph "Security Middleware"
        TenantMW[Tenant Resolution<br/>+ Validation]
        AuthMW[JWT Verification]
        RBACMW[Permission Check]
    end
    
    subgraph "Service Layer"
        Service[Domain Service]
        Validation[Input Validation]
    end
    
    subgraph "Data Layer"
        TenantFilter[Tenant Isolation Filter]
        DB[(Database)]
    end
    
    Req --> TenantMW
    TenantMW --> AuthMW
    AuthMW --> RBACMW
    RBACMW --> Service
    
    Service --> Validation
    Validation --> TenantFilter
    TenantFilter --> DB
    
    style TenantMW fill:#fff9c4
    style AuthMW fill:#fff9c4
    style RBACMW fill:#fff9c4
    style TenantFilter fill:#ffcdd2
```

---

## Event Transport Levels

```mermaid
graph LR
    subgraph "Level 0: Development"
        L0[In-Memory<br/>tokio::mpsc]
    end
    
    subgraph "Level 1: Production"
        L1[Outbox Pattern<br/>PostgreSQL]
    end
    
    subgraph "Level 2: High Load"
        L2[Streaming<br/>Iggy]
    end
    
    L0 -->|"Production Deploy"| L1
    L1 -->|"Scale Up"| L2
    L2 -->|"Fallback"| L1
    
    style L0 fill:#e3f2fd
    style L1 fill:#c8e6c9
    style L2 fill:#fff9c4
```

---

## Health Check Architecture

```mermaid
graph TB
    Client[Health Check Request<br/>/health/ready]
    
    Client --> Aggregator[Health Aggregator]
    
    Aggregator --> DB{Database<br/>Connection}
    Aggregator --> Cache{Redis<br/>Connection}
    Aggregator --> Modules{Module<br/>Health}
    
    DB --> DBResult[Critical]
    Cache --> CacheResult[Non-Critical]
    Modules --> ModResult[Variable]
    
    DBResult --> Evaluate{Evaluate}
    CacheResult --> Evaluate
    ModResult --> Evaluate
    
    Evaluate -->|All Critical OK| Healthy[200 OK]
    Evaluate -->|Critical Failed| Unhealthy[503 Unavailable]
    Evaluate -->|Non-Critical Failed| Degraded[200 OK<br/>degraded: true]
    
    style DB fill:#ffcdd2
    style Cache fill:#fff9c4
    style Modules fill:#c8e6c9
```

---

## Module Registry & Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Registered: register()
    
    Registered --> Initialized: on_enable()
    Initialized --> Healthy: health() == OK
    Initialized --> Degraded: health() == Degraded
    Initialized --> Failed: health() == Unhealthy
    
    Healthy --> Degraded: Dependency Failed
    Degraded --> Healthy: Dependency Recovered
    
    Healthy --> Disabled: on_disable()
    Degraded --> Disabled: on_disable()
    Failed --> Disabled: on_disable()
    
    Disabled --> [*]
    
    note right of Healthy
        Module is fully operational
        All features available
    end note
    
    note right of Degraded
        Module is operational
        Some features disabled
        Fallback mode active
    end note
    
    note right of Failed
        Module is not operational
        Critical dependency failed
        Requests rejected
    end note
```

---

## Backpressure & Circuit Breaker

```mermaid
graph TB
    subgraph "Event Dispatcher"
        Queue[Event Queue]
        BP[Backpressure<br/>Controller]
        Metrics[Queue Metrics]
    end
    
    subgraph "States"
        Normal[Normal<br/>&lt;50% capacity]
        Warning[Warning<br/>50-80% capacity]
        Critical[Critical<br/>&gt;80% capacity]
    end
    
    subgraph "Circuit Breaker"
        Closed[Closed<br/>All requests pass]
        Open[Open<br/>Fail fast]
        HalfOpen[Half-Open<br/>Test recovery]
    end
    
    Queue --> Metrics
    Metrics --> BP
    
    BP -->|depth check| Normal
    BP -->|depth check| Warning
    BP -->|depth check| Critical
    
    Critical -->|reject| Reject[Reject Event]
    
    Closed -->|failures| Open
    Open -->|timeout| HalfOpen
    HalfOpen -->|success| Closed
    HalfOpen -->|failure| Open
    
    style Normal fill:#c8e6c9
    style Warning fill:#fff9c4
    style Critical fill:#ffcdd2
```

---

## Future: Event Sourcing Pattern

```mermaid
graph TB
    subgraph "Command Side"
        Cmd[Command]
        Agg[Aggregate]
        Events[Event Stream]
    end
    
    subgraph "Event Store"
        ES[(Event Store<br/>Append-Only)]
        Snap[(Snapshots)]
    end
    
    subgraph "Read Side"
        Proj[Projections]
        RM[(Read Models)]
    end
    
    subgraph "Query Side"
        Query[Query]
        View[Materialized View]
    end
    
    Cmd --> Agg
    Agg --> Events
    Events --> ES
    
    ES --> Snap
    ES --> Proj
    
    Proj --> RM
    RM --> View
    
    Query --> View
    
    Rebuild[Rebuild from History] --> ES
    Rebuild --> Agg
    
    style ES fill:#ffcdd2
    style RM fill:#c8e6c9
    style Events fill:#fff9c4
```

---

## Deployment Architecture

```mermaid
graph TB
    subgraph "Load Balancer"
        LB[Nginx/ALB]
    end
    
    subgraph "Application Tier"
        API1[API Pod 1]
        API2[API Pod 2]
        API3[API Pod 3]
    end
    
    subgraph "Worker Tier"
        Relay[Outbox Relay Worker]
        Indexer[Index Worker]
    end
    
    subgraph "Data Tier"
        Master[(PostgreSQL<br/>Primary)]
        Replica1[(PostgreSQL<br/>Replica 1)]
        Replica2[(PostgreSQL<br/>Replica 2)]
    end
    
    subgraph "Cache Tier"
        Redis1[(Redis Primary)]
        Redis2[(Redis Replica)]
    end
    
    subgraph "Search Tier"
        Search[(Tantivy/Meilisearch)]
    end
    
    LB --> API1
    LB --> API2
    LB --> API3
    
    API1 --> Master
    API2 --> Master
    API3 --> Master
    
    API1 --> Redis1
    API2 --> Redis1
    API3 --> Redis1
    
    Master --> Replica1
    Master --> Replica2
    
    Relay --> Master
    Indexer --> Replica1
    Indexer --> Search
    
    Redis1 --> Redis2
    
    style Master fill:#ffcdd2
    style Redis1 fill:#fff9c4
    style Search fill:#c8e6c9
```

---

*These diagrams represent the current and planned architecture of RusToK.*
