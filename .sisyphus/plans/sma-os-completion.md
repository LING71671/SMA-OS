# SMA-OS Project Completion Plan

## TL;DR

> **Complete SMA-OS v2.0 by implementing stub modules and adding missing infrastructure.**
>
> **Deliverables**:
> - fractal-gateway: eBPF authentication gateway with IAM
> - vector-kv: FoundationDB/Weaviate/Redis hybrid storage
> - chaos-tests: Resource exhaustion & network partition scenarios
> - test-all scripts: PowerShell + Bash unified testing
> - CI/CD: GitHub Actions workflow
> - Frontend tests: Jest + React Testing Library
>
> **Estimated Effort**: Large (6 major work streams, ~40 tasks)
> **Parallel Execution**: YES - 6 waves
> **Critical Path**: Infrastructure → Core Modules → Integration → Verification

---

## Context

### Original Request
Complete the SMA-OS project by implementing:
1. **fractal-gateway** - Currently stub `add(2,2)`, needs eBPF auth gateway
2. **vector-kv** - Mocked, needs real DB connections (FoundationDB/Weaviate/Redis)
3. **chaos-tests** - Complete TODO scenarios (resource exhaustion, network partition)
4. **test-all scripts** - Missing unified testing
5. **CI/CD** - No GitHub Actions
6. **Frontend tests** - Zero test coverage

### Interview Summary
**Key Discussions**:
- User confirmed all 6 work streams are required
- Must follow existing AGENTS.md conventions
- Must integrate with existing docker-compose infrastructure
- Zero-compromise security (no bypassing auth)

### Research Findings
- **fractal-gateway-ebpf** exists and is fully implemented (7 source files)
- **docker-compose.yml** has all required dependencies (Redis, Weaviate, etc.)
- **17 AGENTS.md** files with complete conventions
- **Chaos framework** exists, just needs 2 scenario implementations

---

## Work Objectives

### Core Objective
Implement all stub modules to production quality and establish complete CI/CD pipeline.

### Concrete Deliverables
1. `control-plane/fractal-gateway/src/` - Full auth gateway implementation
2. `memory-bus/vector-kv/` - Real DB connections (FoundationDB/Weaviate/Redis)
3. `chaos-tests/src/scenarios/` - Complete resource_exhaustion.rs & network_partition.rs
4. `test-all.ps1` & `test-all.sh` - Unified testing scripts
5. `.github/workflows/ci.yml` - GitHub Actions CI/CD
6. `observability-ui/web-dashboard/` - Jest + RTL test setup + tests

### Definition of Done
- [ ] All Rust modules compile with `cargo build --release`
- [ ] All Go modules compile with `go build`
- [ ] All tests pass (`cargo test`, `go test`, `npm test`)
- [ ] CI/CD pipeline passes on GitHub
- [ ] Chaos tests run with `--scenario all` successfully
- [ ] Frontend tests achieve >80% coverage
- [ ] AGENTS.md updated where needed
- [ ] Documentation reflects reality

### Must Have
- eBPF program loading in fractal-gateway
- IAM policy enforcement
- Real DB connections in vector-kv (not mocks)
- Working chaos scenarios with actual resource/network manipulation
- Cross-platform test scripts (PowerShell + Bash)
- GitHub Actions with build matrix (Rust, Go, Node)
- Frontend test infrastructure + initial tests

### Must NOT Have (Guardrails)
- NO mock implementations in production code
- NO hardcoded credentials or API keys
- NO bypass of security checks for "convenience"
- NO scope creep into unrelated modules
- NO removal of existing working functionality
- AI-Slop patterns: excessive comments, generic names, over-abstraction

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES (existing test files)
- **Automated tests**: YES (TDD for new code)
- **Frameworks**: 
  - Rust: `cargo test` with `tokio::test`
  - Go: `go test` with standard library
  - TypeScript: Jest + React Testing Library
- **If TDD**: New code follows RED-GREEN-REFACTOR

### QA Policy
Every task MUST include agent-executed QA scenarios:
- **Backend**: `curl` for API endpoints, `cargo test`/`go test` for unit tests
- **Frontend**: Playwright for UI verification
- **Scripts**: Bash/PowerShell execution verification
- **CI/CD**: Workflow run verification on GitHub

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately - Infrastructure):
├── Task 1: Create test-all.ps1 (PowerShell testing script)
├── Task 2: Create test-all.sh (Bash testing script)
├── Task 3: Setup GitHub Actions CI/CD workflow
└── Task 4: Setup frontend test infrastructure (Jest + RTL)

