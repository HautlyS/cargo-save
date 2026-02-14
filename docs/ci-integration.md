# CI/CD Integration

## Overview

cargo-save is designed to work seamlessly with CI/CD systems, providing significant build time improvements through intelligent caching.

## GitHub Actions

### Basic Setup

```yaml
name: Build

on: [push, pull_request]

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install cargo-save
        run: cargo install cargo-save
      
      - name: Cache cargo-save
        uses: actions/cache@v4
        with:
          path: ~/.cache/cargo-save
          key: ${{ runner.os }}-cargo-save-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-save-
      
      - name: Build
        run: cargo-save build --release
```

### Advanced Setup with Multiple Caches

```yaml
name: Build

on: [push, pull_request]

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full git history for accurate hashing
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install cargo-save
        run: cargo install cargo-save
      
      - name: Generate cache key
        id: cache-key
        run: echo "key=$(cargo-save cache-key --platform github)" >> $GITHUB_OUTPUT
      
      - name: Cache cargo-save
        uses: actions/cache@v4
        with:
          path: ~/.cache/cargo-save
          key: ${{ runner.os }}-cargo-save-${{ steps.cache-key.outputs.key }}
          restore-keys: |
            ${{ runner.os }}-cargo-save-
      
      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry/index
            ~/.cargo/registry/cache
            ~/.cargo/git/db
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Cache target directory
        uses: actions/cache@v4
        with:
          path: target
          key: ${{ runner.os }}-cargo-target-${{ hashFiles('**/Cargo.lock') }}-${{ hashFiles('**/*.rs') }}
          restore-keys: |
            ${{ runner.os }}-cargo-target-${{ hashFiles('**/Cargo.lock') }}-
            ${{ runner.os }}-cargo-target-
      
      - name: Warm cache
        run: cargo-save warm --release
      
      - name: Build
        run: cargo-save build --release
      
      - name: Run tests
        run: cargo-save test --release
      
      - name: Run clippy
        run: cargo-save clippy --release -- -D warnings
      
      - name: Upload build log on failure
        if: failure()
        run: |
          cargo-save query all --last 1 > build.log
          cat build.log
        continue-on-error: true
```

### Matrix Builds

```yaml
name: Build Matrix

on: [push, pull_request]

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta, nightly]
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust ${{ matrix.rust }}
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
      
      - name: Install cargo-save
        run: cargo install cargo-save
      
      - name: Cache cargo-save
        uses: actions/cache@v4
        with:
          path: |
            ~/.cache/cargo-save
            ~/Library/Caches/cargo-save
            ~\AppData\Local\cargo-save
          key: ${{ matrix.os }}-${{ matrix.rust }}-cargo-save-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Build
        run: cargo-save build --release
```

## GitLab CI

### Basic Setup

```yaml
variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo
  CARGO_TERM_COLOR: always

cache:
  key:
    files:
      - Cargo.lock
  paths:
    - target/
    - .cargo/
    - .cache/cargo-save/

stages:
  - build
  - test

build:
  stage: build
  image: rust:latest
  script:
    - cargo install cargo-save
    - cargo-save build --release
  artifacts:
    paths:
      - target/release/
    expire_in: 1 week

test:
  stage: test
  image: rust:latest
  script:
    - cargo install cargo-save
    - cargo-save test --release
  after_script:
    - if [ "$CI_JOB_STATUS" == "failed" ]; then cargo-save query all --last 1; fi
```

### Advanced Setup

```yaml
variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo
  CARGO_TERM_COLOR: always
  CARGO_SAVE_CACHE_DIR: $CI_PROJECT_DIR/.cache/cargo-save

.cache_template: &cache
  key:
    files:
      - Cargo.lock
    prefix: $CI_COMMIT_REF_SLUG
  paths:
    - target/
    - .cargo/
    - .cache/cargo-save/
  policy: pull-push

stages:
  - prepare
  - build
  - test
  - deploy

prepare:
  stage: prepare
  image: rust:latest
  <<: *cache
  script:
    - cargo install cargo-save
    - cargo-save warm --release
  only:
    changes:
      - Cargo.toml
      - Cargo.lock

build:
  stage: build
  image: rust:latest
  <<: *cache
  script:
    - cargo install cargo-save
    - cargo-save build --release
  artifacts:
    paths:
      - target/release/
    expire_in: 1 week

test:
  stage: test
  image: rust:latest
  <<: *cache
  script:
    - cargo install cargo-save
    - cargo-save test --release
    - cargo-save clippy --release -- -D warnings
  coverage: '/^\s*lines:\s*\d+\.\d+\%/'
  after_script:
    - if [ "$CI_JOB_STATUS" == "failed" ]; then cargo-save query errors; fi
```

## CircleCI

