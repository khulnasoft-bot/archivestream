# Contributing to ArchiveStream

**Welcome!** We're thrilled you're interested in contributing to ArchiveStream - the world's most intelligent, distributed web archiving platform.

---

## 🌟 Ways to Contribute

### 1. Code Contributions
- **Bug fixes**: Fix issues, improve error handling
- **Features**: Implement roadmap items or propose new ones
- **Performance**: Optimize crawling, search, or replay
- **Tests**: Add unit tests, integration tests, or benchmarks

### 2. Documentation
- **Guides**: Write tutorials, how-tos, or deployment guides
- **API docs**: Improve endpoint documentation
- **Translations**: Help translate docs to other languages
- **Examples**: Create sample applications using ArchiveStream

### 3. Community Support
- **Answer questions**: Help others on Discord or GitHub Discussions
- **Bug reports**: File detailed, reproducible issues
- **Feature requests**: Propose new capabilities with use cases
- **Blog posts**: Write about your ArchiveStream use cases

### 4. Infrastructure
- **Testing**: Run ArchiveStream at scale and report findings
- **Deployment**: Contribute Helm charts, Terraform modules, or Docker configs
- **Monitoring**: Improve observability dashboards or alerts

---

## 🚀 Getting Started

### Prerequisites

- **Rust**: 1.75+ (`rustup install stable`)
- **Node.js**: 18+ (`nvm install 18`)
- **PostgreSQL**: 14+
- **Docker**: For local infrastructure
- **Git**: For version control

### Local Development Setup

```bash
# 1. Fork and clone
git clone https://github.com/YOUR_USERNAME/archivestream.git
cd archivestream

# 2. Start infrastructure
docker-compose up -d postgres minio opensearch

# 3. Set up database
export DATABASE_URL="postgresql://admin:password@localhost/archivestream"
psql $DATABASE_URL < infra/migrations/001_initial.sql
psql $DATABASE_URL < infra/migrations/002_frontier.sql
psql $DATABASE_URL < infra/migrations/phase4c_observability.sql
psql $DATABASE_URL < infra/migrations/phase5c_multi_region.sql

# 4. Build and run
cargo build
cargo run --bin crawler &
cargo run --bin indexer &
cargo run --bin archive-api &

# 5. Start UI (in another terminal)
cd apps/web-ui
npm install
npm run dev
```

Visit `http://localhost:3000` to verify everything works.

---

## 📝 Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### 2. Make Changes

- Write clean, idiomatic Rust/TypeScript
- Follow existing code style
- Add tests for new functionality
- Update documentation as needed

### 3. Test Your Changes

```bash
# Run Rust tests
cargo test

# Run UI tests
cd apps/web-ui
npm test

# Run linter
cargo clippy -- -D warnings
npm run lint
```

### 4. Commit

We use [Conventional Commits](https://www.conventionalcommits.org/):

```bash
git commit -m "feat: add semantic category for content removal"
git commit -m "fix: resolve race condition in frontier leasing"
git commit -m "docs: update API examples for v1 endpoints"
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

### 5. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then open a Pull Request on GitHub with:
- **Clear title** (following conventional commits)
- **Description** of what changed and why
- **Screenshots** (for UI changes)
- **Related issues** (e.g., "Closes #123")

---

## 🎯 Code Style Guidelines

### Rust

```rust
// ✅ Good: Clear naming, error handling, documentation
/// Extracts semantic text from HTML content
pub fn extract_text(html: &str) -> Result<ExtractionResult> {
    let document = Html::parse_document(html);
    // ... implementation
}

// ❌ Bad: Unclear naming, unwrap without context
pub fn ext(h: &str) -> ExtractionResult {
    let d = Html::parse_document(h);
    d.select(&Selector::parse("body").unwrap()) // Don't unwrap!
}
```

**Guidelines**:
- Use `Result<T>` for fallible operations
- Avoid `.unwrap()` in library code
- Add doc comments (`///`) for public APIs
- Use `tracing::info!` for logging, not `println!`
- Run `cargo fmt` before committing

### TypeScript/React

```typescript
// ✅ Good: Typed props, clear component structure
interface FrontierHeatmapProps {
  data: FrontierMetric[];
  onDomainClick?: (domain: string) => void;
}

export const FrontierHeatmap: React.FC<FrontierHeatmapProps> = ({ data, onDomainClick }) => {
  // ... implementation
};

// ❌ Bad: Untyped, unclear purpose
export const Heatmap = ({ data }: any) => {
  // ... implementation
};
```

**Guidelines**:
- Always type props and state
- Use functional components with hooks
- Extract reusable logic into custom hooks
- Run `npm run lint` before committing
- Use Tailwind CSS for styling (avoid inline styles)

---

## 🧪 Testing Guidelines

### Unit Tests (Rust)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_routing_consistency() {
        let router = RegionRouter::new();
        let domain = "example.com";
        
        // Same domain should always route to same region
        let region1 = router.route_domain(domain);
        let region2 = router.route_domain(domain);
        assert_eq!(region1, region2);
    }
}
```

### Integration Tests

```bash
# Run integration tests (requires running infrastructure)
cargo test --test integration_tests
```

### UI Tests

```typescript
// apps/web-ui/src/components/__tests__/FrontierHeatmap.test.tsx
import { render, screen } from '@testing-library/react';
import { FrontierHeatmap } from '../FrontierHeatmap';

