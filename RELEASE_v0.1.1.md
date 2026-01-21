# Release v0.1.1

## How to Create the Release

Once the PR is merged to main, create the release with these steps:

1. **Create and push the tag:**
   ```bash
   git checkout main
   git pull
   git tag -a v0.1.1 -m "Release v0.1.1 - Fix isDueCompleted field deserialization"
   git push origin v0.1.1
   ```

2. **Create GitHub release:**
   - Go to: https://github.com/cmoi936/planka-mcp/releases/new
   - Tag: `v0.1.1`
   - Title: `v0.1.1 - Fix isDueCompleted field deserialization`
   - Description: (see below)

---

## Release Notes

### Fixed
- Fixed deserialization of `isDueCompleted` field in Card struct
  - The Planka API returns `isDueDateCompleted` but the struct was expecting `isDueCompleted` after camelCase translation
  - Added `#[serde(rename = "isDueDateCompleted")]` attribute to correctly map the field from Planka API response
  - Applied the same fix to CreateCardRequest struct for consistency in serialization

### What's Changed
- Fix isDueCompleted field deserialization to use isDueDateCompleted from API
- Bump version to 0.1.1
- Add CHANGELOG.md for v0.1.1 release

**Full Changelog**: https://github.com/cmoi936/planka-mcp/compare/v0.1.0...v0.1.1

---

## Docker Image

The Docker image will be automatically built and published to GitHub Container Registry when the tag is pushed:

```bash
docker pull ghcr.io/cmoi936/planka-mcp:0.1.1
# or
docker pull ghcr.io/cmoi936/planka-mcp:latest
```