```yaml
version: 2.1

orbs:
  rust: circleci/rust@1.6

jobs:
  build:
    docker:
      - image: cimg/rust:1.75
    
    steps:
      - checkout
      
      - restore_cache:
          keys:
            - cargo-save-{{ checksum "Cargo.lock" }}
            - cargo-save-
      
      - run:
          name: Install cargo-save
          command: cargo install cargo-save
      
      - run:
          name: Build
          command: cargo-save build --release
      
      - save_cache:
          key: cargo-save-{{ checksum "Cargo.lock" }}
          paths:
            - ~/.cache/cargo-save
            - target
            - ~/.cargo
      
      - store_artifacts:
          path: target/release
          destination: binaries

workflows:
  version: 2
  build-and-test:
    jobs:
      - build
```

## Jenkins

```groovy
pipeline {
    agent any
    
    environment {
        CARGO_HOME = "${WORKSPACE}/.cargo"
        CARGO_SAVE_CACHE_DIR = "${WORKSPACE}/.cache/cargo-save"
    }
    
    stages {
        stage('Setup') {
            steps {
                sh 'cargo install cargo-save'
            }
        }
        
        stage('Build') {
            steps {
                sh 'cargo-save build --release'
            }
        }
        
        stage('Test') {
            steps {
                sh 'cargo-save test --release'
            }
        }
        
        stage('Clippy') {
            steps {
                sh 'cargo-save clippy --release -- -D warnings'
            }
        }
    }
    
    post {
        failure {
            sh 'cargo-save query errors'
        }
        always {
            archiveArtifacts artifacts: 'target/release/*', allowEmptyArchive: true
        }
    }
}
```

## Docker

### Multi-stage Build

```dockerfile
# Stage 1: Builder
FROM rust:1.75 as builder

# Install cargo-save
RUN cargo install cargo-save

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY */Cargo.toml ./*/

# Copy source
COPY . .

# Build with caching
RUN cargo-save build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/myapp /usr/local/bin/

CMD ["myapp"]
```

### With Cache Mount

```dockerfile
FROM rust:1.75 as builder

RUN cargo install cargo-save

WORKDIR /app

COPY . .

# Use BuildKit cache mount
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/root/.cache/cargo-save \
    cargo-save build --release && \
    cp target/release/myapp /usr/local/bin/

FROM debian:bookworm-slim
COPY --from=builder /usr/local/bin/myapp /usr/local/bin/
CMD ["myapp"]
```

## Cache Key Generation

### GitHub Actions Format
```bash
cargo-save cache-key --platform github
```

Output:
```
::set-output name=cache-key::cargo-save-a1b2c3d4e5f6g7h8-9a8b7c6d
cargo-save-a1b2c3d4e5f6g7h8-9a8b7c6d
```

### GitLab CI Format
```bash
cargo-save cache-key --platform gitlab
```

Output:
```
cargo-save-a1b2c3d4e5f6g7h8-9a8b7c6d
```

### Generic Format
```bash
cargo-save cache-key --platform generic
```

Output:
```
cargo-save-a1b2c3d4e5f6g7h8-9a8b7c6d
```

## Best Practices

### 1. Cache Multiple Directories
```yaml
- name: Cache all cargo artifacts
  uses: actions/cache@v4
  with:
    path: |
      ~/.cache/cargo-save
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

### 2. Use Restore Keys
```yaml
restore-keys: |
  ${{ runner.os }}-cargo-save-
  ${{ runner.os }}-cargo-
```

### 3. Warm Cache Before Build
```bash
cargo-save warm --release
cargo-save build --release
```

### 4. Clean Old Caches Periodically
```bash
cargo-save clean --days 7
```

### 5. Upload Build Logs on Failure
```bash
if [ "$CI_JOB_STATUS" == "failed" ]; then
  cargo-save query all --last 1 > build.log
  cat build.log
fi
```

## Performance Comparison

### Without cargo-save
```
Build time: 10-15 minutes (full rebuild every time)
Cache size: 2-5 GB (target directory only)
```

### With cargo-save
```
First build: 10-15 minutes (same as without)
Subsequent builds: 1-3 minutes (only changed packages)
Cache size: 2-5 GB (target) + 50-200 MB (cargo-save)
```

### Typical Savings
- **No changes**: 95% faster (instant)
- **One package changed**: 70-80% faster
- **Multiple packages changed**: 40-60% faster

## Troubleshooting

### Cache Not Restoring
**Problem:** Cache key doesn't match

**Solution:**
```bash
# Use consistent cache keys
cargo-save cache-key --platform github
```

### Cache Too Large
**Problem:** CI cache limit exceeded

**Solution:**
```bash
# Clean old caches
cargo-save clean --keep 5
```

### Git Not Available
**Problem:** Shallow clone in CI

**Solution:**
```yaml
- uses: actions/checkout@v4
  with:
    fetch-depth: 0  # Full history
```

### Build Still Slow
**Problem:** Not caching target directory

**Solution:**
```yaml
# Cache both cargo-save and target
- name: Cache
  uses: actions/cache@v4
  with:
    path: |
      ~/.cache/cargo-save
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

## Future Enhancements

1. **Distributed Cache**: Share cache across CI runners
2. **S3 Backend**: Store cache in S3 for persistence
3. **Cache Analytics**: Track cache hit rate in CI
4. **Automatic Cleanup**: Remove stale caches automatically
5. **Compression**: Compress cache for faster upload/download
