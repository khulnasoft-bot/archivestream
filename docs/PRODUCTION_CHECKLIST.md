# Production Deployment Checklist

**ArchiveStream v1.0 - Production Readiness**

---

## ✅ Pre-Deployment

### Infrastructure
- [ ] CockroachDB cluster deployed (3+ nodes across regions)
- [ ] S3 buckets created with versioning enabled
- [ ] OpenSearch cluster deployed (3+ nodes)
- [ ] Load balancer configured for `archive-api`
- [ ] CDN configured for static assets
- [ ] DNS records configured
- [ ] SSL certificates provisioned

### Database
- [ ] Run all migrations in order:
  ```bash
  psql $DATABASE_URL < infra/migrations/001_initial.sql
  psql $DATABASE_URL < infra/migrations/002_frontier.sql
  psql $DATABASE_URL < infra/migrations/phase4c_observability.sql
  psql $DATABASE_URL < infra/migrations/phase5c_multi_region.sql
  ```
- [ ] Create database indexes
- [ ] Set up automated backups (daily)
- [ ] Configure connection pooling
- [ ] Test failover scenarios

### Security
- [ ] API keys generated for external access
- [ ] Rate limiting configured
- [ ] CORS policies set
- [ ] Firewall rules applied
- [ ] Secrets stored in vault (not environment variables)
- [ ] TLS 1.3 enforced
- [ ] Security headers configured (HSTS, CSP, etc.)

### Monitoring
- [ ] Prometheus metrics endpoint exposed
- [ ] Grafana dashboards imported
- [ ] Alert rules configured:
  - [ ] Crawler failure rate > 10%
  - [ ] Frontier queue depth > 1M
  - [ ] API latency p95 > 500ms
  - [ ] Database connection pool exhaustion
  - [ ] Disk usage > 80%
- [ ] PagerDuty/OpsGenie integration
- [ ] Log aggregation (ELK/Datadog)

---

## 🚀 Deployment Steps

### 1. Deploy Database Layer
```bash
# CockroachDB cluster
cockroach start \
  --advertise-addr=<node-ip> \
  --join=<cluster-nodes> \
  --certs-dir=/certs \
  --store=/data

# Run migrations
cockroach sql --url="postgresql://root@<cluster>:26257/archivestream?sslmode=verify-full" \
  < infra/migrations/*.sql
```

### 2. Deploy Storage Layer
```bash
# S3 bucket creation
aws s3 mb s3://archivestream-warc-us-east-1
aws s3 mb s3://archivestream-warc-eu-west-1
aws s3 mb s3://archivestream-warc-ap-south-1

# Enable versioning
aws s3api put-bucket-versioning \
  --bucket archivestream-warc-us-east-1 \
  --versioning-configuration Status=Enabled

# Set lifecycle policies (move to Glacier after 90 days)
aws s3api put-bucket-lifecycle-configuration \
  --bucket archivestream-warc-us-east-1 \
  --lifecycle-configuration file://lifecycle.json
```

### 3. Deploy OpenSearch
```bash
# Helm chart deployment (Kubernetes)
helm install opensearch opensearch/opensearch \
  --set replicas=3 \
  --set persistence.size=500Gi \
  --set resources.requests.memory=16Gi

# Create index template
curl -X PUT "https://opensearch:9200/_index_template/snapshots" \
  -H 'Content-Type: application/json' \
  -d @opensearch-template.json
```

### 4. Deploy Crawler Workers
```bash
# Region A (us-east-1)
docker run -d \
  --name crawler-us-east-1 \
  -e REGION=us-east-1 \
  -e DATABASE_URL=$DATABASE_URL \
  -e S3_ENDPOINT=https://s3.us-east-1.amazonaws.com \
  archivestream/crawler:latest

# Region B (eu-west-1)
docker run -d \
  --name crawler-eu-west-1 \
  -e REGION=eu-west-1 \
  -e DATABASE_URL=$DATABASE_URL \
  -e S3_ENDPOINT=https://s3.eu-west-1.amazonaws.com \
  archivestream/crawler:latest

# Region C (ap-south-1)
docker run -d \
  --name crawler-ap-south-1 \
  -e REGION=ap-south-1 \
  -e DATABASE_URL=$DATABASE_URL \
  -e S3_ENDPOINT=https://s3.ap-south-1.amazonaws.com \
  archivestream/crawler:latest
```

### 5. Deploy Indexer
```bash
docker run -d \
  --name indexer \
  -e DATABASE_URL=$DATABASE_URL \
  -e OPENSEARCH_URL=$OPENSEARCH_URL \
  archivestream/indexer:latest
```

### 6. Deploy API
```bash
docker run -d \
  --name archive-api \
  -p 3001:3001 \
  -e DATABASE_URL=$DATABASE_URL \
  -e OPENSEARCH_URL=$OPENSEARCH_URL \
  -e S3_ENDPOINT=$S3_ENDPOINT \
  archivestream/archive-api:latest
```

### 7. Deploy UI
```bash
docker run -d \
  --name web-ui \
  -p 3000:3000 \
  -e NEXT_PUBLIC_API_URL=https://api.archivestream.com \
  archivestream/web-ui:latest
```

