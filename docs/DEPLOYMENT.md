# ArchiveStream Deployment Guide

## Quick Start Options

### 1. Docker Compose (Recommended for Development)

```bash
# Clone the repository
git clone https://github.com/archivestream/archivestream.git
cd archivestream

# Start all services
docker-compose -f docker-compose.prod.yml up -d

# Access the UI
open http://localhost:3000
```

### 2. Kubernetes with Helm

```bash
# Add Helm repository (once published)
helm repo add archivestream https://archivestream.github.io/charts
helm repo update

# Install with default values
helm install my-archive archivestream/archivestream

# Or install from local chart
helm install my-archive ./infra/helm/archivestream \
  --set config.openaiApiKey=sk-... \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=archive.yourdomain.com
```

### 3. AWS with Terraform

```bash
cd infra/terraform/aws

# Initialize Terraform
terraform init

# Review the plan
terraform plan -var="db_password=YOUR_SECURE_PASSWORD"

# Deploy infrastructure
terraform apply -var="db_password=YOUR_SECURE_PASSWORD"

# Get EKS cluster credentials
aws eks update-kubeconfig --name archivestream-prod --region us-east-1

# Deploy ArchiveStream with Helm
helm install archivestream ./../../helm/archivestream \
  --set config.databaseUrl="postgres://admin:PASSWORD@RDS_ENDPOINT:5432/archivestream" \
  --set config.s3Endpoint="https://s3.amazonaws.com"
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://admin:password@localhost/archivestream` |
| `S3_ENDPOINT` | S3-compatible storage endpoint | `http://localhost:9000` |
| `OPENSEARCH_URL` | OpenSearch cluster URL | `http://localhost:9200` |
| `OPENAI_API_KEY` | OpenAI API key (optional, for ML features) | - |
| `REDIS_URL` | Redis connection string | `redis://localhost:6379` |

### Scaling Considerations

- **Crawlers**: Horizontally scalable. Run 1 crawler per region for optimal performance.
- **API**: Stateless, can be scaled to N replicas behind a load balancer.
- **Database**: Use managed PostgreSQL (RDS, Cloud SQL) for production.
- **Storage**: Use S3, GCS, or Azure Blob Storage for WARC files.

## Monitoring

ArchiveStream exposes Prometheus metrics at `/metrics` on the API service:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'archivestream'
    static_configs:
      - targets: ['archivestream-api:3001']
```

## Backup & Recovery

```bash
# Backup PostgreSQL
pg_dump -h localhost -U admin archivestream > backup.sql

# Backup WARC files (if using MinIO)
mc mirror local/archivestream-minio/archives s3/backup-bucket/archives
```

## Troubleshooting

### Crawler not processing URLs
- Check `url_frontier` table for pending URLs
- Verify crawler logs: `docker logs archivestream-crawler-us-east-1`
- Ensure robots.txt compliance is not blocking

### Search not returning results
- Verify OpenSearch is healthy: `curl http://localhost:9200/_cluster/health`
- Check indexer logs: `docker logs archivestream-indexer`
- Manually trigger reindex if needed

### Replay showing blank pages
- Check WARC file integrity in S3/MinIO
- Verify `snapshots` table has correct `warc_file`, `offset`, `length`
- Review API logs for rewriter errors
