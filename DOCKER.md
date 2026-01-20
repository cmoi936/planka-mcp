# Docker and GHCR Publishing

This document explains the Docker setup and GitHub Container Registry (GHCR) publishing workflow for the planka-mcp project.

## Dockerfile

The `Dockerfile` uses a multi-stage build approach for efficient and secure containerization:

### Build Stage
- **Base Image**: `rust:1.83-slim` - Minimal Rust toolchain
- **Process**:
  1. Installs build dependencies (pkg-config, libssl-dev)
  2. Copies `Cargo.toml` and `Cargo.lock` first for dependency caching
  3. Creates a dummy `main.rs` and builds dependencies (cached layer)
  4. Copies actual source code and builds the final binary

### Runtime Stage
- **Base Image**: `debian:bookworm-slim` - Minimal Debian base
- **Process**:
  1. Installs runtime dependencies (ca-certificates for HTTPS)
  2. Copies only the compiled binary from build stage
  3. Creates a non-root user (`planka`) for security
  4. Sets RUST_LOG environment variable for logging

### Key Features
- **Multi-stage build**: Reduces final image size by excluding build tools
- **Layer caching**: Separates dependency and source builds for faster rebuilds
- **Security**: Runs as non-root user
- **Minimal size**: Final image contains only runtime essentials

## GitHub Actions Workflow

The `.github/workflows/docker-publish.yml` workflow automates Docker image building and publishing:

### Triggers
- **Push to main branch**: Builds and pushes `latest` tag
- **Version tags** (e.g., `v1.0.0`): Builds and pushes versioned tags
- **Pull requests**: Builds but doesn't push (validation only)

### Image Tags
The workflow automatically generates multiple tags:
- `latest` - Always points to the latest main branch build
- `main` - Tracks the main branch
- `v1.0.0`, `v1.0`, `v1` - Semantic version tags (when pushing version tags)
- `main-<sha>` - Commit SHA tags for traceability

### Multi-Architecture Support
The workflow builds for:
- `linux/amd64` (x86_64)
- `linux/arm64` (ARM64/Apple Silicon)

### Caching
Uses GitHub Actions cache to speed up builds:
- `cache-from: type=gha` - Reads from previous build caches
- `cache-to: type=gha,mode=max` - Saves all layers for maximum caching

## Usage

### Pull the Image
```bash
docker pull ghcr.io/cmoi936/planka-mcp:latest
```

### Run the Container
```bash
docker run -it --rm \
  -e PLANKA_URL="https://kanban.local" \
  -e PLANKA_TOKEN="your-token" \
  ghcr.io/cmoi936/planka-mcp:latest
```

### Use in MCP Configuration
```json
{
  "mcpServers": {
    "planka": {
      "command": "docker",
      "args": [
        "run",
        "-i",
        "--rm",
        "ghcr.io/cmoi936/planka-mcp:latest"
      ],
      "env": {
        "PLANKA_URL": "https://kanban.local",
        "PLANKA_TOKEN": "your-token"
      }
    }
  }
}
```

## Publishing Process

When changes are merged to main or a version tag is pushed:

1. GitHub Actions checks out the repository
2. Sets up Docker Buildx for multi-platform builds
3. Authenticates with GitHub Container Registry
4. Extracts metadata (tags, labels)
5. Builds the Docker image for multiple architectures
6. Pushes the image to `ghcr.io/cmoi936/planka-mcp`

## Manual Build (Local)

To build the image locally:

```bash
# Build for current architecture
docker build -t planka-mcp:local .

# Build for specific platform
docker build --platform linux/amd64 -t planka-mcp:local .

# Run local build
docker run -it --rm \
  -e PLANKA_URL="https://kanban.local" \
  -e PLANKA_TOKEN="your-token" \
  planka-mcp:local
```

## Security Considerations

1. **Non-root user**: The container runs as user `planka` (UID 1000)
2. **Minimal base**: Uses `debian:bookworm-slim` to reduce attack surface
3. **No secrets in image**: All configuration via environment variables
4. **HTTPS support**: Includes ca-certificates for secure connections
5. **Read-only binary**: Binary is owned by root, preventing tampering

## Troubleshooting

### Image not found
Ensure the workflow has run successfully and the image is published to GHCR.

### Authentication errors
For private repositories, authenticate with GHCR:
```bash
echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin
```

### Build failures
Check the GitHub Actions logs for detailed error messages. Common issues:
- Dependency conflicts in `Cargo.lock`
- Network issues during crate downloads
- Out of disk space during multi-platform builds