Wave 2 (After Wave 1 - Core Rust Modules, MAX PARALLEL):
├── Task 5: Implement fractal-gateway auth service
├── Task 6: Implement fractal-gateway eBPF loader
├── Task 7: Implement fractal-gateway IAM policies
├── Task 8: Add fractal-gateway integration tests
└── Task 9: Update fractal-gateway AGENTS.md

Wave 3 (After Wave 1 - Core Go Modules):
├── Task 10: Implement vector-kv FoundationDB client
├── Task 11: Implement vector-kv Weaviate client
├── Task 12: Implement vector-kv Redis cache
├── Task 13: Implement vector-kv hybrid query router
├── Task 14: Add vector-kv integration tests
└── Task 15: Update vector-kv AGENTS.md

Wave 4 (After Wave 2 - Chaos Tests):
├── Task 16: Implement resource_exhaustion scenario (CPU)
├── Task 17: Implement resource_exhaustion scenario (memory)
├── Task 18: Implement resource_exhaustion scenario (disk)
├── Task 19: Implement network_partition scenario (tc command)
└── Task 20: Add chaos tests integration verification

Wave 5 (After Wave 1,4 - Frontend Tests):
├── Task 21: Create DagViewer component tests
├── Task 22: Create page.tsx integration tests
├── Task 23: Add test utilities and mocks
└── Task 24: Verify frontend test coverage

Wave 6 (After ALL previous - Final Verification):
├── Task 25: Run complete test-all.ps1 verification
├── Task 26: Run complete test-all.sh verification
├── Task 27: Verify CI/CD pipeline passes
└── Task 28: Final documentation audit

