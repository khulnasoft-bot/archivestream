# ArchiveStream

**A production-grade, globally distributed, intelligent web archiving platform.**

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-production--ready-green.svg)]()

---

## 🌐 Overview

ArchiveStream is a **next-generation web archiving system** that captures, preserves, and makes accessible the evolving web. Unlike traditional archiving solutions, ArchiveStream provides:

- 🚀 **Global Scale**: Multi-region distributed crawling with intelligent domain routing
- 🧠 **Semantic Intelligence**: AI-powered change detection and categorization
- 🔍 **Full-Text Search**: Instant search across billions of archived pages
- ⏱️ **Time Travel**: Navigate temporal snapshots with visual diffs
- 🌍 **Public API**: RESTful API with Python and JavaScript SDKs
- 📊 **Real-Time Observability**: Live dashboard for crawl health and metrics

---

## ✨ Key Features

### Distributed Crawling
- **Stateless workers** that scale horizontally
- **Database-backed frontier** with atomic URL leasing
- **Global rate limiting** to respect crawl politeness
- **Multi-region support** with consistent hashing

### Intelligent Archiving
- **WARC format** (ISO 28500) with revisit records
- **70% deduplication** efficiency
- **Semantic change detection** (Privacy Policy, Price Changes, Breaking News)
- **Full-text indexing** via OpenSearch

### Developer Experience
- **Public API** with versioned endpoints (`/api/v1`)
- **Python SDK**: `pip install archivestream`
- **JavaScript SDK**: `npm install archivestream`
- **Comprehensive documentation**

### Observability
- **Real-time dashboard** showing frontier depth, success rates, and throughput
- **Per-region metrics** for multi-region deployments
- **Crawl event telemetry** with depth tracking
- **Health check endpoints**

---

## 🚀 Quick Start

### Prerequisites
- Rust 1.75+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- PostgreSQL 14+
- MinIO or S3
- OpenSearch 2.x
- Node.js 18+ (for UI)

### Installation

```bash
# Clone the repository
git clone https://github.com/ArchiveStream/archivestream.git
cd archivestream

# Start infrastructure
docker-compose up -d

# Run database migrations
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run --source infra/migrations

# Build and run
cargo build --release

# Start services
cargo run --bin crawler &
cargo run --bin indexer &
cargo run --bin archive-api &

# Start UI (in another terminal)
cd apps/web-ui
npm install
npm run dev
```

Visit `http://localhost:3000` to access the UI.

### One-Click Deployment

```bash
# Deploy to AWS, GCP, or Azure
./scripts/deploy.sh
```

The deployment script will:
- Detect your cloud environment
- Provision infrastructure with Terraform
- Deploy ArchiveStream with Helm
- Configure DNS and TLS

### Kubernetes with Helm

```bash
# Add Helm repository
helm repo add archivestream https://archivestream.github.io/charts
helm install my-archive archivestream/archivestream \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=archive.yourdomain.com
```

See [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) for detailed deployment options.

---

## 🤝 Community

Join our growing community:

