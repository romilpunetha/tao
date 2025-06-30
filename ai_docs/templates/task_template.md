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
<!-- Provide a clear and specific title for this task -->
**Title:** [Brief descriptive title of what you're building/fixing]

### Goal Statement
<!-- One paragraph describing high-level objective -->
**Goal:** [Clear statement of what you want to achieve and why it matters]

### Success Criteria
<!-- Define what "done" looks like -->
- [ ] [Specific measurable outcome 1]
- [ ] [Specific measurable outcome 2]
- [ ] [Specific measurable outcome 3]

---

## 2. Project Analysis and Current State

### Technology Stack
- **Language:** [e.g., Rust 1.70+]
- **Framework:** [e.g., TAO Database System]
- **Database:** [e.g., PostgreSQL with multi-shard support]
- **Serialization:** [e.g., Apache Thrift]
- **Build System:** [e.g., Cargo]
- **Key Dependencies:** [List major crates/libraries]

### Architecture Overview
- **Design Pattern:** [e.g., Entity-Builder pattern, TAO object-association model]
- **Code Generation:** [e.g., Schema-driven with JSON definitions]
- **Data Flow:** [Brief description of how data moves through the system]
- **Key Components:** [List main modules/components]

### Current State Assessment
**Before starting this task, document the current state:**

#### File Structure
```
[Paste relevant directory structure showing files that will be affected]
```

#### Existing Functionality
- [What currently works that's related to this task]
- [Any limitations or issues with current implementation]
- [Dependencies that exist]

#### Related Code Patterns
- [Document existing patterns that should be followed]
- [Any conventions or standards to maintain]

---

## 3. Context and Problem Definition

### Problem Statement
**What is broken or missing?**
[Detailed description of the current problem or missing functionality]

### Root Cause Analysis
**Why does this problem exist?**
- [Technical reason 1]
- [Business/design reason 2]
- [Historical context if relevant]

### Impact Assessment
- **User Impact:** [How does this affect end users]
- **Developer Impact:** [How does this affect development workflow]
- **System Impact:** [Performance, scalability, maintainability effects]

---

## 4. Technical Requirements and Constraints

### Functional Requirements
1. [Specific behavior requirement 1]
2. [Specific behavior requirement 2]
3. [Integration requirement with existing systems]

### Non-Functional Requirements
- **Performance:** [Response time, throughput requirements]
- **Reliability:** [Error handling, failover requirements]
- **Maintainability:** [Code quality, documentation standards]
- **Security:** [Authentication, authorization, data protection]

### Technical Constraints
- **Must preserve:** [Existing functionality that cannot break]
- **Must follow:** [Existing patterns, conventions, interfaces]
- **Cannot change:** [Legacy systems, external dependencies]

---

## 5. Solution Design

### Approach Overview
**High-level strategy:**
[1-2 paragraphs describing your overall approach]

### Architecture Changes
**System-level modifications:**

#### New Components
- **Component Name:** [Purpose and responsibility]
- **Location:** [File path where it will live]
- **Dependencies:** [What it depends on]
- **Interface:** [How other components interact with it]

#### Modified Components
- **Component:** [Name and path]
- **Changes:** [What will be modified and why]
- **Impact:** [How this affects other components]

#### Database Changes
- **Schema Changes:** [New tables, columns, indexes]
- **Migration Strategy:** [How to handle existing data]
- **Performance Impact:** [Query patterns, indexing considerations]

### Data Flow Design
```
[ASCII diagram or description of how data flows through the new/modified system]
User Input → [Component A] → [Component B] → Database
```

---

## 6. Implementation Plan

### Phase Breakdown
1. **Phase 1:** [Description]
   - Tasks: [List specific tasks]
   - Dependencies: [What must be done first]
   - Validation: [How to verify this phase is complete]

2. **Phase 2:** [Description]
   - Tasks: [List specific tasks]
   - Dependencies: [What must be done first]
   - Validation: [How to verify this phase is complete]

### File-by-File Changes
**For each file that will be created or modified:**

#### File: `path/to/file.rs`
- **Change Type:** [New file | Modification | Deletion]
- **Purpose:** [Why this file is being changed]
- **Key Changes:**
  - [Specific change 1]
  - [Specific change 2]
- **Dependencies:** [Other files this depends on]
- **Testing Strategy:** [How to verify changes work]

### Risk Mitigation
- **Risk:** [Potential issue 1]
  - **Mitigation:** [How to prevent or handle it]
- **Risk:** [Potential issue 2]
  - **Mitigation:** [How to prevent or handle it]

---

## 7. Testing Strategy

### Unit Testing
- **New Tests Required:** [List test files/functions to create]
- **Modified Tests:** [Existing tests that need updates]
- **Coverage Goals:** [What percentage of new code should be tested]

### Integration Testing
- **Component Integration:** [How to test components work together]
- **Database Testing:** [Migration and data integrity testing]
- **API Testing:** [Endpoint testing if applicable]

### Manual Testing
- **Test Scenarios:** [User workflows to verify manually]
- **Edge Cases:** [Boundary conditions to test]
- **Performance Testing:** [Load/stress testing if needed]

---

## 8. Implementation Log

> **CRITICAL**: Fill this section as you implement to maintain context for future AI agents

### Changes Made

#### [Date] - [Component/File Modified]
**Change Description:** [What was changed]
**Reason:** [Why this change was necessary]
**Files Modified:**
- `path/to/file1.rs`: [Specific changes made]
- `path/to/file2.rs`: [Specific changes made]

**Code Patterns Established/Modified:**
- [Pattern 1]: [Description and reasoning]
- [Pattern 2]: [Description and reasoning]

**Database Changes:**
- [Schema change]: [DDL and reasoning]
- [Data migration]: [Steps and validation]

**Breaking Changes:**
- [Change]: [Impact and migration path]

**Testing Results:**
- [Test type]: [Results and any issues found]

### Decisions Made
**Key technical decisions and their rationale:**

1. **Decision:** [What was decided]
   - **Alternatives Considered:** [Other options that were evaluated]
   - **Reasoning:** [Why this option was chosen]
   - **Trade-offs:** [What was gained/lost with this decision]

2. **Decision:** [What was decided]
   - **Alternatives Considered:** [Other options that were evaluated]
   - **Reasoning:** [Why this option was chosen]
   - **Trade-offs:** [What was gained/lost with this decision]

### Issues Encountered
**Problems discovered during implementation:**

1. **Issue:** [Description of problem]
   - **Root Cause:** [Why it happened]
   - **Solution:** [How it was resolved]
   - **Prevention:** [How to avoid in future]

### Performance Impact
- **Before:** [Baseline metrics if measured]
- **After:** [Performance metrics after changes]
- **Analysis:** [Performance implications and any optimizations made]

---

## 9. Validation and Verification

### Acceptance Testing
- [ ] [Acceptance criteria 1] - [Status/Result]
- [ ] [Acceptance criteria 2] - [Status/Result]
- [ ] [Acceptance criteria 3] - [Status/Result]

### Code Quality Checks
- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Code follows project conventions
- [ ] Documentation is updated
- [ ] Security review completed (if applicable)

### Integration Verification
- [ ] [Integration point 1] verified working
- [ ] [Integration point 2] verified working
- [ ] No regression in existing functionality

---

## 10. Documentation and Knowledge Transfer

### Code Documentation
- **Inline Comments:** [Standards followed for complex logic]
- **API Documentation:** [Public interfaces documented]
- **Architecture Documentation:** [High-level design docs updated]

### Usage Examples
```rust
// Example of how to use the new functionality
let example = NewComponent::create()
    .with_property(value)
    .build();
```

### Migration Guide
**For other developers/AI agents:**
1. [Step 1 to adopt changes]
2. [Step 2 to migrate existing code]
3. [Step 3 to verify migration worked]

### Future Considerations
- **Tech Debt:** [Any shortcuts taken that should be addressed later]
- **Scaling Considerations:** [How this solution will handle growth]
- **Potential Improvements:** [Ideas for future enhancement]

---

## 11. Handoff Information

### For Future AI Agents
**Critical context for maintaining this system:**

#### Patterns to Maintain
- [Pattern 1]: [Description and why it's important]
- [Pattern 2]: [Description and why it's important]

#### Common Pitfalls
- [Pitfall 1]: [How to avoid]
- [Pitfall 2]: [How to avoid]

#### Extension Points
- [Area 1]: [How to add new functionality here]
- [Area 2]: [How to add new functionality here]

### Related Systems
- **System A**: [How this task affects it]
- **System B**: [Dependencies or integration points]

### Monitoring and Maintenance
- **Metrics to Watch:** [Performance indicators]
- **Log Messages:** [What to look for in logs]
- **Troubleshooting:** [Common issues and solutions]

---

## Template Completion Checklist

Before submitting this document, ensure:

- [ ] All sections are completed with specific, actionable information
- [ ] Technical decisions are well-documented with reasoning
- [ ] File-level changes are clearly specified
- [ ] Database/schema changes are documented with migration paths
- [ ] Testing strategy covers all critical paths
- [ ] Performance implications are considered and documented
- [ ] Future maintainers have sufficient context to continue work
- [ ] Implementation log is filled out as work progresses
- [ ] All acceptance criteria are defined and testable

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