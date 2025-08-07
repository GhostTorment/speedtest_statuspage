#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VALID_PROFILES=("dev" "release" "test" "bench")

if [ $# -ne 1 ]; then
  echo "Usage: $0 <profile>"
  echo "Valid profiles:"
  printf "  - %s\n" "${VALID_PROFILES[@]}"
  exit 1
fi

PROFILE=$1
if [[ ! " ${VALID_PROFILES[*]} " =~ " ${PROFILE} " ]]; then
  echo "Error: Invalid profile '$PROFILE'."
  printf "Valid profiles:\n"
  printf "  - %s\n" "${VALID_PROFILES[@]}"
  exit 1
fi

echo "Cleaning previous build artifacts..."
(cd "$PROJECT_ROOT" && cargo clean)

echo "Building with profile: $PROFILE..."
BUILD_OUTPUT=$(cd "$PROJECT_ROOT" && cargo build --profile "$PROFILE" 2>&1)
echo "$BUILD_OUTPUT"

if [[ "$PROFILE" == "release" ]]; then
  if echo "$BUILD_OUTPUT" | grep -qi "warning:"; then
    echo "Build succeeded with warnings. Skipping version bump and packaging."
    exit 0
  fi

  echo "Build succeeded with no warnings. Proceeding with packaging..."

  # Get the binary path
  BIN_PATH="$PROJECT_ROOT/target/release/speedtest_statuspage"
  DEST_PATH="$PROJECT_ROOT/install/usr/local/bin"
  mkdir -p "$DEST_PATH"
  cp "$BIN_PATH" "$DEST_PATH/speedtest-statuspage"
  chmod +x "$DEST_PATH/speedtest-statuspage"
  echo "Binary copied to $DEST_PATH/speedtest-statuspage."

  # --- Increment Cargo.toml version ---
  echo "Incrementing version in Cargo.toml..."
  CARGO_TOML="$PROJECT_ROOT/Cargo.toml"
  CURRENT_VERSION=$(grep '^version' "$CARGO_TOML" | head -1 | cut -d '"' -f2)
  IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"
  NEW_VERSION="${MAJOR}.${MINOR}.$((PATCH + 1))"
  sed -i.bak "s/^version *= *\"[0-9.]*\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML"
  echo "Version bumped to $NEW_VERSION"

  # Rebuild with new version
  echo "Rebuilding with new version..."
  (cd "$PROJECT_ROOT" && cargo build --release)

  # --- Git Commit & Push ---
  read -rp "Enter Git commit message: " COMMIT_MSG
  cd "$PROJECT_ROOT"
  git add Cargo.toml Cargo.toml.bak
  git commit -am "$COMMIT_MSG"
  git push

  # --- Docker Build ---
  read -rp "Enter your DockerHub username or org (e.g. myuser): " DOCKER_REPO
  IMAGE_NAME="$DOCKER_REPO/speedtest_statuspage"

  echo "Building Docker images..."
  docker build -t "$IMAGE_NAME:latest" -f Dockerfile .
  docker build -t "$IMAGE_NAME:latest-alpine" -f Dockerfile.alpine .

  docker tag "$IMAGE_NAME:latest" "$IMAGE_NAME:$NEW_VERSION"
  docker tag "$IMAGE_NAME:latest-alpine" "$IMAGE_NAME:$NEW_VERSION-alpine"

  # --- Docker Test Function ---
  test_docker_image() {
    local image="$1"
    echo "Testing image $image..."

    # Run the container in detached mode and wait briefly
    CONTAINER_ID=$(docker run -d --rm "$image" || true)

    if [ -z "$CONTAINER_ID" ]; then
      echo "❌ Failed to start container for image: $image"
      exit 1
    fi

    sleep 2  # Wait for the container to settle

    # Check container is still running (or exited cleanly)
    STATUS=$(docker inspect -f '{{.State.Status}}' "$CONTAINER_ID" 2>/dev/null || echo "notfound")

    if [[ "$STATUS" != "running" && "$STATUS" != "exited" ]]; then
      echo "❌ Container for $image failed (status: $STATUS)"
      docker logs "$CONTAINER_ID"
      exit 1
    fi

    echo "✅ Image $image passed test (status: $STATUS)"
  }

  # --- Test Docker Images Before Pushing ---
  test_docker_image "$IMAGE_NAME:latest"
  test_docker_image "$IMAGE_NAME:latest-alpine"
  test_docker_image "$IMAGE_NAME:$NEW_VERSION"
  test_docker_image "$IMAGE_NAME:$NEW_VERSION-alpine"

  # --- Docker Push ---
  echo "Logging into DockerHub..."
  docker login

  echo "Pushing Docker images to DockerHub..."
  docker push "$IMAGE_NAME:latest"
  docker push "$IMAGE_NAME:latest-alpine"
  docker push "$IMAGE_NAME:$NEW_VERSION"
  docker push "$IMAGE_NAME:$NEW_VERSION-alpine"

else
  echo "Build complete for profile '$PROFILE'. Skipping packaging steps."
fi