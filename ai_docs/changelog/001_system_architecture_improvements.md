# AI Task Template

> **⚠️ CRITICAL INSTRUCTIONS FOR AI AGENTS ⚠️**
> 
> **DO NOT MODIFY THIS TEMPLATE FILE!** This is a read-only template.
> 
> **USAGE INSTRUCTIONS:**
> 1. **READ ONLY**: Use this template as a reference - never edit this file
> 2. **CREATE CHANGELOG**: Copy this template to create new changelog documents in `ai_docs/changelog/` 
> 3. **NAMING CONVENTION**: Use incremental naming like SQL migrations: `001_task_name.md`, `002_next_task.md`, etc.
> 4. **FILL COMPLETELY**: Every AI agent must fill this template completely when making changes to maintain system context and enable reproducibility
> 
> **Example Usage:**
> ```
> cp ai_docs/templates/task_template.md ai_docs/changelog/001_add_builder_save_method.md
> # Then fill out the copied file with your specific task details
> ```

> **Instructions:** This template helps you create comprehensive task documents for AI-driven development. Fill out each section thoroughly to ensure the AI agent has all necessary context and can execute tasks systematically.

---

## 1. Task Overview

### Task Title
**Title:** TAO System Architecture Improvements and Design Optimization

### Goal Statement
**Goal:** Analyze the current TAO codebase and implement comprehensive system design improvements to enhance code structure, eliminate architectural anti-patterns, improve type safety, and create a more maintainable and extensible codebase while preserving existing functionality.

### Success Criteria
- [ ] Unified builder pattern eliminating duplication between EntBuilder trait and concrete builders
- [ ] Enhanced type safety with strong typing using newtype patterns
- [ ] Cleaner module organization with proper separation of concerns
- [ ] Updated documentation reflecting new architecture
- [ ] All existing functionality preserved and tests passing

---

## 2. Project Analysis and Current State

