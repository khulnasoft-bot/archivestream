# ArchiveStream Project Structure

```
ArchiveStream/
│
├── 📚 Documentation
│   ├── README.md                          # Project overview & quickstart
│   ├── ROADMAP.md                         # Future vision (Phases 6-10)
│   ├── CONTRIBUTING.md                    # Contribution guidelines
│   ├── docs/
│   │   ├── IMPLEMENTATION_SUMMARY.md      # Complete technical overview
│   │   ├── API_V1.md                      # REST API reference
│   │   ├── SCALING.md                     # Horizontal scaling guide
│   │   ├── SCALING_MULTI_REGION.md        # Multi-region architecture
│   │   ├── PHASE_5C_STATUS.md             # Latest development status
│   │   └── PRODUCTION_CHECKLIST.md        # Deployment guide
│
├── 🦀 Rust Backend (Crates)
│   ├── crates/
│   │   ├── common/                        # Shared utilities
│   │   │   ├── src/
│   │   │   │   ├── lib.rs                 # Module exports
│   │   │   │   ├── warc.rs                # WARC format handling
│   │   │   │   ├── replay.rs              # Replay URL parsing
│   │   │   │   └── extractor.rs           # HTML text extraction
│   │   │   └── Cargo.toml
│   │   │
│   │   ├── crawler/                       # Distributed crawler
│   │   │   ├── src/
│   │   │   │   ├── lib.rs                 # Main crawler logic
│   │   │   │   ├── fetcher.rs             # HTTP fetching
│   │   │   │   ├── parser.rs              # Link extraction
│   │   │   │   ├── robots.rs              # robots.txt support
│   │   │   │   ├── warc.rs                # WARC writing
│   │   │   │   ├── dedup.rs               # Deduplication service
│   │   │   │   ├── frontier.rs            # URL frontier (DB-backed)
│   │   │   │   ├── region.rs              # Multi-region routing
│   │   │   │   └── rate_limit.rs          # Global rate limiting
│   │   │   ├── main.rs                    # Crawler binary
│   │   │   └── Cargo.toml
│   │   │
│   │   ├── indexer/                       # Search indexing
│   │   │   ├── src/
│   │   │   │   ├── main.rs                # Indexer binary
│   │   │   │   └── opensearch_client.rs   # OpenSearch integration
│   │   │   └── Cargo.toml
│   │   │
│   │   ├── semantic/                      # Semantic classification
│   │   │   ├── src/
│   │   │   │   ├── lib.rs                 # Module exports
│   │   │   │   ├── classifier.rs          # Change categorization
│   │   │   │   └── alert.rs               # Alert system (future)
│   │   │   └── Cargo.toml
│   │   │
│   │   └── archive-api/                   # REST API server
│   │       ├── src/
│   │       │   ├── main.rs                # Axum router & handlers
│   │       │   ├── replay.rs              # Replay logic
│   │       │   ├── search.rs              # Search service
│   │       │   ├── diff.rs                # Diff computation
│   │       │   └── semantic.rs            # Semantic endpoint
│   │       └── Cargo.toml
│   │
│   └── Cargo.toml                         # Workspace configuration
│
├── ⚛️ Next.js Frontend
│   └── apps/web-ui/
│       ├── src/
│       │   ├── app/
│       │   │   ├── page.tsx               # Home page with search
│       │   │   ├── dashboard/
│       │   │   │   └── page.tsx           # Observability dashboard
│       │   │   └── web/[timestamp]/[...url]/
│       │   │       └── page.tsx           # Replay page
│       │   │
│       │   └── components/
│       │       ├── TimeScrubber.tsx       # Timeline navigation
│       │       ├── DiffViewer.tsx         # Visual diff display
│       │       └── dashboard/
│       │           ├── FrontierHeatmap.tsx # Frontier visualization
│       │           └── CrawlOutcomes.tsx   # Success/failure metrics
│       │
│       ├── package.json
│       └── tailwind.config.js
│
├── 🗄️ Database & Infrastructure
│   ├── infra/
│   │   ├── migrations/
│   │   │   ├── 001_initial.sql            # Core schema
│   │   │   ├── 002_frontier.sql           # URL frontier
│   │   │   ├── phase4c_observability.sql  # Telemetry tables
│   │   │   └── phase5c_multi_region.sql   # Multi-region schema
│   │   │
│   │   └── docker-compose.yml             # Local development stack
│   │
│   └── .env.example                       # Environment variables template
│
├── 📦 SDKs
│   ├── sdk/python/
│   │   ├── archivestream.py               # Python client
│   │   ├── README.md                      # Python SDK docs
│   │   └── setup.py                       # Package config
│   │
│   └── sdk/js/
│       ├── index.ts                       # TypeScript client
│       ├── README.md                      # JS SDK docs
│       └── package.json
│
└── 🔧 Configuration
    ├── .gitignore
    ├── .dockerignore
    └── LICENSE                            # MIT License
```

