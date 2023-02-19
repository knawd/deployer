#! /bin/bash
export MANIFEST_NAME="deployer-multiarch"

# Set the required variables
export BUILD_PATH="."
export REGISTRY="quay.io"
export USER="knawd"
export IMAGE_NAME="deployer"
export IMAGE_TAG="v1.2.0"

cd ../

# Create a multi-architecture manifest
buildah manifest create ${MANIFEST_NAME}

# Build your amd64 architecture container
buildah bud \
    --tag "${REGISTRY}/${USER}/${IMAGE_NAME}:${IMAGE_TAG}" \
    --manifest ${MANIFEST_NAME} \
    --arch amd64 \
    ${BUILD_PATH}

# Build your arm64 architecture container
buildah bud \
    --tag "${REGISTRY}/${USER}/${IMAGE_NAME}:${IMAGE_TAG}" \
    --manifest ${MANIFEST_NAME} \
    --arch aarch64 \
    --build-arg ARCH=aarch64
    ${BUILD_PATH}

# # Push the full manifest, with both CPU Architectures
buildah manifest push --all \
    ${MANIFEST_NAME} \
    ${REGISTRY}/${USER}/${IMAGE_NAME}:${IMAGE_TAG}"