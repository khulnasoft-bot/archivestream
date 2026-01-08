# ArchiveStream JS/TS SDK

The official JavaScript/TypeScript client for the ArchiveStream API.

## Usage
```typescript
import { ArchiveStream } from "./index";

const archive = new ArchiveStream("http://localhost:3001");

// Search
const results = await archive.search("privacy policy");

// Resolve
const resolved = await archive.resolve("https://example.com", "20240101000000");
console.log(resolved.replay_url);
```
