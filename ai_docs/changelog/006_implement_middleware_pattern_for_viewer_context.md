# Implement Middleware Pattern for ViewerContext Creation

## 1. Task Overview

### Task Title
**Title:** Implement Middleware Pattern for ViewerContext Creation and Clean Architecture Separation

### Goal Statement
**Goal:** Implement Meta's authentic architecture where business logic handlers only receive ViewerContext while infrastructure concerns (AppState) are handled by middleware. This separates infrastructure from business logic, making the code cleaner and more maintainable while following Meta's actual patterns.

### Success Criteria
- [ ] Middleware extracts ViewerContext from AppState + request data
- [ ] Business logic handlers only receive ViewerContext (never AppState)
- [ ] Clean separation between infrastructure layer and business logic
- [ ] All web handlers use the new middleware pattern
- [ ] AppState remains for infrastructure dependencies only
- [ ] Authentication/authorization logic integrated into middleware
- [ ] Code compiles and web server works correctly

---

## 2. Project Analysis and Current State

### Technology Stack
- **Language:** Rust 1.70+
- **Framework:** Axum web framework with TAO Database System
- **Middleware:** Axum middleware for request processing
- **Authentication:** ViewerContext-based with capabilities
- **Build System:** Cargo

### Architecture Overview
- **Design Pattern:** Middleware pattern with clean architecture separation
- **Current Issue:** Business logic handlers access AppState directly (mixing concerns)
- **Target Pattern:** Middleware creates ViewerContext, handlers only see ViewerContext
- **Key Components:** Middleware, AppState, ViewerContext, Web Handlers

### Current State Assessment

#### File Structure
```
src/
├── bin/
│   └── tao_web_server.rs        # Web handlers with AppState access
├── infrastructure/
│   └── viewer/
│       └── viewer.rs            # ViewerContext with TAO field
```

#### Current Problem
```rust
// ❌ Current: Business logic sees infrastructure
async fn create_user(State(state): State<AppState>, ...) {
    let viewer_context = ViewerContext::system(id, state.tao.clone());
    let user = EntUser::create(viewer_context)...
}
```

#### Target Solution
```rust
// ✅ Target: Business logic only sees ViewerContext
async fn create_user(
    Extension(vc): Extension<Arc<ViewerContext>>,
    Json(request): Json<CreateUserRequest>,
) -> impl IntoResponse {
    let user = EntUser::create(vc)...
}
```

---

## 3. Context and Problem Definition

### Problem Statement
**What is broken or missing?**
Current web handlers mix infrastructure concerns (AppState) with business logic. Business logic should only interact with ViewerContext, while middleware should handle the translation from infrastructure to application context.

### Root Cause Analysis
**Why does this problem exist?**
- Direct coupling between web handlers and infrastructure layer
- No middleware layer to handle context creation
- Business logic handlers know about AppState structure
- Missing clean architecture separation

### Impact Assessment
- **User Impact:** No direct impact (implementation detail)
- **Developer Impact:** Harder to test, mixed concerns, not following Meta's pattern
- **System Impact:** Tighter coupling, harder to maintain and evolve

---

## 4. Technical Requirements and Constraints

### Functional Requirements
1. Middleware must create ViewerContext from AppState + request data
2. All web handlers must receive ViewerContext only (no AppState access)
3. Authentication/authorization logic integrated into middleware
4. Support different viewer types (authenticated user, anonymous, system)
5. Error handling for authentication failures
6. Maintain all existing API functionality

### Non-Functional Requirements
- **Performance:** No significant performance impact from middleware
- **Reliability:** Robust error handling for auth failures
- **Maintainability:** Clean separation of concerns
- **Security:** Secure authentication and authorization

### Technical Constraints
- **Must preserve:** All existing API functionality and behavior
- **Must follow:** Meta's authentic pattern where business logic only sees ViewerContext
- **Cannot change:** AppState structure (still needed for infrastructure)

---

## 5. Solution Design

### Approach Overview
**High-level strategy:**
Create Axum middleware that extracts authentication data from requests, combines with AppState infrastructure, creates appropriate ViewerContext, and injects it into request extensions. Update all handlers to use ViewerContext extension instead of AppState.

### Architecture Changes

