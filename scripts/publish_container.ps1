# Initialise buildx (Skip this if already done)
docker buildx create --use

# Login to GitHub Container Registry (Skip if already logged in)
$env:GH_TOKEN | docker login ghcr.io -u $env:GH_USERNAME --password-stdin

# Build and push multi-platform image
docker buildx build --platform linux/amd64, linux/arm64 --push -t ghcr.io/$env:GH_USERNAME/muninn:latesdocker buildx build --platform "linux/amd64,linux/arm64" `
    --push `
    -t ghcr.io/$env:GH_USERNAME/Muninn:latest .t