Critical Path: Task 1-4 → Task 5-9,10-15,16-20,21-24 → Task 25-28
Parallel Speedup: ~75% faster than sequential
Max Concurrent: 4 waves (Waves 2-5 can overlap partially)
```

### Dependency Matrix
- **Task 1-4**: None (can start immediately)
- **Task 5-9**: Requires Task 1-4 (infrastructure ready)
- **Task 10-15**: Requires Task 1-4 (infrastructure ready)
- **Task 16-20**: Requires Task 1-4 (infrastructure ready)
- **Task 21-24**: Requires Task 4 (frontend test infra)
- **Task 25-28**: Requires ALL implementation tasks

### Agent Dispatch Summary
- **Wave 1**: 4 tasks → `quick` (script creation)
- **Wave 2**: 5 tasks → `deep` (complex auth logic)
- **Wave 3**: 6 tasks → `unspecified-high` (DB integration)
- **Wave 4**: 5 tasks → `deep` (system-level testing)
- **Wave 5**: 4 tasks → `visual-engineering` (frontend)
- **Wave 6**: 4 tasks → `unspecified-high` (verification)

---

## TODOs

### Wave 1: Infrastructure (Tasks 1-4)

- [ ] 1. Create test-all.ps1 PowerShell testing script

  **What to do**:
  - Create unified PowerShell script at root `test-all.ps1`
  - Test all Rust modules: `cargo test` for each Cargo.toml
  - Test all Go modules: `go test ./...` for each go.mod
  - Test frontend: `npm test` in web-dashboard
  - Run chaos tests: `cargo run --release -- --scenario all --dry-run`
  - Generate summary report with pass/fail counts
  - Exit code 0 on success, non-zero on failure
  
  **Must NOT do**:
  - Skip any module even if tests fail (report all)
  - Hardcode paths that won't work cross-platform
  - Require manual intervention
  
  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: None
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 25 (Final verification)
  - **Blocked By**: None
  
  **References**:
  - AGENTS.md test commands section
  - Existing Cargo.toml locations (13 files)
  - Existing go.mod locations (3 files)
  
  **Acceptance Criteria**:
  - [ ] `test-all.ps1` exists at project root
  - [ ] `./test-all.ps1` runs without errors on Windows
  - [ ] Script discovers all modules automatically
  - [ ] Summary report shows pass/fail per module
  - [ ] Exit code reflects overall success
  
  **QA Scenarios**:
  ```
  Scenario: Run PowerShell test script
  Tool: Bash (PowerShell)
  Preconditions: Windows environment, Rust/Go/Node installed
  Steps:
    1. Execute: ./test-all.ps1
    2. Wait for completion
    3. Check exit code: $LASTEXITCODE
  Expected Result: Exit code 0, summary shows test counts
  Evidence: .sisyphus/evidence/task-1-powershell-test.txt
  ```
  
  **Commit**: YES
  - Message: `feat: add unified PowerShell test script`
  - Files: `test-all.ps1`

- [ ] 2. Create test-all.sh Bash testing script

  **What to do**:
  - Create unified Bash script at root `test-all.sh`
  - Same functionality as test-all.ps1 but for Linux/macOS
  - Test all Rust, Go, and frontend modules
  - Generate summary report
  - Make script executable (`chmod +x`)
  
  **Must NOT do**:
  - Require PowerShell-specific features
  - Skip macOS compatibility
  
  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: None
  
  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 1)
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 26 (Final verification)
  - **Blocked By**: None
  
  **Acceptance Criteria**:
  - [ ] `test-all.sh` exists at project root
  - [ ] `./test-all.sh` runs without errors on Linux/macOS
  - [ ] Script is executable
  - [ ] Same output format as PowerShell version
  
  **QA Scenarios**:
  ```
  Scenario: Run Bash test script
  Tool: Bash
  Preconditions: Linux/macOS, Rust/Go/Node installed
  Steps:
    1. Execute: chmod +x test-all.sh && ./test-all.sh
    2. Wait for completion
    3. Check exit code: $?
  Expected Result: Exit code 0, summary shows test counts
  Evidence: .sisyphus/evidence/task-2-bash-test.txt
  ```
  
  **Commit**: YES
  - Message: `feat: add unified Bash test script`
  - Files: `test-all.sh`

- [ ] 3. Setup GitHub Actions CI/CD workflow

  **What to do**:
  - Create `.github/workflows/ci.yml`
  - Build matrix: Rust (1.75+), Go (1.25+), Node (20+)
  - Jobs: build, test, lint
  - Run on: push, pull_request
  - Cache dependencies (Cargo, Go modules, npm)
  - Upload artifacts on failure
  
  **Must NOT do**:
  - Skip any module in CI
  - Use outdated action versions
  - Skip linting
  
  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: None
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 27 (CI verification)
  - **Blocked By**: None
  
  **References**:
  - GitHub Actions documentation
  - Existing Cargo.toml and go.mod locations
  
  **Acceptance Criteria**:
  - [ ] `.github/workflows/ci.yml` exists
  - [ ] Workflow triggers on push/PR
  - [ ] All 3 language jobs run in parallel
  - [ ] Build passes for all modules
  - [ ] Tests pass for all modules
  
  **QA Scenarios**:
  ```
  Scenario: CI workflow validation
  Tool: GitHub Actions (verify yaml)
  Preconditions: Push to any branch
  Steps:
    1. Commit and push workflow file
    2. Open Actions tab on GitHub
    3. Verify jobs start and complete
  Expected Result: All jobs pass (green checkmarks)
  Evidence: Screenshot of GitHub Actions page
  ```
  
  **Commit**: YES
  - Message: `ci: add GitHub Actions workflow`
  - Files: `.github/workflows/ci.yml`

- [ ] 4. Setup frontend test infrastructure

  **What to do**:
  - Install Jest + React Testing Library in web-dashboard
  - Configure jest.config.js with Next.js support
  - Add test scripts to package.json
  - Create example test for DagViewer component
  - Setup coverage reporting
  
  **Must NOT do**:
  - Remove existing Next.js configuration
  - Use outdated testing libraries
  - Skip TypeScript support
  
  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: `frontend-ui-ux`
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 21-24 (Frontend tests)
  - **Blocked By**: None
  
  **References**:
  - `observability-ui/web-dashboard/package.json`
  - Next.js testing documentation
  - AGENTS.md web-dashboard conventions
  
  **Acceptance Criteria**:
  - [ ] `npm install --save-dev jest @testing-library/react @testing-library/jest-dom`
  - [ ] `jest.config.js` created with Next.js setup
  - [ ] `npm test` command works
  - [ ] Example test passes
  - [ ] Coverage reporting enabled
  
  **QA Scenarios**:
  ```
  Scenario: Run frontend tests
  Tool: Bash (npm)
  Preconditions: Node.js installed
  Steps:
    1. cd observability-ui/web-dashboard
    2. npm install
    3. npm test
  Expected Result: Tests pass, coverage report generated
  Evidence: .sisyphus/evidence/task-4-frontend-test.txt
  ```
  
  **Commit**: YES
  - Message: `test: setup Jest and RTL for web-dashboard`
  - Files: `observability-ui/web-dashboard/jest.config.js`, `package.json`

### Wave 2: fractal-gateway Implementation (Tasks 5-9)

- [ ] 5. Implement fractal-gateway auth service

  **What to do**:
  - Replace `add(2,2)` stub in `src/lib.rs` with actual authentication service
  - Create `AuthService` struct with methods: `authenticate`, `authorize`, `validate_token`
  - Implement JWT token validation
  - Add IAM policy enforcement
  - Create proper error types with `thiserror`
  - Follow existing `fractal-gateway/AGENTS.md` conventions
  
  **Must NOT do**:
  - Keep the `add()` stub function
  - Skip error handling with `unwrap()`
  - Hardcode credentials
  
  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: None
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 8 (Integration tests)
  - **Blocked By**: Task 1-4 (infrastructure)
  
  **References**:
  - `control-plane/fractal-gateway/src/lib.rs` (current stub)
  - `control-plane/fractal-gateway/AGENTS.md` (conventions)
  - `control-plane/fractal-gateway-ebpf/` (eBPF programs to load)
  - `execution-layer/sandbox-daemon/src/microvm.rs` (consumes gateway)
  
  **Acceptance Criteria**:
  - [ ] `AuthService` struct with auth methods
  - [ ] JWT token validation
  - [ ] IAM policy enforcement
  - [ ] Proper error handling
  - [ ] Unit tests for auth logic
  - [ ] `cargo build` passes
  
  **QA Scenarios**:
  ```
  Scenario: Authenticate with valid token
  Tool: Bash (cargo test)
  Preconditions: None
  Steps:
    1. cd control-plane/fractal-gateway
    2. cargo test auth::tests
  Expected Result: Tests pass
  Evidence: .sisyphus/evidence/task-5-auth-test.txt
  ```
  
  **Commit**: YES
  - Message: `feat(gateway): implement auth service with JWT`
  - Files: `control-plane/fractal-gateway/src/lib.rs`

- [ ] 6. Implement fractal-gateway eBPF loader

  **What to do**:
  - Create `EbpfLoader` struct to load programs from `fractal-gateway-ebpf`
  - Load XDP program for network filtering
  - Integrate with `aya` crate
  - Handle eBPF program attachment to interfaces
  - Add proper error handling for eBPF operations
  
  **Must NOT do**:
  - Skip eBPF error handling
  - Hardcode interface names
  - Bypass eBPF security
  
  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: None
  
  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 5)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 8
  - **Blocked By**: Task 1-4
  
  **References**:
  - `control-plane/fractal-gateway-ebpf/src/main.rs` (eBPF entry)
  - `control-plane/fractal-gateway/Cargo.toml` (dependencies)
  - `aya` crate documentation
  
  **Acceptance Criteria**:
  - [ ] `EbpfLoader` struct
  - [ ] XDP program loading
  - [ ] Interface attachment
  - [ ] Error handling
  - [ ] Tests (mocked if needed)
  
  **QA Scenarios**:
  ```
  Scenario: Load eBPF program
  Tool: Bash (cargo test -- --ignored if requires root)
  Steps:
    1. cargo test ebpf::tests
  Expected Result: Tests pass or properly skipped
  Evidence: .sisyphus/evidence/task-6-ebpf-test.txt
  ```
  
  **Commit**: YES
  - Message: `feat(gateway): implement eBPF loader`
  - Files: `control-plane/fractal-gateway/src/ebpf.rs`

- [ ] 7. Implement fractal-gateway IAM policies

  **What to do**:
  - Create `PolicyEngine` for RBAC (Role-Based Access Control)
  - Define policy structure (resource, action, effect)
  - Implement policy evaluation logic
  - Add policy storage (in-memory for now, Redis later)
  - Create default policies for SMA-OS services
  
  **Must NOT do**:
  - Skip authorization checks
  - Allow wildcard permissions by default
  - Skip audit logging
  
  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: None
  
  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 5-6)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 8
  - **Blocked By**: Task 1-4
  
  **References**:
  - AGENTS.md security conventions
  - AWS IAM policy structure (for reference)
  
  **Acceptance Criteria**:
  - [ ] `PolicyEngine` with evaluate method
  - [ ] Policy struct with resource/action/effect
  - [ ] RBAC implementation
  - [ ] Default policies for services
  - [ ] Unit tests
  
  **QA Scenarios**:
  ```
  Scenario: Evaluate policy for allowed action
  Tool: cargo test
  Steps:
    1. cargo test policy::tests
  Expected Result: Policy evaluation tests pass
  Evidence: .sisyphus/evidence/task-7-policy-test.txt
  ```
  
  **Commit**: YES
  - Message: `feat(gateway): implement IAM policy engine`
  - Files: `control-plane/fractal-gateway/src/policy.rs`

- [ ] 8. Add fractal-gateway integration tests

  **What to do**:
  - Create `tests/` directory
  - Add integration tests for auth flow
  - Test eBPF loading (mocked or with privilege check)
  - Test policy evaluation end-to-end
  - Test with actual sandbox-daemon integration
  
  **Must NOT do**:
  - Skip integration testing
  - Mock everything (test real components where possible)
  
  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: None
  
  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Task 5-7)
  - **Parallel Group**: Wave 2
  - **Blocks**: None
  - **Blocked By**: Task 5,6,7
  
  **Acceptance Criteria**:
  - [ ] Integration tests directory
  - [ ] Auth flow tests
  - [ ] eBPF integration tests
  - [ ] All tests pass
  
  **QA Scenarios**:
  ```
  Scenario: Full auth flow
  Tool: cargo test --test integration
  Steps:
    1. cargo test --test '*'
  Expected Result: All integration tests pass
  Evidence: .sisyphus/evidence/task-8-integration.txt
  ```
  
  **Commit**: YES
  - Message: `test(gateway): add integration tests`
  - Files: `control-plane/fractal-gateway/tests/integration.rs`

- [ ] 9. Update fractal-gateway AGENTS.md

  **What to do**:
  - Update AGENTS.md to reflect actual implementation
  - Add new sections for AuthService, EbpfLoader, PolicyEngine
  - Update "Where to Look" table
  - Remove stub references
  - Add usage examples
  
  **Must NOT do**:
  - Leave stub documentation
  - Skip error handling documentation
  
  **Recommended Agent Profile**:
  - **Category**: `writing`
  - **Skills**: None
  
  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Task 5-8)
  - **Blocks**: None
  - **Blocked By**: Task 5-8
  
  **Acceptance Criteria**:
  - [ ] AGENTS.md updated
  - [ ] All components documented
  - [ ] Usage examples added
  
  **QA Scenarios**:
  ```
  Scenario: Verify documentation
  Tool: Read AGENTS.md
  Steps:
    1. Read file
    2. Verify all components documented
  Expected Result: Complete documentation
  Evidence: .sisyphus/evidence/task-9-docs.txt
  ```
  
  **Commit**: YES
  - Message: `docs(gateway): update AGENTS.md with implementation`
  - Files: `control-plane/fractal-gateway/AGENTS.md`

---

### Wave 3: vector-kv Implementation (Tasks 10-15)

- [ ] 10. Implement vector-kv FoundationDB client

  **What to do**:
  - Add FoundationDB Go client dependency
  - Create `FDBClient` struct with connection management
  - Implement key-value operations: Get, Set, Delete, Range
  - Add transaction support
  - Handle connection errors
  
  **Must NOT do**:
  - Skip error handling
  - Leave connection unclosed
  
  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: None
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 14
  - **Blocked By**: Task 1-4
  
  **References**:
  - FoundationDB Go bindings documentation
  - `memory-bus/vector-kv/AGENTS.md`
  - Existing `main.go` structure
  
  **Acceptance Criteria**:
  - [ ] FDB client implementation
  - [ ] KV operations working
  - [ ] Connection management
  - [ ] Error handling
  
  **Commit**: YES
  - Message: `feat(vector-kv): add FoundationDB client`
  - Files: `memory-bus/vector-kv/fdb.go`

- [ ] 11. Implement vector-kv Weaviate client

  **What to do**:
  - Add Weaviate Go client dependency
  - Create `WeaviateClient` struct
  - Implement vector operations: Add, Search, Delete
  - Define schema for context embeddings
  - Handle HNSW indexing
  
  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  
  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 10)
  - **Parallel Group**: Wave 3
  
  **Acceptance Criteria**:
  - [ ] Weaviate client
  - [ ] Vector operations
  - [ ] Schema definition
  
  **Commit**: YES
  - Message: `feat(vector-kv): add Weaviate client`
  - Files: `memory-bus/vector-kv/weaviate.go`

- [ ] 12. Implement vector-kv Redis cache

  **What to do**:
  - Add go-redis dependency
  - Create `RedisCache` struct
  - Implement hot cache: Get, Set, Expire
  - Add TTL management
  - Connection pooling
  
  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  
  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 10-11)
  
  **Acceptance Criteria**:
  - [ ] Redis cache client
  - [ ] Hot cache operations
  - [ ] TTL support
  
  **Commit**: YES
  - Message: `feat(vector-kv): add Redis hot cache`
  - Files: `memory-bus/vector-kv/redis.go`

- [ ] 13. Implement vector-kv hybrid query router

  **What to do**:
  - Create query router that decides: Redis → Weaviate → FoundationDB
  - Implement `ReadWithCache` with actual DB calls
  - Replace mock responses
  - Add latency metrics
  - Fallback chain logic
  
  **Must NOT do**:
  - Keep mock responses
  - Skip metrics
  
  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  
  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Task 10-12)
  - **Blocked By**: Task 10,11,12
  
  **Acceptance Criteria**:
  - [ ] Query router implementation
  - [ ] Actual DB calls (no mocks)
  - [ ] Latency <1ms target
  - [ ] Fallback logic
  
  **Commit**: YES
  - Message: `feat(vector-kv): implement hybrid query router`
  - Files: `memory-bus/vector-kv/router.go`, `main.go`

- [ ] 14. Add vector-kv integration tests

  **What to do**:
  - Create integration tests
  - Test against real databases (or testcontainers)
  - Verify latency targets
  - Test fallback scenarios
  
  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  
  **Parallelization**:
  - **Blocked By**: Task 10-13
  
  **Acceptance Criteria**:
  - [ ] Integration tests
  - [ ] Latency verification
  - [ ] Fallback tests
  
  **Commit**: YES
  - Message: `test(vector-kv): add integration tests`
  - Files: `memory-bus/vector-kv/*_test.go`

- [ ] 15. Update vector-kv AGENTS.md

  **What to do**:
  - Document actual implementation
  - Update "Where to Look"
  - Add connection configuration
  - Update architecture description
  
  **Recommended Agent Profile**:
  - **Category**: `writing`
  
  **Parallelization**:
  - **Blocked By**: Task 10-14
  
  **Commit**: YES
  - Message: `docs(vector-kv): update AGENTS.md`
  - Files: `memory-bus/vector-kv/AGENTS.md`

---

### Wave 4: Chaos Tests Implementation (Tasks 16-20)

- [ ] 16. Implement resource_exhaustion scenario (CPU)

  **What to do**:
  - Implement CPU exhaustion logic in `resource_exhaustion.rs`
  - Spawn processes that consume CPU
  - Monitor system CPU usage
  - Verify system remains responsive
  - Implement recovery check
  
  **Must NOT do**:
  - Skip recovery verification
  - Leave system unrecoverable
  
  **Recommended Agent Profile**:
  - **Category**: `deep`
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4
  
  **Acceptance Criteria**:
  - [ ] CPU exhaustion logic
  - [ ] System monitoring
  - [ ] Recovery verification
  
  **Commit**: YES
  - Message: `feat(chaos): implement CPU exhaustion scenario`
  - Files: `chaos-tests/src/scenarios/resource_exhaustion.rs`

- [ ] 17. Implement resource_exhaustion scenario (memory)

  **What to do**:
  - Add memory allocation that causes pressure
  - Monitor memory usage
  - Test OOM handling
  - Verify recovery
  
  **Recommended Agent Profile**:
  - **Category**: `deep`
  
  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 16)
  
  **Commit**: YES
  - Message: `feat(chaos): add memory exhaustion`
  - Files: `chaos-tests/src/scenarios/resource_exhaustion.rs`

- [ ] 18. Implement resource_exhaustion scenario (disk)

  **What to do**:
  - Add disk space consumption
  - Create large temporary files
  - Monitor disk usage
  - Cleanup after test
  
  **Recommended Agent Profile**:
  - **Category**: `deep`
  
  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 16-17)
  
  **Commit**: YES
  - Message: `feat(chaos): add disk exhaustion`
  - Files: `chaos-tests/src/scenarios/resource_exhaustion.rs`

- [ ] 19. Implement network_partition scenario

  **What to do**:
  - Use `tc` (traffic control) command for network manipulation
  - Implement latency injection
  - Implement packet loss
  - Implement full partition (isolation)
  - Test split-brain prevention
  - Cleanup network rules after test
  
  **Must NOT do**:
  - Leave network rules after test
  - Skip cleanup
  
  **Recommended Agent Profile**:
  - **Category**: `deep`
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4
  
  **Acceptance Criteria**:
  - [ ] tc command integration
  - [ ] Latency injection
  - [ ] Partition simulation
  - [ ] Cleanup logic
  
  **Commit**: YES
  - Message: `feat(chaos): implement network partition scenario`
  - Files: `chaos-tests/src/scenarios/network_partition.rs`

- [ ] 20. Add chaos tests integration verification

  **What to do**:
  - Run all scenarios: `cargo run --release -- --scenario all`
  - Verify all pass
  - Verify cleanup happens
  - Update README with results
  
  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  
  **Parallelization**:
  - **Blocked By**: Task 16-19
  
  **Acceptance Criteria**:
  - [ ] All scenarios pass
  - [ ] Cleanup verified
  - [ ] Documentation updated
  
  **Commit**: YES
  - Message: `test(chaos): verify all scenarios pass`
  - Files: `chaos-tests/README.md`

---

### Wave 5: Frontend Tests (Tasks 21-24)

- [ ] 21. Create DagViewer component tests

  **What to do**:
  - Test DagViewer component rendering
  - Test node interactions
  - Test edge rendering
  - Mock ReactFlow for unit tests
  
  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: `frontend-ui-ux`
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 5
  - **Blocked By**: Task 4
  
  **Acceptance Criteria**:
  - [ ] Component renders without errors
  - [ ] Node interactions work
  - [ ] Tests pass
  
  **Commit**: YES
  - Message: `test(ui): add DagViewer component tests`
  - Files: `observability-ui/web-dashboard/src/components/DagViewer.test.tsx`

- [ ] 22. Create page.tsx integration tests

  **What to do**:
  - Test full page rendering
  - Test data flow
  - Test WebSocket/mock connections
  - Test error states
  
  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: `frontend-ui-ux`
  
  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 21)
  
  **Acceptance Criteria**:
  - [ ] Page renders
  - [ ] Data flow tested
  - [ ] Error states handled
  
  **Commit**: YES
  - Message: `test(ui): add page integration tests`
  - Files: `observability-ui/web-dashboard/app/page.test.tsx`

- [ ] 23. Add test utilities and mocks

  **What to do**:
  - Create test utilities (render helpers, mock providers)
  - Mock external dependencies
  - Setup test fixtures
  - Create custom matchers if needed
  
  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  
  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 21-22)
  
  **Commit**: YES
  - Message: `test(ui): add test utilities and mocks`
  - Files: `observability-ui/web-dashboard/src/test-utils/`

- [ ] 24. Verify frontend test coverage

  **What to do**:
  - Run `npm test -- --coverage`
  - Verify >80% coverage
  - Add missing tests if below target
  - Document coverage in README
  
  **Must NOT do**:
  - Accept <80% coverage
  - Skip coverage reporting
  
  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  
  **Parallelization**:
  - **Blocked By**: Task 21-23
  
  **Acceptance Criteria**:
  - [ ] Coverage >80%
  - [ ] All components tested
  - [ ] Coverage report generated
  
  **Commit**: YES
  - Message: `test(ui): verify 80%+ test coverage`
  - Files: `observability-ui/web-dashboard/README.md`

---

### Wave 6: Final Verification (Tasks 25-28)

- [ ] 25. Run complete test-all.ps1 verification

  **What to do**:
  - Execute `test-all.ps1` on Windows
  - Verify all modules tested
  - Verify summary report
  - Fix any failures
  
  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  
  **Parallelization**:
  - **Blocked By**: Task 1,5-24
  
  **Acceptance Criteria**:
  - [ ] Script runs successfully
  - [ ] All modules pass
  - [ ] Summary correct
  
  **QA Scenarios**:
  ```
  Scenario: Full PowerShell test run
  Tool: PowerShell
  Steps:
    1. ./test-all.ps1
    2. Check exit code
  Expected Result: Exit code 0, all pass
  Evidence: .sisyphus/evidence/task-25-full-ps.txt
  ```
  
  **Commit**: NO (verification only)

- [ ] 26. Run complete test-all.sh verification

  **What to do**:
  - Execute `test-all.sh` on Linux/macOS
  - Verify cross-platform compatibility
  - Fix any platform-specific issues
  
  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  
  **Parallelization**:
  - **Blocked By**: Task 2,5-24
  
  **Acceptance Criteria**:
  - [ ] Script runs on Linux/macOS
  - [ ] All modules pass
  
  **QA Scenarios**:
  ```
  Scenario: Full Bash test run
  Tool: Bash
  Steps:
    1. ./test-all.sh
    2. Check exit code
  Expected Result: Exit code 0
  Evidence: .sisyphus/evidence/task-26-full-bash.txt
  ```
  
  **Commit**: NO

- [ ] 27. Verify CI/CD pipeline passes

  **What to do**:
  - Push to GitHub
  - Verify Actions workflow triggers
  - Verify all jobs pass
  - Verify artifacts uploaded
  - Check build times are reasonable
  
  **Must NOT do**:
  - Merge with failing CI
  - Skip CI verification
  
  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  
  **Parallelization**:
  - **Blocked By**: Task 3,5-24
  
  **Acceptance Criteria**:
  - [ ] Workflow triggers
  - [ ] All jobs pass
  - [ ] Build times <10 min
  
  **QA Scenarios**:
  ```
  Scenario: CI pipeline validation
  Tool: GitHub Actions
  Steps:
    1. Push branch
    2. Open Actions tab
    3. Verify all green
  Expected Result: All jobs pass
  Evidence: Screenshot or CI logs
  ```
  
  **Commit**: NO

- [ ] 28. Final documentation audit

  **What to do**:
  - Review all AGENTS.md files
  - Remove references to non-existent scripts
  - Verify all documentation matches implementation
  - Update root README with build badges
  - Verify all TODOs addressed
  
  **Must NOT do**:
  - Leave stale documentation
  - Skip root README update
  
  **Recommended Agent Profile**:
  - **Category**: `writing`
  
  **Parallelization**:
  - **Blocked By**: ALL previous tasks
  
  **Acceptance Criteria**:
  - [ ] All AGENTS.md reviewed
  - [ ] Stale references removed
  - [ ] README updated with badges
  - [ ] All TODOs resolved
  
  **Commit**: YES
  - Message: `docs: final documentation audit and cleanup`
  - Files: `README.md`, various AGENTS.md

---

## Final Verification Wave (MANDATORY)

- [ ] F1. Plan Compliance Audit — `oracle`
  - Read entire plan
  - Verify all 28 tasks have implementation
  - Check evidence files exist
  - Compare deliverables against plan
  - Output: VERDICT

- [ ] F2. Code Quality Review — `unspecified-high`
  - Run `cargo clippy --all-targets -- -D warnings`
  - Run `golangci-lint run ./...`
  - Run `npm run lint` in web-dashboard
  - Check for `unwrap()`, `panic!`, commented code
  - Output: Quality report

- [ ] F3. Integration Tests — `unspecified-high`
  - Run full test-all.ps1
  - Run full test-all.sh
  - Run chaos tests: `cargo run --release -- --scenario all`
  - Verify CI/CD passes
  - Output: Integration test report

- [ ] F4. Documentation Review — `deep`
  - Verify all AGENTS.md updated
  - Verify README has badges
  - Verify no stale references
  - Verify all TODOs resolved
  - Output: Documentation review

---

## Commit Strategy

- **Wave 1 (Tasks 1-4)**: Infrastructure commits, can be squashed
- **Wave 2 (Tasks 5-9)**: fractal-gateway feature commits
- **Wave 3 (Tasks 10-15)**: vector-kv feature commits
- **Wave 4 (Tasks 16-20)**: chaos-tests feature commits
- **Wave 5 (Tasks 21-24)**: Frontend test commits
- **Wave 6 (Tasks 25-28)**: Verification commits, documentation

**Commit message format**:
```
type(scope): description

- type: feat, fix, test, docs, ci, refactor
- scope: module name
- description: imperative mood, lowercase
```

---

## Success Criteria

### Verification Commands
```bash
# Rust build
cd control-plane && cargo build --release
cd execution-layer && cargo build --release

# Go build
cd memory-bus && go build ./...
cd orchestration && go build ./...

# Frontend build
cd observability-ui/web-dashboard && npm run build

# Tests
./test-all.ps1  # Windows
./test-all.sh   # Linux/macOS

# CI/CD
git push && verify GitHub Actions pass
```

### Final Checklist
- [ ] All 28 tasks complete
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All tests pass
- [ ] CI/CD green
- [ ] Documentation accurate
- [ ] No TODOs remaining
- [ ] Code quality passes linting

---

## Agent Summary

**Total Tasks**: 28
**Waves**: 6 + Final Verification
**Categories Used**:
- `quick`: Tasks 1-4 (infrastructure scripts)
- `deep`: Tasks 5-9 (fractal-gateway), 16-20 (chaos tests)
- `unspecified-high`: Tasks 10-15 (vector-kv), 25-27 (verification)
- `visual-engineering`: Tasks 21-24 (frontend tests)
- `writing`: Tasks 9, 15, 28 (documentation)

**Skills Used**:
- `frontend-ui-ux`: Tasks 4, 21-24 (frontend)

**Estimated Timeline**:
- Wave 1: 1-2 days
- Wave 2: 3-4 days (complex auth logic)
- Wave 3: 3-4 days (DB integration)
- Wave 4: 2-3 days (chaos scenarios)
- Wave 5: 2 days (frontend tests)
- Wave 6: 1 day (verification)

**Total**: ~12-16 days with parallel execution