- **Discord**: [discord.gg/archivestream](https://discord.gg/archivestream) - Real-time chat and support
- **GitHub Discussions**: [Share ideas and feedback](https://github.com/archivestream/archivestream/discussions)
- **Twitter**: [@archivestream](https://twitter.com/archivestream) - Latest updates
- **Blog**: [archivestream.org/blog](https://archivestream.org/blog) - Tutorials and case studies

### Community Plugins

Browse and contribute plugins at [archivestream/plugins](https://github.com/archivestream/plugins):

- `archivestream-plugin-reddit` - Reddit thread archiving
- `archivestream-plugin-twitter` - Twitter/X preservation  
- `archivestream-plugin-github` - GitHub repository snapshots
- `archivestream-plugin-ecommerce` - E-commerce price tracking

See [docs/PLUGIN_DEVELOPMENT.md](docs/PLUGIN_DEVELOPMENT.md) to create your own.

---

## 📖 Usage

### Crawling a Website

```bash
# Add a seed URL
curl -X POST http://localhost:3001/crawl \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'
```

### Searching the Archive

```bash
# Search for content
curl "http://localhost:3001/api/v1/search?q=privacy+policy"
```

### Using the Python SDK

```python
from archivestream import ArchiveStream

archive = ArchiveStream("http://localhost:3001")

# Search
results = archive.search("climate change")

# Get snapshots for a URL
snapshots = archive.get_snapshots("https://nytimes.com")

# Resolve best snapshot for a point in time
resolved = archive.resolve("https://example.com", "20240101000000")
print(f"Replay at: {resolved['replay_url']}")

# Semantic change analysis
semantic = archive.get_semantic(
    "https://example.com",
    from_ts="20240101000000",
    to_ts="20240201000000"
)
print(f"Categories: {semantic['classification']['categories']}")
```

### Using the JavaScript SDK

```typescript
import { ArchiveStream } from 'archivestream';

const archive = new ArchiveStream('http://localhost:3001');

// Search
const results = await archive.search('breaking news');

// Get timeline
const timeline = await archive.getTimeline('https://bbc.com');

// Semantic analysis
const semantic = await archive.getSemantic(
  'https://example.com',
  '20240101000000',
  '20240201000000'
);
console.log(semantic.classification.summary);
```

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Global Control Plane                   │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │ Region A │  │ Region B │  │ Region C │              │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘              │
└───────┼─────────────┼─────────────┼─────────────────────┘
        │             │             │
   ┌────▼─────────────▼─────────────▼────┐
   │   Distributed Frontier Queue        │
   │   (CockroachDB / Postgres Citus)    │
   └────────────────┬────────────────────┘
                    │
        ┌───────────┼───────────┐
        │           │           │
   ┌────▼────┐ ┌────▼────┐ ┌────▼────┐
   │Crawler  │ │Crawler  │ │Crawler  │
   │Workers  │ │Workers  │ │Workers  │
   └────┬────┘ └────┬────┘ └────┬────┘
        │           │           │
        └───────────┼───────────┘
                    │
         ┌──────────▼──────────┐
         │  S3 / MinIO Storage │
         └──────────┬──────────┘
                    │
        ┌───────────┼───────────┐
        │           │           │
   ┌────▼────┐ ┌────▼────┐ ┌────▼────┐
   │OpenSearch│ │PostgreSQL│ │archive-api│
   └─────────┘ └─────────┘ └────┬────┘
                                 │
                            ┌────▼────┐
                            │Next.js UI│
                            └─────────┘
```

See [IMPLEMENTATION_SUMMARY.md](docs/IMPLEMENTATION_SUMMARY.md) for detailed architecture.

---

## 📚 Documentation

- [Implementation Summary](docs/IMPLEMENTATION_SUMMARY.md) - Complete feature overview
- [API Documentation](docs/API_V1.md) - REST API reference
- [Scaling Guide](docs/SCALING.md) - Horizontal scaling strategies
- [Multi-Region Deployment](docs/SCALING_MULTI_REGION.md) - Global deployment guide
- [Phase 5C Status](docs/PHASE_5C_STATUS.md) - Latest development status

---

## 🛠️ Technology Stack

**Backend**:
- Rust (Tokio async runtime)
- Axum (Web framework)
- PostgreSQL / CockroachDB (Metadata)
- OpenSearch (Full-text search)
- S3 / MinIO (Object storage)

**Frontend**:
- Next.js 14 (React framework)
- TypeScript
- Tailwind CSS
- Lucide React (Icons)

**Infrastructure**:
- Docker & Docker Compose
- Kubernetes (production)
- GitHub Actions (CI/CD)

---

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js (via nvm)
nvm install 18

# Install dependencies
cargo build
cd apps/web-ui && npm install

# Run tests
cargo test
npm test
```

---

## 📊 Performance

| Metric | Single Region | Multi-Region (3x) |
|--------|--------------|-------------------|
| Crawl Throughput | 50 pages/sec | 150 pages/sec |
| Search Latency | <100ms | <100ms |
| Replay Latency | <200ms | <80ms |
| Storage Efficiency | 70% dedup | 70% dedup |
| Fault Tolerance | Single point | Region-resilient |

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- [WARC Format](https://iipc.github.io/warc-specifications/) - ISO 28500 standard
- [OpenSearch](https://opensearch.org/) - Search and analytics
- [Rust Community](https://www.rust-lang.org/community) - Amazing ecosystem

---

## 📞 Contact

- **Issues**: [GitHub Issues](https://github.com/ArchiveStream/archivestream/issues)
- **Discussions**: [GitHub Discussions](https://github.com/ArchiveStream/archivestream/discussions)
- **Email**: archivestream@ArchiveStream.com

---

**Built with ❤️ by the ArchiveStream Team**

*Preserving the web, one snapshot at a time.*
