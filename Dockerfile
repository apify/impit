# Specify the base Docker image. You can read more about
# the available images at https://crawlee.dev/docs/guides/docker-images
# You can also use any other image from Docker Hub.
FROM apify/actor-node-playwright-chrome:22-1.58.1 AS builder

# Check preinstalled packages
RUN npm ls @crawlee/core apify puppeteer playwright

# Copy just package.json and package-lock.json
# to speed up the build using Docker layer cache.
COPY --chown=myuser:myuser package*.json Dockerfile ./

# Check Playwright version is the same as the one from base image.
RUN node check-playwright-version.mjs

# ---------------------------------------------------------------------------
# Build the impit native module for Linux x86_64
# ---------------------------------------------------------------------------
USER root
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl build-essential pkg-config libssl-dev cmake \
    && rm -rf /var/lib/apt/lists/*
USER myuser

# Install Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
ENV PATH="/home/myuser/.cargo/bin:${PATH}"

# Copy impit source and build the native module for Linux
# (.dockerignore excludes target/, node_modules/, and *.node from host)
COPY --chown=myuser:myuser impit/ ./impit/
USER root
RUN npm install -g @napi-rs/cli
USER myuser
RUN cd impit/impit-node \
    && napi build --platform --release --no-const-enum \
    && echo "Built native module:" && ls -la *.node \
    && rm -rf target ../target  # Clean up Rust build artifacts to save space

# ---------------------------------------------------------------------------
# Install project deps and build TypeScript
# ---------------------------------------------------------------------------
RUN npm install --include=dev --audit=false

# Copy source files
COPY --chown=myuser:myuser . ./

# Build TypeScript
RUN npm run build || true

# =========================================================================
# Create final image
# =========================================================================
FROM apify/actor-node-playwright-chrome:22-1.58.1

# Check preinstalled packages
RUN npm ls @crawlee/core apify puppeteer playwright

# Copy package.json (includes "impit": "file:./impit/impit-node")
COPY --chown=myuser:myuser package*.json ./

# Copy source files
COPY --chown=myuser:myuser . ./

# Copy the impit/impit-node directory with the Linux-built native module from builder.
# This MUST come after "COPY . ./" to ensure the Linux .node binary overwrites
# anything from the host context (macOS .node is excluded by .dockerignore anyway).
COPY --from=builder --chown=myuser:myuser /home/myuser/impit/impit-node/ ./impit/impit-node/

# Install NPM packages (production only)
RUN npm --quiet set progress=false \
    && npm install --omit=dev \
    && echo "Installed NPM packages:" \
    && (npm list --omit=dev --all || true) \
    && echo "Node.js version:" \
    && node --version \
    && echo "NPM version:" \
    && npm --version \
    && rm -r ~/.npm

# Verify the native module loads correctly with all browser profiles
RUN node -e " \
  const impit = require('impit'); \
  new impit.Impit({ browser: 'chrome' }); \
  new impit.Impit({ browser: 'safari18' }); \
  new impit.Impit({ browser: 'edge136' }); \
  console.log('impit native module OK — chrome, safari18, edge136 all working'); \
"

# Copy built JS files from builder image
COPY --from=builder --chown=myuser:myuser /home/myuser/dist ./dist

# Run the image.
CMD ["node", "dist/main.js"]