### Technology Stack
- **Language:** Rust 1.70+
- **Framework:** TAO Database System (inspired by Facebook's TAO)
- **Database:** PostgreSQL with multi-shard support
- **Serialization:** Apache Thrift
- **Build System:** Cargo
- **Key Dependencies:** sqlx, tokio, axum, thrift, serde, async-trait

### Architecture Overview
- **Design Pattern:** Entity-Builder pattern with TAO object-association model
- **Code Generation:** Schema-driven with JSON definitions
- **Data Flow:** Builder → TAO Core → Database with caching layer
- **Key Components:** TAO Core, Entity Framework, Code Generation System, Domain Entities

### Current State Assessment
**Before starting this task, document the current state:**

#### File Structure
```
src/
├── ent_framework/          # Entity framework with mixed concerns
├── infrastructure/         # Infrastructure with global state pattern
├── domains/                # Generated entity domains
├── codegen/                # Code generation at root level
└── bin/                    # Application binaries
```

#### Existing Functionality
- 6 entity types: EntUser, EntPost, EntComment, EntGroup, EntPage, EntEvent
- Dual builder system: concrete builders + EntBuilder trait
- Global TAO state management
- Code generation system for entities and builders
- Multi-shard PostgreSQL backend with caching

#### Related Code Patterns
- Builder pattern with fluent interface
- Trait-based entity definitions
- Async TAO operations with connection pooling
- Thrift serialization for data persistence

---

## 3. Context and Problem Definition

### Problem Statement
**What is broken or missing?**
The current TAO system has several architectural issues that impact maintainability and elegance:

1. **Dual Builder Anti-pattern**: Both concrete builders and EntBuilder trait implement the same `build()` logic, causing code duplication
2. **Global State**: Global TAO instance creates tight coupling and testing difficulties
3. **Weak Typing**: Primitive type aliases (TaoId = i64) provide no type safety
4. **Mixed Concerns**: Infrastructure modules have unclear responsibilities
5. **Scattered Code Generation**: Code generators mixed with business logic

### Root Cause Analysis
**Why does this problem exist?**
- **Historical Development**: System grew organically without architectural refactoring
- **Pattern Inconsistency**: Different patterns emerged for similar functionality over time
- **Tight Coupling**: Components depend on concrete implementations rather than abstractions

### Impact Assessment
- **Developer Impact**: Code duplication makes maintenance difficult and error-prone
- **System Impact**: Global state makes testing and modularity challenging
- **Maintainability Impact**: Weak typing leads to runtime errors that could be caught at compile time

---

## 4. Technical Requirements and Constraints

### Functional Requirements
1. Maintain all existing entity CRUD operations
2. Preserve TAO's create() method flow: generate ID → build entity → save
3. Keep existing API contracts for entity builders
4. Maintain Thrift serialization compatibility

### Non-Functional Requirements
- **Performance:** No performance regression in entity operations
- **Maintainability:** Cleaner, more modular code structure
- **Type Safety:** Enhanced compile-time error detection

### Technical Constraints
- **Must preserve:** All existing public APIs and entity generation patterns
- **Must follow:** TAO architectural principles (ID generation, caching, etc.)
- **Cannot change:** Database schema or Thrift serialization format

---

## 5. Solution Design

### Approach Overview
**High-level strategy:**
Implement a phased refactoring approach that eliminates architectural anti-patterns while preserving functionality. Focus on unifying the builder pattern, enhancing type safety, and improving module organization. Use dependency injection to eliminate global state and strong typing to prevent runtime errors.

### Architecture Changes
**System-level modifications:**

#### New Components
- **Core Types Module** (`src/core/types.rs`): Strong typed wrappers for EntityId, EntityType, etc.
- **Unified Builder System**: Single EntBuilder trait eliminating concrete builder build() methods
- **Context-based TAO Access**: Replace global state with dependency injection

#### Modified Components
- **EntBuilder Trait**: Becomes the single source of truth for entity building
- **Concrete Builders**: Remove duplicate build() methods, keep only fluent setters
- **TAO Core**: Enhanced to work with unified builder pattern
- **Code Generators**: Generate only EntBuilder trait implementations

#### Database Changes
- **Schema Changes:** None required
- **Migration Strategy:** No data migration needed
- **Performance Impact:** No impact on query patterns

### Data Flow Design
```
User Code → EntityBuilder.fluent_methods() → TAO.create(builder) 
         → TAO.generate_id() → EntBuilder.build(id) → Storage
```

---

## 6. Implementation Plan

### Phase Breakdown
1. **Phase 1: Type System Enhancement**
   - Tasks: Create strong typed wrappers, update core modules
   - Dependencies: None
   - Validation: Compilation succeeds with new types

2. **Phase 2: Unified Builder Pattern**
   - Tasks: Remove duplicate build() methods, update code generation
   - Dependencies: Phase 1 complete
   - Validation: All builders use only EntBuilder trait

3. **Phase 3: Module Reorganization**
   - Tasks: Restructure modules according to new organization
   - Dependencies: Phase 2 complete
   - Validation: Clear separation of concerns achieved

### File-by-File Changes
**For each file that will be created or modified:**

#### File: `src/core/types.rs`
- **Change Type:** New file
- **Purpose:** Define strong typed wrappers for better type safety
- **Key Changes:**
  - EntityId(i64) newtype wrapper
  - EntityType(String) newtype wrapper
  - AssociationType(String) newtype wrapper
- **Dependencies:** None
- **Testing Strategy:** Unit tests for type conversions and serialization

#### File: `src/domains/*/builder.rs`
- **Change Type:** Modification
- **Purpose:** Remove duplicate build() method from concrete builders
- **Key Changes:**
  - Remove pub fn build() method
  - Keep only EntBuilder trait implementation
  - Remove savex() duplication
- **Dependencies:** EntBuilder trait
- **Testing Strategy:** Verify TAO create() flow still works

#### File: `src/codegen/builder_generator.rs`
- **Change Type:** Modification  
- **Purpose:** Generate only EntBuilder implementations
- **Key Changes:**
  - Remove generation of concrete build() methods
  - Focus on EntBuilder trait implementation
  - Simplify generated code
- **Dependencies:** EntBuilder trait definition
- **Testing Strategy:** Generated code compiles and works correctly

### Risk Mitigation
- **Risk:** Breaking existing API contracts
  - **Mitigation:** Maintain all public method signatures during refactoring
- **Risk:** Performance regression
  - **Mitigation:** Benchmark before/after to ensure no performance loss

---

## 7. Testing Strategy

### Unit Testing
- **New Tests Required:** Strong type conversion tests, unified builder tests
- **Modified Tests:** Update tests using old builder patterns
- **Coverage Goals:** Maintain current test coverage while improving reliability

### Integration Testing
- **Component Integration:** Test TAO create() flow with new unified builders
- **Database Testing:** Verify entity persistence works with new types
- **API Testing:** Ensure web server endpoints continue working

### Manual Testing
- **Test Scenarios:** Create entities through web API, verify database storage
- **Edge Cases:** Test with missing required fields, invalid data
- **Performance Testing:** Benchmark entity creation performance

---

## 8. Implementation Log

> **CRITICAL**: Fill this section as you implement to maintain context for future AI agents

### Changes Made

#### [2024-12-XX] - Architecture Analysis and Planning
**Change Description:** Analyzed current TAO system and documented comprehensive improvement plan
**Reason:** Need to eliminate architectural anti-patterns and improve maintainability
**Files Modified:**
- `gemini.md`: Updated with new architecture principles and proposed structure
- `ai_docs/changelog/001_system_architecture_improvements.md`: Created comprehensive task documentation

**Code Patterns Established/Modified:**
- Documented unified builder pattern approach
- Planned strong typing system with newtype patterns
- Outlined dependency injection approach

**Database Changes:**
- None required for this phase

**Breaking Changes:**
- None yet - planning phase only

**Testing Results:**
- Analysis phase - no tests run yet

### Decisions Made
**Key technical decisions and their rationale:**

1. **Decision:** Keep EntBuilder trait, remove concrete build() methods
   - **Alternatives Considered:** Remove EntBuilder trait, keep concrete methods
   - **Reasoning:** EntBuilder trait enables generic TAO operations and eliminates duplication
   - **Trade-offs:** Slight additional trait complexity for significant code reduction

2. **Decision:** Use newtype pattern for strong typing
   - **Alternatives Considered:** Keep primitive aliases, use generics
   - **Reasoning:** Newtype provides type safety with zero runtime cost
   - **Trade-offs:** More verbose type definitions for compile-time safety

### Issues Encountered
**Problems discovered during implementation:**

1. **Issue:** Dual builder pattern creating maintenance burden
   - **Root Cause:** Historical growth without architectural review
   - **Solution:** Unify around EntBuilder trait pattern
   - **Prevention:** Document architectural principles clearly

### Performance Impact
- **Before:** Not measured yet
- **After:** Will benchmark after implementation
- **Analysis:** No expected performance impact from architectural changes

---

## 9. Validation and Verification

### Acceptance Testing
- [ ] All entity builders use unified EntBuilder pattern - Pending
- [ ] Strong typing system implemented - Pending  
- [ ] Module organization improved - Pending
- [ ] Documentation updated - In Progress
- [ ] All existing tests pass - Pending

### Code Quality Checks
- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Code follows project conventions
- [ ] Documentation is updated
- [ ] Architecture review completed

### Integration Verification
- [ ] TAO create() flow works with unified builders
- [ ] Entity persistence functions correctly
- [ ] No regression in existing functionality

---

## 10. Documentation and Knowledge Transfer

### Code Documentation
- **Inline Comments:** Document architectural decisions in complex modules
- **API Documentation:** Update public interface documentation
- **Architecture Documentation:** Enhanced gemini.md with new design principles

### Usage Examples
```rust
// New unified builder usage (no change for users)
let user = EntUser::create()
    .username("john".to_string())
    .email("john@example.com".to_string())
    .savex().await?;  // This uses TAO.create(builder) internally

// Strong typing usage
let entity_id = EntityId::from(123i64);
let entity_type = EntityType::from("ent_user".to_string());
```

### Migration Guide
**For other developers/AI agents:**
1. Use only EntBuilder trait for building entities
2. Leverage strong types for better type safety
3. Follow new module organization patterns
4. Use dependency injection instead of global state

### Future Considerations
- **Tech Debt:** Plan to eliminate global TAO state in favor of context passing
- **Scaling Considerations:** New architecture better supports testing and modularity
- **Potential Improvements:** Consider using const generics for entity types

---

## 11. Handoff Information

### For Future AI Agents
**Critical context for maintaining this system:**

#### Patterns to Maintain
- **Unified Builder Pattern**: Only EntBuilder trait should contain build() logic
- **Strong Typing**: Use newtype patterns instead of primitive aliases
- **Dependency Injection**: Avoid global state, pass context explicitly

#### Common Pitfalls
- **Builder Duplication**: Don't add build() methods to concrete builders
- **Global State**: Avoid using global TAO instance in new code
- **Weak Typing**: Don't use primitive aliases for domain types

#### Extension Points
- **New Entity Types**: Follow EntBuilder pattern exclusively
- **Storage Backends**: Implement StorageBackend trait for new databases
- **Cache Layers**: Implement CacheInterface for new caching strategies

### Related Systems
- **Code Generation**: Affects how builders are generated
- **TAO Core**: Central to entity creation and storage
- **Entity Framework**: Foundation for all domain entities

### Monitoring and Maintenance
- **Metrics to Watch:** Entity creation performance, compilation times
- **Log Messages:** Look for builder pattern usage in logs
- **Troubleshooting:** Check EntBuilder implementations for build() logic

---

## Template Completion Checklist

Before submitting this document, ensure:

- [x] All sections are completed with specific, actionable information
- [x] Technical decisions are well-documented with reasoning
- [x] File-level changes are clearly specified
- [ ] Database/schema changes are documented with migration paths
- [x] Testing strategy covers all critical paths
- [x] Performance implications are considered and documented
- [x] Future maintainers have sufficient context to continue work
- [ ] Implementation log is filled out as work progresses
- [x] All acceptance criteria are defined and testable

---

## IMPORTANT REMINDERS FOR AI AGENTS

### Template Usage Protocol
1. **NEVER EDIT THIS TEMPLATE** - This file should remain unchanged
2. **COPY TO CHANGELOG** - Always create a new file in `ai_docs/changelog/` using incremental naming
3. **COMPLETE ALL SECTIONS** - Fill out every section thoroughly for future AI agents
4. **DOCUMENT REASONING** - Explain why decisions were made, not just what was implemented

### Changelog File Naming Examples
- `001_initial_builder_system.md` - First major feature
- `002_add_save_method_to_builders.md` - Enhancement to builders
- `003_fix_database_connection_pooling.md` - Bug fix
- `004_implement_caching_layer.md` - New infrastructure

### Context Preservation Goals
This template ensures that:
- Any AI agent can understand the current system state
- Changes are fully documented with reasoning
- Future modifications can build on previous work
- The system remains maintainable and extensible

> **Note for AI Agents**: This template is designed to capture not just what you're building, but why and how you're building it. The goal is reproducibility and context preservation. Another AI agent should be able to read this document and either continue your work or reproduce it from scratch. Be thorough, be specific, and document your reasoning at every step.

> **🔄 WORKFLOW REMINDER**: Copy this template → Rename with incremental number → Fill completely → Document all changes