test('renders domain names', () => {
  const data = [{ domain: 'example.com', count: 100, depth_range: [0, 5] }];
  render(<FrontierHeatmap data={data} />);
  expect(screen.getByText('example.com')).toBeInTheDocument();
});
```

---

## 📚 Documentation Standards

### Code Comments

```rust
// ✅ Good: Explains WHY, not just WHAT
// Use consistent hashing to ensure the same domain always routes to the same region.
// This minimizes cross-region traffic and respects geo-blocking.
let region = self.region_router.route_domain(&domain);

// ❌ Bad: States the obvious
// Get the region for the domain
let region = self.region_router.route_domain(&domain);
```

### API Documentation

When adding new endpoints, update `docs/API_V1.md`:

```markdown
### Get Semantic Change

`GET /api/v1/semantic`

Analyzes semantic changes between two snapshots.

**Query Parameters**:
- `url` (string, required): The URL to analyze
- `from` (string, required): Start timestamp (YYYYMMDDHHMMSS)
- `to` (string, required): End timestamp (YYYYMMDDHHMMSS)

**Example**:
\`\`\`bash
curl "https://api.archivestream.com/api/v1/semantic?url=https://example.com&from=20240101&to=20240201"
\`\`\`
```

---

## 🐛 Bug Reports

Great bug reports include:

1. **Clear title**: "Crawler fails on URLs with non-ASCII characters"
2. **Environment**: OS, Rust version, database version
3. **Steps to reproduce**: Minimal, reproducible example
4. **Expected behavior**: What should happen
5. **Actual behavior**: What actually happens
6. **Logs/screenshots**: Relevant error messages

**Template**:

```markdown
## Bug Description
Crawler crashes when encountering URLs with emoji characters.

## Environment
- OS: Ubuntu 22.04
- Rust: 1.75.0
- PostgreSQL: 14.5

## Steps to Reproduce
1. Add URL with emoji: `https://example.com/🎉`
2. Start crawler
3. Observe crash

## Expected Behavior
URL should be crawled or gracefully skipped.

## Actual Behavior
Crawler panics with: `thread 'main' panicked at 'invalid UTF-8'`

## Logs
```
[ERROR] Failed to parse URL: https://example.com/🎉
thread 'main' panicked at 'invalid UTF-8'
```
```

---

## 💡 Feature Requests

Great feature requests include:

1. **Use case**: Why is this needed?
2. **Proposed solution**: How should it work?
3. **Alternatives considered**: What else did you think about?
4. **Additional context**: Screenshots, mockups, examples

**Template**:

```markdown
## Feature Request: Browser Extension

### Use Case
As a researcher, I want to archive pages I'm reading without leaving my browser.

### Proposed Solution
A Chrome/Firefox extension with:
- One-click archiving
- Show historical snapshots inline
- Highlight changes since last visit

### Alternatives Considered
- Bookmarklet (less convenient)
- Desktop app (requires context switching)

### Mockup
[Attach screenshot or wireframe]
```

---

## 🏆 Recognition

Contributors are recognized in:

- **README.md**: Top contributors section
- **Release notes**: Credited for their contributions
- **Hall of Fame**: `CONTRIBUTORS.md` with all contributors
- **Swag**: Stickers and t-shirts for significant contributions

---

## 📜 Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please:

- ✅ Be respectful and constructive
- ✅ Welcome newcomers and help them learn
- ✅ Focus on what's best for the community
- ✅ Show empathy towards other community members

- ❌ Don't use sexualized language or imagery
- ❌ Don't troll, insult, or make derogatory comments
- ❌ Don't harass others publicly or privately
- ❌ Don't publish others' private information

Violations may result in temporary or permanent ban from the project.

---

## 🔒 Security

Found a security vulnerability? **Do not open a public issue.**

Instead:
1. Email: security@archivestream.org
2. Include: Description, impact, reproduction steps
3. We'll respond within 48 hours
4. We'll credit you in the security advisory (if desired)

---

## 📞 Getting Help

- **Discord**: [discord.gg/archivestream](https://discord.gg/archivestream) - Real-time chat
- **GitHub Discussions**: For questions and ideas
- **Stack Overflow**: Tag with `archivestream`
- **Email**: hello@archivestream.org

---

## 🎓 Learning Resources

### For Rust Beginners
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

### For Web Archiving
- [WARC Format Spec](https://iipc.github.io/warc-specifications/)
- [Internet Archive's Technical Overview](https://archive.org/about/tech.php)
- [Web Archiving Bibliography](https://github.com/iipc/awesome-web-archiving)

### For Distributed Systems
- [Designing Data-Intensive Applications](https://dataintensive.net/)
- [Distributed Systems Course](https://www.youtube.com/playlist?list=PLeKd45zvjcDFUEv_ohr_HdUFe97RItdiB)

---

## 🙏 Thank You!

Every contribution, no matter how small, makes ArchiveStream better. Whether you:

- Fix a typo in the docs
- Report a bug
- Implement a major feature
- Answer a question on Discord

**You're helping preserve and understand the web's history. Thank you!** 🌐✨

---

**Happy Contributing!**

— The ArchiveStream Team