---

## 🧪 Post-Deployment Validation

### Smoke Tests
```bash
# Health check
curl https://api.archivestream.com/health
# Expected: {"status":"ok"}

# Add seed URL
curl -X POST https://api.archivestream.com/crawl \
  -H "Content-Type: application/json" \
  -d '{"url":"https://example.com"}'

# Wait 30 seconds, then search
curl "https://api.archivestream.com/api/v1/search?q=example"
# Expected: Non-empty results array

# Check frontier health
curl https://api.archivestream.com/api/health/frontier
# Expected: Regional stats

# Check dashboard
open https://archivestream.com/dashboard
# Expected: Live metrics visible
```

### Load Testing
```bash
# Use k6 or Apache Bench
k6 run load-test.js

# Expected thresholds:
# - p95 latency < 500ms
# - Error rate < 1%
# - Throughput > 100 req/sec
```

### Failover Testing
```bash
# Kill one crawler region
docker stop crawler-us-east-1

# Verify:
# - Other regions pick up work
# - No URLs lost
# - Dashboard shows region as down

# Restart
docker start crawler-us-east-1

# Verify:
# - Region rejoins cluster
# - Work rebalances
```

---

## 📊 Monitoring Dashboards

### Key Metrics to Track

**Crawler Health**:
- URLs crawled per second (by region)
- Success rate (target: >99%)
- Frontier queue depth (by region)
- Average crawl latency

**Storage**:
- WARC files written per hour
- Deduplication rate (target: >70%)
- S3 storage used
- S3 API error rate

**Search**:
- Documents indexed per second
- Search query latency p95 (target: <100ms)
- Index size
- OpenSearch cluster health

**API**:
- Request rate
- Latency p50, p95, p99
- Error rate (target: <0.1%)
- Active connections

---

## 🔧 Operational Runbooks

### Scaling Up Crawlers
```bash
# Add more workers to a region
docker run -d \
  --name crawler-us-east-1-worker-2 \
  -e REGION=us-east-1 \
  -e DATABASE_URL=$DATABASE_URL \
  archivestream/crawler:latest
```

### Handling High Frontier Queue
```bash
# Check queue depth
curl https://api.archivestream.com/api/health/frontier

# If > 1M URLs:
# 1. Scale up crawlers (add 5-10 workers per region)
# 2. Check for stuck URLs (fetch_attempts > 5)
# 3. Investigate domain-specific failures
```

### Database Maintenance
```bash
# Vacuum old rate limit windows
psql $DATABASE_URL -c "DELETE FROM rate_limits WHERE window_start < NOW() - INTERVAL '24 hours';"

# Analyze query performance
psql $DATABASE_URL -c "SELECT * FROM pg_stat_statements ORDER BY total_time DESC LIMIT 10;"

# Reindex if needed
psql $DATABASE_URL -c "REINDEX TABLE url_frontier;"
```

### Emergency Procedures

**Complete Outage**:
1. Check database connectivity
2. Verify S3 access
3. Check OpenSearch cluster
4. Review recent deployments
5. Check for DDoS attack
6. Failover to backup region

**Data Corruption**:
1. Stop all crawlers immediately
2. Restore from latest backup
3. Replay WAL logs
4. Verify data integrity
5. Resume crawling

---

## 📈 Capacity Planning

### Current Capacity (3 regions, 10 workers each)
- **Crawl Rate**: 150 pages/second
- **Daily Volume**: ~13M pages
- **Monthly Volume**: ~390M pages
- **Storage**: ~2TB/month (after dedup)

### Scaling Targets
| Workers | Pages/Sec | Daily Volume | Monthly Storage |
|---------|-----------|--------------|-----------------|
| 30      | 150       | 13M          | 2TB             |
| 60      | 300       | 26M          | 4TB             |
| 120     | 600       | 52M          | 8TB             |
| 240     | 1200      | 104M         | 16TB            |

---

## ✅ Production Readiness Checklist

### Before Go-Live
- [ ] All smoke tests passing
- [ ] Load tests passing
- [ ] Failover tests passing
- [ ] Monitoring dashboards configured
- [ ] Alerts configured and tested
- [ ] Runbooks documented
- [ ] On-call rotation established
- [ ] Backup/restore tested
- [ ] Security audit completed
- [ ] Performance benchmarks met

### Day 1
- [ ] Monitor error rates closely
- [ ] Watch for memory leaks
- [ ] Verify deduplication working
- [ ] Check S3 costs
- [ ] Validate search results
- [ ] Test API rate limiting

### Week 1
- [ ] Review all alerts
- [ ] Tune rate limits if needed
- [ ] Optimize slow queries
- [ ] Adjust worker counts
- [ ] Review storage costs
- [ ] Gather user feedback

---

## 🎯 Success Criteria

After 1 week in production:
- ✅ 99.9% uptime
- ✅ <1% error rate
- ✅ p95 latency < 500ms
- ✅ >70% deduplication rate
- ✅ Zero data loss
- ✅ All regions operational

---

**Deployment Lead**: _____________  
**Date**: _____________  
**Sign-off**: _____________