#### New Components
- **ViewerContextMiddleware:** Creates ViewerContext from AppState + request
- **Authentication Extraction:** Extract auth info from headers/tokens
- **Context Factory:** Create different viewer types based on request

#### Modified Components
- **Web Handlers:** Remove AppState dependency, use ViewerContext extension
- **AppState:** Remains unchanged (infrastructure dependencies)
- **Error Handling:** Add authentication error responses

### Data Flow Design
```
Request → Middleware → Extract Auth → Create ViewerContext → Handler(VC) → Response
          ↓
       AppState (TAO, configs)
```

---

## 6. Implementation Plan

### Phase Breakdown
1. **Phase 1:** Create middleware infrastructure
   - Tasks: Create middleware function, authentication extraction
   - Dependencies: None
   - Validation: Middleware compiles and can extract auth data

2. **Phase 2:** Implement ViewerContext creation logic
   - Tasks: Create factory functions for different viewer types
   - Dependencies: Phase 1 complete
   - Validation: ViewerContext created correctly for different scenarios

3. **Phase 3:** Update web handlers
   - Tasks: Modify all handlers to use ViewerContext extension
   - Dependencies: Phase 2 complete
   - Validation: All handlers compile with new pattern

4. **Phase 4:** Integration and testing
   - Tasks: Wire middleware into app, test all endpoints
   - Dependencies: Phase 3 complete
   - Validation: Web server works correctly with new pattern

### File-by-File Changes

#### File: `src/infrastructure/middleware/mod.rs` (New)
- **Change Type:** New file
- **Purpose:** Middleware for ViewerContext creation
- **Key Changes:**
  - Create middleware function
  - Authentication extraction logic
  - ViewerContext factory functions
  - Error handling for auth failures
- **Dependencies:** ViewerContext, AppState
- **Testing Strategy:** Unit tests for auth extraction and context creation

#### File: `src/bin/tao_web_server.rs`
- **Change Type:** Modification
- **Purpose:** Update all handlers to use ViewerContext extension
- **Key Changes:**
  - Remove State<AppState> from handler signatures
  - Add Extension<Arc<ViewerContext>> to handlers
  - Remove ViewerContext creation logic from handlers
  - Add middleware to app router
- **Dependencies:** Updated middleware
- **Testing Strategy:** Test all API endpoints work correctly

### Risk Mitigation
- **Risk:** Breaking existing API functionality
  - **Mitigation:** Thorough testing of all endpoints
- **Risk:** Performance impact from middleware
  - **Mitigation:** Efficient implementation, measure performance

---

## 7. Testing Strategy

### Unit Testing
- **New Tests Required:** Test middleware authentication extraction and context creation
- **Modified Tests:** Update integration tests for new handler signatures
- **Coverage Goals:** 100% coverage of middleware logic

### Integration Testing
- **Component Integration:** Test middleware + handlers work together
- **API Testing:** Test all endpoints with various authentication scenarios
- **Error Testing:** Test authentication failures and error responses

### Manual Testing
- **Test Scenarios:** Create user, get user, list users with different auth types
- **Edge Cases:** Invalid auth tokens, missing auth headers
- **Performance Testing:** Measure middleware overhead

---

## 8. Implementation Log

> **CRITICAL**: Fill this section as you implement to maintain context for future AI agents

### Changes Made

#### [Date] - [Component/File Modified]
**Status:** PENDING - Ready to begin implementation

### Decisions Made

### Issues Encountered

### Performance Impact

---

## 9. Validation and Verification

### Acceptance Testing
- [ ] Middleware creates ViewerContext correctly - PENDING
- [ ] Handlers only receive ViewerContext - PENDING
- [ ] All API endpoints work correctly - PENDING
- [ ] Authentication integrated properly - PENDING
- [ ] Error handling works for auth failures - PENDING

### Code Quality Checks
- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Code follows project conventions
- [ ] Documentation is updated

### Integration Verification
- [ ] Web server starts correctly
- [ ] All endpoints respond correctly
- [ ] Authentication works as expected

---

## Template Completion Checklist

- [x] All sections are completed with specific, actionable information
- [x] Technical decisions are well-documented with reasoning
- [x] File-level changes are clearly specified
- [ ] Implementation log is filled out as work progresses
- [x] All acceptance criteria are defined and testable

---

**Status:** PENDING - Ready to begin implementation following this plan