---

## 🎯 Key Components by Phase

### Phase 1: Foundation ✅
- `crates/crawler/` - Core crawling engine
- `crates/common/src/warc.rs` - WARC format support
- `infra/migrations/001_initial.sql` - Database schema

### Phase 2: Search & Discovery ✅
- `crates/indexer/` - OpenSearch integration
- `crates/common/src/extractor.rs` - Text extraction
- `crates/archive-api/src/search.rs` - Search API

### Phase 3: Horizontal Scaling ✅
- `crates/crawler/src/frontier.rs` - Distributed frontier
- `crates/crawler/src/dedup.rs` - Shared deduplication
- `infra/migrations/002_frontier.sql` - Frontier schema

### Phase 4A: Time-Travel Scrubber ✅
- `apps/web-ui/src/components/TimeScrubber.tsx` - Timeline UI
- `crates/archive-api/src/main.rs` - `/api/timeline` endpoint

### Phase 4B: Visual Differential ✅
- `crates/archive-api/src/diff.rs` - Diff engine
- `apps/web-ui/src/components/DiffViewer.tsx` - Diff UI

### Phase 4C: Crawl Health & Observability ✅
- `apps/web-ui/src/app/dashboard/` - Observability dashboard
- `crates/crawler/src/frontier.rs` - Event tracking
- `infra/migrations/phase4c_observability.sql` - Telemetry schema

### Phase 5A: Public API & SDK ✅
- `crates/archive-api/src/main.rs` - `/api/v1` router
- `sdk/python/archivestream.py` - Python SDK
- `sdk/js/index.ts` - JavaScript SDK
- `docs/API_V1.md` - API documentation

### Phase 5B: Semantic Change Classification ✅
- `crates/semantic/` - Classification engine
- `crates/archive-api/src/semantic.rs` - Semantic API
- `crates/common/src/extractor.rs` - Enhanced extraction

### Phase 5C: Multi-Region Frontier & Scale ✅
- `crates/crawler/src/region.rs` - Region routing
- `crates/crawler/src/rate_limit.rs` - Rate limiting
- `infra/migrations/phase5c_multi_region.sql` - Multi-region schema
- `docs/SCALING_MULTI_REGION.md` - Architecture guide

---

## 📊 Statistics

| Metric | Count |
|--------|-------|
| **Rust Crates** | 5 (common, crawler, indexer, semantic, archive-api) |
| **API Endpoints** | 12+ |
| **UI Components** | 10+ |
| **Database Tables** | 8 |
| **Documentation Files** | 8 |
| **SDK Languages** | 2 (Python, JavaScript) |
| **Supported Regions** | 3+ (expandable) |
| **Total Lines of Code** | ~15,000+ |

---

## 🚀 Quick Commands

```bash
# Build everything
cargo build --release

# Run crawler
cargo run --bin crawler

# Run indexer
cargo run --bin indexer

# Run API
cargo run --bin archive-api

# Run UI
cd apps/web-ui && npm run dev

# Run tests
cargo test
cd apps/web-ui && npm test

# Database migrations
psql $DATABASE_URL < infra/migrations/*.sql
```

---

## 🌟 Architecture Highlights

### Backend (Rust)
- **Async/await** with Tokio runtime
- **Axum** web framework
- **SQLx** for compile-time checked queries
- **OpenSearch** for full-text search
- **S3/MinIO** for WARC storage

### Frontend (Next.js)
- **App Router** (Next.js 14)
- **TypeScript** for type safety
- **Tailwind CSS** for styling
- **Lucide React** for icons

### Infrastructure
- **PostgreSQL/CockroachDB** for metadata
- **Docker Compose** for local dev
- **Kubernetes** for production (optional)

---

**This structure represents a complete, production-ready web archiving platform.** 🌐✨
