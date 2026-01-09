#!/bin/bash
set -e

echo "🚀 ArchiveStream One-Click Deployment"
echo "======================================"
echo ""

# Detect cloud provider
if command -v aws &> /dev/null; then
    CLOUD="aws"
elif command -v gcloud &> /dev/null; then
    CLOUD="gcp"
elif command -v az &> /dev/null; then
    CLOUD="azure"
else
    CLOUD="local"
fi

echo "Detected environment: $CLOUD"
echo ""

# Prompt for configuration
read -p "Enter your domain (e.g., archive.example.com): " DOMAIN
read -sp "Enter PostgreSQL password: " DB_PASSWORD
echo ""
read -p "Enter OpenAI API key (optional, press Enter to skip): " OPENAI_KEY
echo ""

case $CLOUD in
    aws)
        echo "📦 Deploying to AWS..."
        cd infra/terraform/aws
        
        terraform init
        terraform apply -auto-approve \
            -var="db_password=$DB_PASSWORD" \
            -var="cluster_name=archivestream-prod"
        
        # Get outputs
        CLUSTER_NAME=$(terraform output -raw cluster_name)
        
        # Configure kubectl
        aws eks update-kubeconfig --name $CLUSTER_NAME --region us-east-1
        
        # Install with Helm
        cd ../../..
        helm install archivestream ./infra/helm/archivestream \
            --set ingress.enabled=true \
            --set ingress.hosts[0].host=$DOMAIN \
            --set config.openaiApiKey=$OPENAI_KEY
        
        echo ""
        echo "✅ Deployment complete!"
        echo "🌐 Access your instance at: https://$DOMAIN"
        ;;
        
    gcp)
        echo "📦 Deploying to Google Cloud..."
        echo "⚠️  GCP deployment coming soon. Use Helm manually for now."
        ;;
        
    azure)
        echo "📦 Deploying to Azure..."
        echo "⚠️  Azure deployment coming soon. Use Helm manually for now."
        ;;
        
    local)
        echo "📦 Deploying locally with Docker Compose..."
        
        # Update .env file
        cat > .env << EOF
DATABASE_URL=postgresql://admin:$DB_PASSWORD@postgres:5432/archivestream
OPENAI_API_KEY=$OPENAI_KEY
EOF
        
        # Start services
        docker-compose -f docker-compose.prod.yml up -d
        
        echo ""
        echo "✅ Deployment complete!"
        echo "🌐 Access your instance at: http://localhost:3000"
        echo "📊 MinIO Console: http://localhost:9001 (admin/minioadmin)"
        echo "🔍 OpenSearch: http://localhost:9200"
        ;;
esac

echo ""
echo "📚 Next steps:"
echo "  1. Visit the UI and create your first crawl"
echo "  2. Install the browser extension from apps/extension"
echo "  3. Read the docs at docs/API.md"
echo ""
echo "Need help? Join our Discord: https://discord.gg/archivestream"
