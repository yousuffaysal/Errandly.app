# PRODUCT REQUIREMENTS DOCUMENT (PRD)
# ACTIONDESK AI
### A Local-First, Agentic AI Desktop Assistant

**Version:** 2.0 — Startup Edition  
**Company:** Foxmen Studio  
**Product Name:** ActionDesk AI (Temporary Name)  
**Founder:** Foxmen Studio  
**Initial Platform:** macOS (Apple Silicon)  
**Future Platforms:** Windows and Linux  
**Product Category:** AI Desktop Productivity & Automation  
**Business Model:** Free Core + Optional Premium Features  
**AI Architecture:** Local AI + Agentic Automation + Optional External AI  
**Data Architecture:** 100% User-Owned Local Storage by Default  
**Cloud Storage Provider:** None Required  
**Mandatory Backend Server:** None  
**Document Date:** October 8, 2026  
**Status:** Product Requirements — Development Planning

---

# 1. Executive Summary

## 1.1 Product Overview

ActionDesk AI is an intelligent, agentic desktop application that allows users to automate everyday computer tasks using natural-language instructions.

Unlike conventional AI chatbots, which primarily generate text responses, ActionDesk AI will perform actual tasks on a user's computer.

The application will combine:

- Local large language models (LLMs).
- Agentic task planning.
- Local file system automation.
- Document intelligence.
- Spreadsheet processing.
- Business workflow automation.
- Academic productivity tools.
- Natural-language interaction.
- Privacy-focused, offline-first architecture.

**The entire core application will run on the user's own computer, utilizing their own processing power, memory, and storage.**

ActionDesk AI will not require Foxmen Studio to maintain expensive cloud storage infrastructure, host user documents, or pay for AI inference on behalf of users.

The objective is to develop an accessible desktop AI assistant that delivers practical automation without requiring users to purchase expensive AI subscriptions.

### Core Product Statement

> **ActionDesk AI is your personal AI employee. It runs on your computer, uses your own storage, and completes your work without requiring a paid AI subscription.**

## 1.2 Example User Experience

A business owner opens ActionDesk AI and enters:

"Analyze all invoices in my October folder, calculate my total expenses, organize the invoices by category, and generate an Excel report."

ActionDesk AI will:

1. Understand the instruction using its local AI model.
2. Request permission to access the selected folder.
3. Identify the invoice files.
4. Extract invoice information.
5. Categorize expenses.
6. Calculate totals using local processing.
7. Create an Excel report.
8. Present proposed file changes for approval.
9. Organize the approved files.
10. Save all outputs on the user's computer.

**Nothing needs to be uploaded to Foxmen Studio servers.**

The application processes the files locally and saves the completed results directly to the user's Mac.

---

# 2. Fundamental Product Principles

These principles are mandatory and must guide every product and engineering decision.

## 2.1 Zero Founder-Hosted Storage

**Requirement ID: CORE-001**

Foxmen Studio will not provide cloud storage for user documents, conversations, AI models, or generated files.

All user-generated and application-specific data must be stored locally.

| Data Category | Storage Location | Foxmen Studio Storage |
|---|---|---|
| User documents | User's Mac | None |
| AI models | User's Mac | None |
| AI conversations | Local SQLite database | None |
| Task history | Local SQLite database | None |
| Generated reports | User-selected local folder | None |
| PDF files | User-selected local folder | None |
| Spreadsheet files | User-selected local folder | None |
| AI embeddings | Local application storage | None |
| Vector indexes | Local application storage | None |
| Workflow configurations | Local SQLite database | None |
| Temporary processing files | Local cache | None |
| User preferences | Local application settings | None |
| API credentials | macOS Keychain | None |

**Acceptance Criteria:**

- Core features operate without connecting to Foxmen Studio servers.
- User data remains on the user's device by default.
- Documents are never automatically uploaded to developer-owned storage.
- Users can manage their own application storage.
- The application remains functional if Foxmen Studio's website becomes unavailable.

## 2.2 Zero Mandatory Cloud Infrastructure

**Requirement ID: CORE-002**

ActionDesk AI must not depend on a continuously running backend server for ordinary use.

The core application should operate as:

```text
             USER'S MAC
                 |
                 v
       +--------------------+
       |    ActionDesk AI   |
       |   Desktop Software |
       +--------------------+
                 |
          +------+------+
          |             |
          v             v
      Local AI      Agent Engine
          |             |
          +------+------+
                 |
                 v
         Local Computer
                 |
       +---------+---------+
       |         |         |
       v         v         v
     Files    Documents  Spreadsheets
       |         |         |
       +---------+---------+
                 |
                 v
          Local Storage
                 |
                 v
        Completed Results
```

No mandatory backend is needed for:

- AI inference.
- File processing.
- Document analysis.
- Spreadsheet calculations.
- Task execution.
- Report generation.
- Local workflow management.
- Local task history.

Optional online services, such as model downloads, software updates, payments, and third-party AI integrations, must remain independent of the core application.

## 2.3 No Mandatory AI Subscription

**Requirement ID: CORE-003**

Users must be able to perform core AI tasks without purchasing OpenAI, Anthropic, Grok, or other cloud AI API credits.

ActionDesk AI will use locally executed models as the default intelligence engine.

Users may optionally connect third-party AI providers if they want additional capabilities.

However, the core product must never depend on an external API to function.

## 2.4 User Ownership and Control

**Requirement ID: CORE-004**

Users must maintain control over:

- Their files.
- Their AI models.
- Their documents.
- Their workspaces.
- Their task history.
- Their application data.
- Their generated outputs.
- Their API credentials.

Users must be able to export or delete their application data.

Uninstalling ActionDesk AI must not automatically delete unrelated personal files or generated business documents.

## 2.5 Offline-First Operation

**Requirement ID: CORE-005**

After installation and local model setup, users must be able to perform supported core operations without internet access.

The application must support:

- Offline AI conversations.
- Offline document processing.
- Offline file organization.
- Offline spreadsheet analysis.
- Offline report generation.
- Offline task execution.

Internet connectivity will only be necessary for features that inherently require external access.

---

# 3. Business Objectives

## 3.1 Primary Business Objective

Build a useful AI-powered desktop application with minimal recurring operational expenses.

The application should take advantage of the computing resources users already own.

### Startup Infrastructure Strategy

| Resource | Provider | Who Bears the Resource Cost? |
|---|---|---|
| AI processing | User's CPU/GPU | User |
| AI model storage | User's SSD | User |
| Documents | User's local storage | User |
| Generated files | User's local storage | User |
| Database | Local SQLite | User |
| Document indexing | Local computer | User |
| File automation | Local operating system | User |
| Optional Grok API | User's own API account | User |
| Optional cloud backups | User's selected provider | User |
| Application distribution | Release hosting / download provider | Foxmen Studio, if applicable |
| Developer website | Static hosting | Foxmen Studio, if applicable |

This reduces the need for recurring storage, database hosting, and centralized inference expenses.

**Important:** Zero hosted user storage does not mean zero business expenses. Software signing, distribution, support, development, domain registration, and optional payment infrastructure may still have costs.

## 3.2 Product Objectives

1. Deliver reliable agentic task execution.
2. Provide AI capabilities without mandatory API payments.
3. Minimize operating costs for Foxmen Studio.
4. Protect user privacy through local processing.
5. Simplify repetitive computer tasks.
6. Support students, entrepreneurs, and business owners.
7. Maintain a modular architecture for future expansion.
8. Generate revenue through software features rather than mandatory cloud storage.

---

# 4. Target Users

## 4.1 Students and Researchers

### Primary Problems

- Large collections of academic documents.
- Difficulty organizing lecture materials.
- Time-consuming summarization.
- Repetitive assignment preparation.
- Difficulty finding information across PDFs.

### Core Features

- Lecture PDF summarization.
- Academic file organization.
- Research paper analysis.
- Question generation.
- Study note creation.
- Document comparison.
- Topic extraction.

### Example Instruction

"Read these five lecture PDFs, prepare topic-wise notes, and create 30 practice questions."

### Expected Output

```text
Documents/
    Statistics/
        Lecture_1.pdf
        Lecture_2.pdf
        Lecture_3.pdf

Generated/
    Topic_Wise_Notes.pdf
    Practice_Questions.pdf
    Study_Summary.md
```

All outputs remain on the student's computer.

---

## 4.2 Business Owners

### Primary Problems

- Manual invoice processing.
- Disorganized financial documents.
- Repetitive expense calculations.
- Time-consuming spreadsheet management.
- Limited access to affordable automation.

### Core Features

- Invoice information extraction.
- Expense categorization.
- Sales analysis.
- Automated reporting.
- Business document organization.
- Spreadsheet processing.
- Profit and expense summaries.

### Example Instruction

"Analyze my September sales records and create a monthly business performance report."

### Expected Output

```text
Business/
    September/
        Sales.xlsx
        Expenses.xlsx

Reports/
    September_Summary.xlsx
    September_Report.pdf
```

No financial records are stored on ActionDesk AI's developer servers.

---

## 4.3 Entrepreneurs and Freelancers

### Primary Problems

- Managing multiple projects.
- Preparing proposals.
- Organizing client documents.
- Tracking project information.
- Repetitive administrative work.

### Core Features

- Proposal generation.
- Client brief analysis.
- Project folder organization.
- Contract comparison.
- Requirements document generation.
- Project summary creation.
- Reusable task workflows.

### Example Instruction

"Read this client brief and generate a structured software development proposal."

The application generates an editable proposal in the selected local output folder.

---

## 4.4 General Professionals

Additional potential users include:

- Accountants.
- Teachers.
- Administrative professionals.
- Office workers.
- Consultants.
- Designers.
- Researchers.
- Project managers.

These users can benefit from document intelligence and repetitive workflow automation.

---

# 5. Product Scope

Development will follow three levels of priority.

**P0 — Minimum Viable Product:** Features required for the first functional release.

**P1 — Version 1 Expansion:** Features developed after validating the MVP.

**P2 — Future Development:** Advanced functionality beyond the initial release.

## 5.1 Feature Priority Matrix

| Feature | Priority |
|---|---|
| Installable macOS application | P0 |
| Local AI model management | P0 |
| Offline AI inference | P0 |
| Natural-language command interface | P0 |
| Agentic task planning | P0 |
| Local file access | P0 |
| File organization | P0 |
| Document extraction | P0 |
| PDF summarization | P0 |
| Spreadsheet analysis | P0 |
| Report generation | P0 |
| Local task history | P0 |
| Local workspace management | P0 |
| User-selected output directories | P0 |
| Storage usage monitoring | P0 |
| Permission and approval system | P0 |
| Undo for supported file operations | P0 |
| Execution verification | P0 |
| Offline functionality | P0 |
| Optional Grok integration | P1 |
| Local semantic search | P1 |
| Reusable workflows | P1 |
| Bengali-language support | P1 |
| Advanced OCR | P1 |
| Local workflow scheduling | P1 |
| External drive support | P1 |
| Voice commands | P2 |
| Browser automation | P2 |
| General GUI computer control | P2 |
| Email integrations | P2 |
| Calendar integrations | P2 |
| Multi-agent orchestration | P2 |
| Windows application | P2 |
| Third-party plugin marketplace | P2 |

## 5.2 Explicit MVP Exclusions

The initial release will not include:

- Cloud document storage.
- Mandatory user accounts.
- Cloud-hosted vector databases.
- Centralized conversation storage.
- Hosted AI inference.
- Multi-user cloud workspaces.
- Cloud document synchronization.
- Autonomous online purchases.
- Unrestricted terminal command execution.
- Unrestricted control of every Mac application.
- Automatic email sending.
- Permanent file deletion.

These exclusions protect the startup's budget and reduce technical and security complexity.

---

# 6. Technical Architecture

## 6.1 Architecture Overview

ActionDesk AI will be a standalone desktop application.

Its architecture will contain six principal components:

1. Desktop user interface.
2. Local AI inference engine.
3. Agent orchestration engine.
4. Local tool execution engine.
5. Local data management system.
6. Optional external AI integration.

### High-Level Architecture

```text
┌──────────────────────────────────────────────┐
│                 USER'S MAC                   │
│                                              │
│  ┌────────────────────────────────────────┐  │
│  │            ACTIONDESK AI               │  │
│  │                                        │  │
│  │        Desktop User Interface          │  │
│  │         React + TypeScript             │  │
│  └───────────────────┬────────────────────┘  │
│                      │                       │
│                      ▼                       │
│  ┌────────────────────────────────────────┐  │
│  │            AGENT ENGINE                │  │
│  │                                        │  │
│  │  • Understand Instructions             │  │
│  │  • Generate Execution Plans            │  │
│  │  • Select Tools                        │  │
│  │  • Validate Actions                    │  │
│  │  • Execute Tasks                       │  │
│  │  • Verify Results                      │  │
│  └───────────────────┬────────────────────┘  │
│                      │                       │
│            ┌─────────┴─────────┐             │
│            │                   │             │
│            ▼                   ▼             │
│  ┌─────────────────┐ ┌───────────────────┐   │
│  │    LOCAL AI     │ │    LOCAL TOOLS    │   │
│  │                 │ │                   │   │
│  │ • Local LLM     │ │ • File Manager    │   │
│  │ • Embeddings    │ │ • PDF Processor   │   │
│  │ • Classification│ │ • Excel Processor │   │
│  │ • Summarization │ │ • Report Generator│   │
│  └────────┬────────┘ └─────────┬─────────┘   │
│           │                    │             │
│           └─────────┬──────────┘             │
│                     │                        │
│                     ▼                        │
│  ┌────────────────────────────────────────┐  │
│  │          LOCAL DATA LAYER              │  │
│  │                                        │  │
│  │ • SQLite Database                      │  │
│  │ • AI Model Files                       │  │
│  │ • User Documents                       │  │
│  │ • Generated Reports                    │  │
│  │ • Local Search Indexes                 │  │
│  │ • Task History                         │  │
│  └────────────────────────────────────────┘  │
│                                              │
└──────────────────────────────────────────────┘

          OPTIONAL CONNECTION ONLY
                     │
                     ▼
          ┌─────────────────────┐
          │  External AI API    │
          │                     │
          │  Grok / Other APIs  │
          │                     │
          │  User's API Key     │
          └─────────────────────┘
```

### Mandatory Architectural Rule

The local application must contain everything required to execute its core features.

An unavailable Foxmen Studio website or backend must not prevent users from accessing their existing workspaces and completing local tasks.

---

# 7. Local Storage Architecture

**Priority: P0 — Critical**

This is one of the most important technical requirements in the entire PRD.

## 7.1 Storage Philosophy

ActionDesk AI must use the computer's existing storage instead of purchasing centralized storage infrastructure.

The software will not maintain copies of user documents in Foxmen Studio's cloud.

Users are responsible for providing sufficient local storage.

### Storage Responsibilities

| Storage Component | Responsibility |
|---|---|
| Original user documents | Remain in existing user folders |
| AI model files | Downloaded to user-controlled local storage |
| Database | Stored within application support directory |
| Temporary processing files | Stored in local application cache |
| Search indexes | Stored locally |
| Agent logs | Stored locally |
| Generated reports | Stored in user-selected output location |
| Backups | User-controlled |
| Cloud synchronization | Not provided by ActionDesk |

## 7.2 Default Directory Structure

On macOS, the application should use appropriate operating-system directories.

Proposed structure:

```text
~/Library/
    Application Support/
        ActionDesk/
            database/
                actiondesk.sqlite

            models/
                model-1.gguf
                model-2.gguf

            indexes/
                documents/

            workspaces/
                workspace-metadata/

            logs/
                execution/

            settings/
                config.json

    Caches/
        ActionDesk/
            extraction/
            previews/
            temporary/
```

Generated files should use a separately selected directory.

For example:

```text
~/Documents/
    ActionDesk/
        Reports/
        Summaries/
        Spreadsheets/
        Exports/
```

The displayed product name and the actual filesystem directory identifiers should be configurable independently so the application can be rebranded without requiring users to move data manually.

## 7.3 Avoid Unnecessary File Duplication

**Requirement ID: STORAGE-001**

ActionDesk must not automatically duplicate every document added to a workspace.

For example, when a user adds:

```text
~/Documents/Research/Paper.pdf
```

The application should store a secure reference to the file, relevant metadata, and any explicitly enabled local index.

It should not automatically make another complete copy in application storage.

### Benefits

- Reduced storage consumption.
- Faster workspace creation.
- Less unnecessary data duplication.
- Simpler file management.

## 7.4 Storage Location Selection

**Requirement ID: STORAGE-002**

Users must be able to choose where generated files are saved.

The application should allow:

- Default output directory.
- Workspace-specific output directory.
- Custom model directory.
- Export directory.
- Local external drive where supported.

For the MVP, custom output folders are mandatory.

Relocating model storage to external drives may be delivered in V1.

## 7.5 Storage Manager

**Requirement ID: STORAGE-003**

The application must include a storage management screen.

### Example Interface

```text
ActionDesk Storage

Application Data           85 MB
Downloaded AI Models      4.2 GB
Document Indexes          240 MB
Temporary Cache           125 MB
Task History               18 MB

Total Managed Storage      4.7 GB

Available Disk Space      68.5 GB

[Manage Models]
[Clear Cache]
[Delete Old History]
[Change Output Folder]
```

The actual values will be measured from the user's device.

### Required Features

- Display local storage usage.
- Identify large model files.
- Remove unused models.
- Clear temporary files.
- Delete selected task histories.
- Manage document indexes.
- Display available disk space.
- Warn before large downloads.

## 7.6 Storage Quotas

**Requirement ID: STORAGE-004**

The application must prevent uncontrolled storage growth.

Users should be able to configure:

| Resource | Proposed Control |
|---|---|
| Temporary cache | Maximum cache size |
| Task logs | Retention period |
| Document indexes | Enable/disable per workspace |
| AI models | Manual model removal |
| Generated documents | User-managed output folders |
| Temporary extraction data | Automatic cleanup |

The application should avoid automatically deleting generated user documents.

Only disposable caches and explicitly selected application data may be removed automatically.

## 7.7 Insufficient Storage Handling

**Requirement ID: STORAGE-005**

Before downloading models or producing large files, ActionDesk must check available disk space.

If storage is insufficient, the application should:

1. Stop the operation safely.
2. Explain the storage requirement.
3. Show available space.
4. Offer cache cleanup where appropriate.
5. Allow the user to select another location.
6. Preserve existing data.

### Acceptance Criteria

- No unnecessary duplication of uploaded documents.
- Local storage can be inspected.
- Cache cleanup works.
- Users can remove downloaded models.
- Disk-full conditions do not corrupt the database.
- Generated files are never silently removed.

---

# 8. Local AI Engine

**Priority: P0**

## 8.1 Overview

ActionDesk AI will run an AI model directly on the user's computer.

The local AI model is responsible for interpreting instructions and helping construct task plans.

The local inference engine must not require a developer-hosted server.

## 8.2 Inference Options

Two practical approaches should be evaluated.

### Option A: Embedded AI Runtime

Use an inference runtime integrated into the desktop application, such as a compatible `llama.cpp` implementation.

Advantages:

- No separate AI application installation.
- More consistent user experience.
- Greater control over model compatibility.
- Entirely local inference.
- No mandatory AI API key.

Disadvantages:

- Additional engineering work.
- Runtime packaging complexity.
- Hardware compatibility testing.
- Model updates and licensing responsibilities.

### Option B: Local Ollama Integration

Connect ActionDesk AI to Ollama running on the user's own Mac.

Advantages:

- Faster initial development.
- Existing model-management tools.
- Local API access.
- Reduced inference-engine development.

Disadvantages:

- Users may need to install Ollama separately.
- Additional setup complexity.
- Less control over the complete experience.

### Recommended Approach

For the earliest technical prototype, support Ollama.

For the consumer MVP, prioritize an integrated local inference experience so nontechnical users can install ActionDesk and download a model without separately configuring an AI runtime.

The final runtime choice must be based on performance and packaging tests.

## 8.3 Model Downloading

**Requirement ID: AI-001**

AI models should be downloaded directly to the user's computer from an authorized model distribution source.

Foxmen Studio should not host large AI model files unless there is a compelling reason.

### Proposed Download Flow

```text
User Opens ActionDesk
          |
          v
Detect Mac Hardware
          |
          v
Recommend Model
          |
          v
User Selects Model
          |
          v
Download from Model Provider
          |
          v
Save to User's Local Storage
          |
          v
Verify File Integrity
          |
          v
Initialize Model
          |
          v
Ready for Offline Tasks
```

The application must use properly licensed models and comply with the distribution provider's terms.

Direct downloads should use verified sources, pinned model versions, and integrity checks.

## 8.4 Model Selection

Models must be selected according to:

- Tool-calling accuracy.
- Instruction-following ability.
- Structured output reliability.
- Reasoning quality.
- Memory consumption.
- Apple Silicon performance.
- Commercial licensing.
- Multilingual support.

Potential model families can include commercially usable variants from the Qwen, Gemma, or other open-weight ecosystems.

Specific versions should be chosen after testing their licenses and performance.

## 8.5 Hardware-Aware Models

The application must detect available resources.

### Initial Recommendations

| Mac Configuration | Recommended Strategy |
|---|---|
| 8 GB unified memory | Lightweight quantized models |
| 16 GB unified memory | Balanced local models |
| 24–32 GB unified memory | Larger models and contexts |
| 64 GB+ unified memory | Advanced local inference |

These are deployment categories, not guarantees of model performance.

The application's AI settings should explain when a model is unsuitable for the user's hardware.

## 8.6 Model Storage

AI model files will be saved locally.

Approximate quantized model sizes vary by architecture and quantization level.

A lightweight model might require approximately 1–3 GB, while a larger model might require 4–8 GB or more.

Actual download sizes must be displayed before installation.

### Mandatory Requirements

- Download progress.
- Resume interrupted downloads.
- Verify downloaded files.
- Model removal.
- Model switching.
- Storage usage display.
- Offline model loading.
- Hardware compatibility warnings.

---

# 9. Optional Grok AI Integration

**Priority: P1**

## 9.1 Purpose

Grok will provide an optional alternative for tasks where users prefer externally hosted AI.

However, the product must not assume that Grok API usage is free.

The xAI API is subject to provider pricing and account conditions.

## 9.2 Integration Model

ActionDesk will implement **Bring Your Own API Key (BYOK)**.

### User Experience

1. User opens Settings.
2. Selects AI Providers.
3. Chooses Grok.
4. Enters their API key.
5. Authorizes use of the external service.
6. Chooses which tasks may use cloud inference.

API keys must be stored using macOS Keychain.

## 9.3 AI Routing

| Operation | Default |
|---|---|
| Simple natural-language tasks | Local AI |
| Document summarization | Local AI |
| File classification | Local AI |
| Spreadsheet interpretation | Local AI |
| Agent planning | Local AI |
| Optional advanced reasoning | User-selected Grok |
| File execution | Local tools |
| Numerical calculations | Local deterministic code |

**Cloud AI must never activate silently.**

## 9.4 Data Privacy

When external AI is enabled, the application must explain what information will be transmitted.

Users must approve sending document contents or sensitive data to external services.

ActionDesk must never send documents to Foxmen Studio as an intermediate storage or processing service.

Where technically supported, requests should travel directly from the user's device to the selected provider.

---

# 10. Agentic AI Engine

**Priority: P0**

## 10.1 Purpose

The Agent Engine transforms user instructions into executable computer tasks.

It is the central component of ActionDesk AI.

## 10.2 Agent Workflow

```text
USER INSTRUCTION
       |
       v
INTENT UNDERSTANDING
       |
       v
TASK CLASSIFICATION
       |
       v
TASK PLANNING
       |
       v
TOOL SELECTION
       |
       v
PERMISSION VALIDATION
       |
       v
USER APPROVAL
       |
       v
LOCAL EXECUTION
       |
       v
RESULT VERIFICATION
       |
       v
COMPLETED OUTPUT
```

## 10.3 Functional Requirements

| ID | Requirement | Priority |
|---|---|---|
| AGENT-001 | Interpret natural-language requests | P0 |
| AGENT-002 | Identify supported task categories | P0 |
| AGENT-003 | Generate structured plans | P0 |
| AGENT-004 | Select authorized tools | P0 |
| AGENT-005 | Validate tool arguments | P0 |
| AGENT-006 | Display plans before execution | P0 |
| AGENT-007 | Execute approved operations | P0 |
| AGENT-008 | Track task progress | P0 |
| AGENT-009 | Handle failures safely | P0 |
| AGENT-010 | Verify completed actions | P0 |
| AGENT-011 | Support task cancellation | P0 |
| AGENT-012 | Save task history locally | P0 |
| AGENT-013 | Support reusable workflows | P1 |
| AGENT-014 | Support scheduled workflows | P1 |
| AGENT-015 | Support multiple specialized agents | P2 |

## 10.4 Agent Execution Policy

The local LLM must not have unrestricted access to the computer.

The system must use a controlled tool registry.

For every tool call:

1. Validate the tool identifier.
2. Validate the input schema.
3. Check user permissions.
4. Check risk classification.
5. Request approval if necessary.
6. Execute through a trusted local service.
7. Verify the result.
8. Record the execution.

### Agent States

```typescript
type TaskStatus =
  | "created"
  | "planning"
  | "awaiting_approval"
  | "executing"
  | "verifying"
  | "completed"
  | "partially_completed"
  | "failed"
  | "cancelled";
```

The application must distinguish between planned, attempted, and successfully completed actions.

---

# 11. File Management Agent

**Priority: P0**

## 11.1 Purpose

Allow users to organize and manage files using natural-language instructions.

## 11.2 Core Features

- File identification.
- File categorization.
- File renaming.
- File organization.
- Folder creation.
- Exact duplicate detection.
- File metadata extraction.
- Content-based classification.
- Batch processing.
- Operation previews.
- Undo for supported operations.

## 11.3 Example Workflow

**User:**

"Organize my Downloads folder into Business, University, Personal, and Images."

**Expected Process:**

1. Request folder access.
2. Scan supported files.
3. Identify categories.
4. Produce an organization plan.
5. Display proposed changes.
6. Request approval.
7. Execute approved operations.
8. Verify file locations.
9. Save the audit history locally.

## 11.4 Example Output

```text
Downloads/
    Business/
        Invoices/
        Reports/

    University/
        Lectures/
        Research/

    Personal/
        Documents/

    Images/
        Photos/
        Graphics/
```

## 11.5 Safety Requirements

- Never move files without required authorization.
- Never overwrite files silently.
- Never permanently delete files in the MVP.
- Detect filename collisions.
- Validate file paths.
- Prevent path traversal.
- Reject unauthorized directory access.
- Record reversible operations.

### Acceptance Criteria

The File Management Agent can organize an authorized folder using a user-approved plan without modifying unrelated directories.

---

# 12. Document Intelligence Agent

**Priority: P0**

## 12.1 Purpose

Allow users to analyze documents, extract information, and generate useful outputs.

## 12.2 Supported Formats

| Format | Read | Generate |
|---|---|---|
| PDF | Yes | Yes |
| TXT | Yes | Yes |
| Markdown | Yes | Yes |
| CSV | Yes | Yes |
| XLSX | Yes | Yes |
| DOCX | Yes | Yes |
| PNG/JPEG | Basic local OCR | No |
| PPTX | Future | Future |

Advanced formatting and complex document structures may require additional compatibility work.

## 12.3 Document Summarization

### Example

"Summarize these three research papers."

Expected output:

- Individual paper summaries.
- Main findings.
- Research methodologies.
- Key differences.
- Limitations.
- Combined summary.
- Source references.

### Required Capabilities

- Extract document text.
- Process large files in chunks.
- Identify document structure.
- Summarize extracted content.
- Compare multiple documents.
- Generate structured notes.
- Preserve available page references.
- Export results.

## 12.4 Academic Document Processing

Supported use cases should include:

- Lecture summaries.
- Topic-wise notes.
- Practice question generation.
- Study guides.
- Research paper comparisons.
- Concept explanations.

The application should make clear when information cannot be found in the supplied documents.

## 12.5 Data Storage Requirements

Original documents should remain in their existing location.

Extracted text and indexes should be cached locally only when necessary.

Users must be able to delete cached document content and indexes.

### Acceptance Criteria

The user can summarize a supported PDF offline, obtain an exportable result, and verify source references where available.

---

# 13. Spreadsheet and Business Analysis Agent

**Priority: P0**

## 13.1 Purpose

Automate common spreadsheet and financial reporting operations.

This feature is especially valuable to business owners.

## 13.2 Supported Tasks

- Read spreadsheets.
- Detect columns.
- Validate data.
- Remove exact duplicate records when requested.
- Filter records.
- Sort records.
- Group information.
- Calculate totals.
- Calculate averages.
- Generate summaries.
- Create charts.
- Export structured reports.

## 13.3 Business Reporting

### Example

User:

"Analyze my sales report and show which product generated the highest revenue."

The application should:

1. Read the spreadsheet.
2. Identify relevant columns.
3. Validate numeric values.
4. Calculate revenue.
5. Group records by product.
6. Rank products.
7. Generate a summary.
8. Export the report.

## 13.4 Invoice Processing

A user selects a folder containing invoices.

The agent extracts:

- Invoice number.
- Vendor.
- Date.
- Amount.
- Currency.
- Tax.
- Payment status, if explicitly present.

Information that cannot be reliably extracted must be flagged for review.

## 13.5 Calculation Integrity

The AI model should not be responsible for performing final financial arithmetic.

Use deterministic code for:

- Summation.
- Currency calculations.
- Percentage calculations.
- Profit calculations.
- Expense aggregation.
- Data validation.

AI should interpret results and generate explanations.

### Acceptance Criteria

The numeric results in an exported report must match the verified calculations performed on the input data.

---

# 14. Workspace Management

**Priority: P0**

## 14.1 Purpose

Allow users to separate tasks and documents by project or activity.

### Example Workspaces

```text
ActionDesk

├── University
│   ├── Research
│   ├── Assignments
│   └── Lecture Notes
│
├── Business
│   ├── Financial Reports
│   ├── Invoices
│   └── Clients
│
├── Freelance Projects
│   ├── Project A
│   ├── Project B
│   └── Project C
│
└── Personal
    └── Documents
```

## 14.2 Workspace Requirements

Each workspace must contain:

- Workspace name.
- Description.
- Authorized directories.
- Conversation history.
- Tasks.
- Generated output references.
- Preferred AI model.
- Local settings.

## 14.3 Storage Requirements

Workspace metadata must be stored in the user's local SQLite database.

Files should remain in their original locations unless the user requests a move or copy.

Deleting a workspace must not automatically delete original documents.

---

# 15. Local Database Architecture

**Priority: P0**

## 15.1 Database Technology

Use SQLite as the primary application database.

### Why SQLite?

- No database server.
- No hosting fees.
- Works offline.
- Lightweight.
- Embedded into desktop applications.
- Supports transactions.
- Suitable for local task metadata.

A hosted database such as Supabase, Firebase, or PostgreSQL is unnecessary for the core MVP.

## 15.2 Proposed Database Tables

```text
actiondesk.sqlite

├── workspaces
├── conversations
├── messages
├── tasks
├── task_steps
├── tool_executions
├── file_references
├── permission_grants
├── generated_artifacts
├── model_configurations
├── saved_workflows
├── application_settings
├── document_index_metadata
└── audit_events
```

## 15.3 Core Entity Definitions

### Workspace

```typescript
interface Workspace {
  id: string;
  name: string;
  description?: string;
  createdAt: string;
  updatedAt: string;
}
```

### Task

```typescript
interface AgentTask {
  id: string;
  workspaceId: string;

  instruction: string;

  status:
    | "created"
    | "planning"
    | "awaiting_approval"
    | "executing"
    | "verifying"
    | "completed"
    | "partially_completed"
    | "failed"
    | "cancelled";

  modelId: string;

  createdAt: string;
  completedAt?: string;
}
```

### Generated Artifact

```typescript
interface GeneratedArtifact {
  id: string;
  taskId: string;

  fileName: string;
  localPath: string;

  mimeType: string;
  createdAt: string;
}
```

### AI Model

```typescript
interface LocalModel {
  id: string;
  name: string;

  filePath: string;
  fileSize: number;

  contextLength: number;

  installed: boolean;
  isDefault: boolean;
}
```

These interfaces are proposed application contracts, not finalized database schemas.

## 15.4 Database Requirements

- Automatic schema migrations.
- Transactional updates.
- Local backup/export.
- Recovery from interrupted writes.
- Configurable retention.
- Secure handling of sensitive records.
- No compulsory external synchronization.

---

# 16. Local Search and Document Indexing

**Priority: P1**

## 16.1 Purpose

Allow users to search document contents using natural language.

### Example

"Find the PDF where I discussed Bayesian regression."

The application searches authorized local documents and returns matching files.

## 16.2 Search Architecture

```text
Local Documents
       |
       v
Text Extraction
       |
       v
Local Chunking
       |
       v
Local Embedding Model
       |
       v
Local Search Index
       |
       v
Semantic Search
       |
       v
Relevant Documents
```

## 16.3 Storage Strategy

Do not use a hosted vector database.

Potential solutions include:

- SQLite FTS5 for full-text search.
- Local vector-index libraries.
- Embedded vector extensions where suitable.

Semantic search is optional for the MVP.

For the first release, filename search and extracted-text search may be sufficient.

## 16.4 Index Management

Users must be able to:

- Index a selected folder.
- Remove a folder from the index.
- Rebuild an index.
- Delete all indexes.
- View index storage usage.
- Disable automatic indexing.

Indexes must not be uploaded to Foxmen Studio servers.

---

# 17. User Interface and Experience

## 17.1 Design Philosophy

ActionDesk AI should feel like a premium macOS productivity application.

The interface must be:

- Minimal.
- Modern.
- Fast.
- Intuitive.
- Accessible.
- Consistent.
- Transparent during AI execution.

## 17.2 Main Application Layout

```text
┌──────────────────────────────────────────────────────────┐
│ ActionDesk                                  Local AI ●   │
├────────────────┬─────────────────────────────────────────┤
│                │                                         │
│  + New Task    │          Good Morning                   │
│                │                                         │
│  Home          │       What can I do for you?            │
│                │                                         │
│  Workspaces    │  ┌───────────────────────────────────┐  │
│                │  │ Describe a task...                │  │
│  Files         │  └───────────────────────────────────┘  │
│                │                                         │
│  Workflows     │  Quick Actions                          │
│                │                                         │
│  History       │  ┌──────────────┐ ┌──────────────┐      │
│                │  │ Organize     │ │ Summarize    │      │
│  Storage       │  │ Files        │ │ Documents    │      │
│                │  └──────────────┘ └──────────────┘      │
│  Settings      │                                         │
│                │  ┌──────────────┐ ┌──────────────┐      │
│                │  │ Analyze      │ │ Generate     │      │
│                │  │ Spreadsheet  │ │ Report       │      │
│                │  └──────────────┘ └──────────────┘      │
│                │                                         │
├────────────────┴─────────────────────────────────────────┤
│ Offline Mode       Model: Local        Storage: 4.7 GB    │
└──────────────────────────────────────────────────────────┘
```

## 17.3 Required Screens

| Screen | Purpose |
|---|---|
| Onboarding | First-time setup |
| Dashboard | Main interaction |
| Task Workspace | Agent conversations |
| File Manager | File processing |
| Document Workspace | Document analysis |
| Task Preview | Approval interface |
| Task Execution | Progress monitoring |
| Results | Generated outputs |
| History | Previous task execution |
| Storage | Local storage management |
| Models | Local AI management |
| Settings | Application configuration |

## 17.4 Onboarding Process

### Step 1: Welcome

Explain the core promise:

"Your AI runs on your Mac. Your data stays with you."

### Step 2: Hardware Detection

Detect:

- macOS version.
- Processor.
- Available memory.
- Available storage.

### Step 3: AI Model Setup

Recommend a compatible model.

Display:

- Model name.
- Download size.
- Storage requirements.
- Estimated hardware suitability.

### Step 4: Privacy Configuration

Explain that the application processes data locally by default.

### Step 5: Choose Output Location

Allow the user to select a default directory.

### Step 6: First Task

Provide a simple demonstration such as organizing a sample folder.

Users should not be required to create an account or enter payment information.

---

# 18. Task Approval and Security System

**Priority: P0 — Critical**

ActionDesk will interact with real user files, so safety controls must be enforced outside the language model.

## 18.1 Permission Model

The application should request access only to directories explicitly selected by the user.

Avoid requesting Full Disk Access for ordinary MVP workflows.

Use appropriate macOS file access mechanisms.

## 18.2 Risk Classification

| Risk | Examples | Policy |
|---|---|---|
| Low | Read files, summarize documents | Allowed within granted scope |
| Medium | Generate reports, create folders | Show destination and planned changes |
| High | Rename, move, replace files | Explicit approval |
| Critical | Permanent deletion, terminal execution | Not supported in MVP |

## 18.3 Approval Interface

```text
ActionDesk wants to organize 42 files.

Planned Actions

Create folders:    5
Move files:       42
Rename files:      0
Delete files:      0

Source:
~/Downloads

Destination:
~/Documents/Organized

[Review Changes]

[Cancel]        [Approve & Execute]
```

## 18.4 Security Requirements

**SEC-001:** Restrict operations to authorized locations.

**SEC-002:** Validate paths and symbolic links.

**SEC-003:** Prevent path traversal.

**SEC-004:** Block unauthorized file modifications.

**SEC-005:** Protect API keys using macOS Keychain.

**SEC-006:** Treat document contents as untrusted input.

**SEC-007:** Prevent document-based prompt injection from overriding tool permissions.

**SEC-008:** Never execute arbitrary shell commands generated by the LLM.

**SEC-009:** Avoid logging sensitive document contents.

**SEC-010:** Support safe cancellation and recovery.

**SEC-011:** Require authorization before transmitting files externally.

**SEC-012:** Record meaningful file modifications in a local audit log.

## 18.5 Undo and Recovery

Every supported reversible file modification should have a corresponding recovery mechanism.

Examples:

- Rename → Restore original filename.
- Move → Restore original location.
- Create folder → Remove if empty and created by the task.
- Generate document → Preserve output and allow explicit removal.

Permanent deletion will not be supported initially.

---

# 19. Offline Capabilities

## 19.1 Fully Offline Features

After downloading a compatible local model, the following must work without internet connectivity.

| Feature | Offline Support |
|---|---|
| AI conversation | Yes |
| File organization | Yes |
| PDF extraction | Yes |
| Document summarization | Yes |
| Spreadsheet analysis | Yes |
| Local report generation | Yes |
| Local task history | Yes |
| Workspace management | Yes |
| File search | Yes |
| Local workflows | Yes |
| AI model loading | Yes |

## 19.2 Features Requiring Internet

| Feature | Internet Requirement |
|---|---|
| Initial model download | Usually required |
| Software updates | Required to check/download |
| Grok API | Required |
| Other external AI providers | Required |
| Online integrations | Required |
| Payment processing | May be required |
| Online documentation | Required unless bundled |

## 19.3 Offline Mode Requirements

When the user is offline:

- The application must not continuously attempt cloud requests.
- Core features must remain accessible.
- Task history must remain available.
- The UI must clearly indicate the active AI model.
- Optional online features must show an appropriate offline message.

---

# 20. Software Distribution Strategy

## 20.1 Distribution Principle

Foxmen Studio should distribute a downloadable desktop application instead of hosting user workspaces.

### Initial Distribution

Provide a signed, notarized macOS installer from the product website or a suitable release-hosting provider.

For early prototypes, GitHub Releases or another third-party release service may reduce distribution infrastructure requirements, subject to its current terms and limits.

## 20.2 Application Packaging

The installer should contain:

- Desktop interface.
- Application backend.
- Agent orchestration engine.
- Tool implementations.
- Local database initialization.
- Local inference runtime.
- Model configuration system.

Large AI model files should generally be downloaded separately.

### Reason

Bundling multiple large AI models would dramatically increase application download size.

A smaller installer is more practical for distribution.

## 20.3 Installation Flow

```text
Download ActionDesk.dmg
           |
           v
Install Application
           |
           v
Launch ActionDesk
           |
           v
Detect Hardware
           |
           v
Download Local Model
           |
           v
Configure Storage
           |
           v
Start Using AI
```

## 20.4 Update Strategy

The application should support:

- Signed software updates.
- Release notes.
- Update verification.
- Database migrations.
- Recovery from failed updates.

Application updates and model downloads should be managed separately.

---

# 21. Recommended Technology Stack

## 21.1 Core Technologies

| Component | Recommended Technology |
|---|---|
| Desktop framework | Tauri 2 |
| User interface | React |
| Programming language | TypeScript |
| Styling | Tailwind CSS |
| UI components | shadcn/ui |
| Core backend | Rust |
| Native macOS functionality | Swift |
| Local database | SQLite |
| Local inference | Embedded llama.cpp-compatible runtime |
| Early prototype inference | Ollama |
| Apple Silicon acceleration | Metal-compatible inference |
| Optional future inference | MLX |
| PDF processing | PDFKit / compatible local parsers |
| OCR | Apple Vision |
| Spreadsheet reading | Rust spreadsheet libraries |
| Spreadsheet generation | Rust XLSX writer |
| Full-text search | SQLite FTS5 |
| Optional vector search | Local vector index |
| Secure credential storage | macOS Keychain |
| Application distribution | Signed and notarized DMG |

Technology selection should be finalized after a feasibility prototype.

## 21.2 Why Not Use a Hosted Backend?

A hosted backend would introduce avoidable costs and dependencies for the MVP.

For example:

| Hosted Technology | Local Alternative |
|---|---|
| Supabase database | SQLite |
| Firebase database | SQLite |
| Hosted vector database | Local search index |
| Cloud object storage | User's filesystem |
| Hosted LLM | Local open-weight model |
| Cloud job queue | Local task scheduler |
| Cloud document processing | Native local processing |
| Cloud workflow execution | Local agent engine |

This is a product architecture decision, not an assertion that these hosted technologies are inherently unsuitable.

They simply are not required for ActionDesk's initial use cases.

## 21.3 Example Project Structure

```text
actiondesk/
│
├── src/
│   ├── app/
│   │   ├── dashboard/
│   │   ├── workspace/
│   │   ├── tasks/
│   │   ├── files/
│   │   ├── models/
│   │   └── settings/
│   │
│   ├── components/
│   ├── hooks/
│   ├── stores/
│   ├── services/
│   └── types/
│
├── src-tauri/
│   ├── src/
│   │   ├── agents/
│   │   │   ├── planner.rs
│   │   │   ├── executor.rs
│   │   │   └── verifier.rs
│   │   │
│   │   ├── tools/
│   │   │   ├── files.rs
│   │   │   ├── documents.rs
│   │   │   ├── spreadsheets.rs
│   │   │   └── reports.rs
│   │   │
│   │   ├── ai/
│   │   │   ├── inference.rs
│   │   │   ├── models.rs
│   │   │   └── providers.rs
│   │   │
│   │   ├── storage/
│   │   │   ├── sqlite.rs
│   │   │   ├── cache.rs
│   │   │   └── models.rs
│   │   │
│   │   ├── security/
│   │   │   ├── permissions.rs
│   │   │   ├── validation.rs
│   │   │   └── approvals.rs
│   │   │
│   │   └── main.rs
│   │
│   └── capabilities/
│
├── tests/
│   ├── agents/
│   ├── documents/
│   ├── files/
│   └── security/
│
└── README.md
```

---

# 22. Performance Requirements

Performance will vary according to user hardware.

The following are proposed engineering targets, not guarantees.

| Metric | Initial Target |
|---|---|
| App startup excluding model loading | Under 3 seconds |
| Typical UI interaction latency | Under 100 ms |
| File metadata scan | 1,000 typical files within 10 seconds |
| Local AI response | Hardware and model dependent |
| Task cancellation | Safe, prompt interruption |
| Offline operations | No external dependency |
| Database availability | Local |
| Successful task recovery | Tested for supported operations |
| Crash-free sessions | ≥99% |

## 22.1 Resource Management

The application should:

- Avoid loading unnecessary models.
- Release models when possible.
- Limit concurrent agent execution.
- Restrict document chunk sizes.
- Set memory limits where feasible.
- Prevent uncontrolled agent loops.
- Cap temporary cache growth.
- Avoid unnecessary file duplication.

## 22.2 Low-Memory Devices

Users with 8 GB Macs may require smaller models and shorter context windows.

The software must be transparent about limitations.

It must not promise that a small local model can perform every complicated task reliably.

---

# 23. Reusable Automation Workflows

**Priority: P1**

## 23.1 Purpose

Allow users to save common tasks and execute them repeatedly.

### Example

"Create a workflow that organizes new invoices and prepares an expense report."

The application generates:

```text
Workflow:
Monthly Invoice Processing

Trigger:
Manual

Actions:
1. Read selected invoice folder.
2. Extract invoice information.
3. Validate values.
4. Categorize invoices.
5. Calculate expenses.
6. Generate Excel report.
7. Save output locally.
```

## 23.2 Local Execution

Workflows must execute on the user's computer.

No cloud job queue is necessary.

### Future Trigger Types

- Manual execution.
- Scheduled execution.
- Authorized folder changes.
- Supported application events.

If ActionDesk is closed, scheduled operations must follow the documented local background-execution behavior.

The software must not imply that tasks can execute while the computer is powered off.

## 23.3 Workflow Security

Saved workflows must preserve permissions and authorization limits.

High-risk actions cannot silently bypass approval merely because a workflow was previously saved.

---

# 24. Privacy and Data Protection

## 24.1 Privacy Commitment

**"Your files stay on your device. Your AI works on your device."**

This should be a central marketing message, with clear disclosure of any optional online capabilities.

## 24.2 Privacy Requirements

The application must not automatically collect:

- Document contents.
- Private file paths.
- Conversation transcripts.
- Financial records.
- Research documents.
- API keys.
- Personal files.

Product analytics should be disabled by default or require explicit opt-in.

## 24.3 External Services

When users activate third-party providers, the application must explain:

- Which provider receives data.
- What data will be transmitted.
- Whether charges may apply.
- How the provider handles submitted content.
- How to disable the integration.

## 24.4 User Backups

Because Foxmen Studio will not store user documents, **Foxmen Studio cannot guarantee recovery if a user loses their Mac or local drive**.

The application should support exporting workspace metadata and important application settings.

Users may save these backups on:

- External SSDs.
- USB drives.
- Another local computer.
- Their own cloud-synchronized folders.

If a user chooses iCloud Drive, Google Drive, or another synchronized location, the operating system or third-party provider may upload those files. That is separate from ActionDesk's default local processing.

The user must understand this distinction.

---

# 25. Business Model

## 25.1 Recommended Strategy

Use a free core application with optional paid software features.

Do not require users to pay for cloud storage they do not need.

## 25.2 Proposed Plans

| Feature | Free | Pro |
|---|---|---|
| Local AI | Yes | Yes |
| Local file processing | Yes | Yes |
| Offline AI chat | Yes | Yes |
| Basic file organization | Yes | Yes |
| PDF summarization | Yes | Yes |
| Basic spreadsheet analysis | Yes | Yes |
| Advanced reporting templates | Limited | Full |
| Reusable workflows | Limited | Full |
| Advanced batch automation | No | Yes |
| Premium productivity templates | No | Yes |
| Cloud AI | BYOK | BYOK |
| User-owned local storage | Yes | Yes |

### Possible Pricing

- **Free:** $0.
- **Pro:** One-time purchase, with pricing to be validated.
- **Business:** Optional advanced license.

A one-time license is worth evaluating because the core application does not require ongoing AI infrastructure.

However, ongoing software development and support still cost money.

Licensing and updates must be structured accordingly.

## 25.3 License Management

The free product must work without registration.

A premium license system may be introduced later.

Possible approaches:

- Offline-verifiable signed license files.
- Third-party license management.
- Online activation for optional premium features.

The MVP should avoid a complex licensing backend.

## 25.4 Payment Infrastructure

If premium licenses are sold, payment processing can be handled through an established payment provider.

Foxmen Studio should not build its own payment-processing infrastructure.

Payment-related services are separate from user file storage and local AI execution.

---

# 26. Startup Cost Optimization

This section specifies how the product should minimize recurring infrastructure expenses.

## 26.1 Infrastructure Cost Matrix

| Infrastructure | Required for MVP? | Strategy |
|---|---|---|
| Application server | No | Local execution |
| Cloud database | No | SQLite |
| Cloud object storage | No | User's storage |
| Hosted AI models | No | Local inference |
| Paid AI API | No | Optional BYOK |
| Cloud vector database | No | Local index |
| User authentication backend | No | No mandatory accounts |
| Cloud workflow engine | No | Local task execution |
| AI model hosting | No | Direct authorized model downloads |
| Product website | Optional, recommended | Static hosting |
| Installer distribution | Yes | Suitable release hosting |
| Apple code signing/notarization | Yes for recommended public distribution | Apple developer infrastructure |
| Payment provider | Only if selling licenses | Third-party service |
| Support infrastructure | Recommended | Low-cost email/documentation |

## 26.2 Development Cost Strategy

As an early-stage startup, Foxmen Studio should prioritize existing skills and open-source technologies.

Recommended:

- Tauri instead of maintaining separate native desktop codebases.
- SQLite instead of hosted databases.
- Existing local inference runtimes.
- Open-weight models with compatible commercial licenses.
- Built-in macOS frameworks where practical.
- Static documentation.
- Direct model downloads.
- Minimal external dependencies.

## 26.3 Costs That Cannot Be Completely Eliminated

Even with a fully local architecture, potential expenses remain:

- Apple Developer Program membership for normal signing/notarization workflows.
- Domain registration.
- Website hosting.
- Installer bandwidth or hosting.
- Support and maintenance.
- Testing hardware.
- Payment processing fees.
- Security maintenance.
- Software development.

The product requirement is therefore:

**Eliminate mandatory recurring cloud storage and AI inference costs, not all possible business expenses.**

---

# 27. Product Development Roadmap

## Phase 0 — Technical Validation

### Objective

Determine whether the proposed local AI architecture is technically viable.

### Deliverables

- Basic Tauri desktop application.
- Local LLM integration.
- Structured tool-calling prototype.
- User-selected file access.
- Basic file operation execution.
- Local SQLite database.
- Basic document extraction.

### Exit Criteria

The application can interpret an instruction, generate a valid plan, execute an authorized local tool, and verify the result.

---

## Phase 1 — Core Desktop Application

### Objective

Build the initial usable application.

### Deliverables

- Dashboard.
- Chat interface.
- AI model management.
- Workspace management.
- File permissions.
- Agent orchestration.
- Task history.
- Storage manager.

### Exit Criteria

Users can install the application, configure a local model, select files, and initiate supported tasks.

---

## Phase 2 — Core Productivity Agents

### Objective

Deliver the three primary workflows.

### Deliverables

**File Agent**

- File scanning.
- Classification.
- Organization.
- Renaming.
- Approval.
- Undo.

**Document Agent**

- PDF extraction.
- Text summarization.
- Document comparison.
- Report generation.

**Spreadsheet Agent**

- Spreadsheet reading.
- Data validation.
- Calculation.
- Summary generation.
- Export.

### Exit Criteria

Each workflow operates reliably using the local AI model and local tools.

---

## Phase 3 — Private Beta

### Objective

Validate the product with real users.

### Activities

- Test different Mac configurations.
- Measure AI execution reliability.
- Measure memory consumption.
- Test storage management.
- Perform security testing.
- Validate file recovery.
- Gather usability feedback.
- Fix critical bugs.

### Exit Criteria

The release meets minimum reliability, security, and usability targets.

---

## Phase 4 — Public MVP

### Deliverables

- Signed and notarized macOS installer.
- Public release page.
- Local model installer.
- Core agents.
- Privacy documentation.
- User documentation.
- Support process.
- Software update system.

---

## Phase 5 — Version 1 Expansion

### Planned Features

- Optional Grok API.
- Bengali-language support.
- Semantic document search.
- Reusable workflows.
- Scheduled task execution.
- Advanced OCR.
- Improved local model selection.
- Advanced business report templates.

---

## Phase 6 — Long-Term Expansion

Potential future releases:

- Windows support.
- Cross-platform local agents.
- Browser automation.
- Supported application automation.
- Voice control.
- Additional AI providers.
- Plugin marketplace.
- Business-specific workflow packs.

These features require separate feasibility assessments and specifications.

---

# 28. Testing and Quality Assurance

## 28.1 Testing Strategy

| Test Type | Focus |
|---|---|
| Unit Testing | Core logic |
| Integration Testing | Agent and tool interaction |
| Functional Testing | Real task completion |
| Security Testing | Permissions and unsafe actions |
| Performance Testing | Memory, CPU, latency |
| Offline Testing | Network independence |
| Storage Testing | Disk usage and recovery |
| Compatibility Testing | Apple Silicon configurations |
| Usability Testing | Nontechnical users |
| Regression Testing | Previously working workflows |

## 28.2 Mandatory Test Cases

| ID | Test Case | Expected Result |
|---|---|---|
| T-001 | Run app without internet | Core functions remain available |
| T-002 | Summarize local PDF | Correctly generates local output |
| T-003 | Organize 100 files | Authorized changes only |
| T-004 | Analyze CSV | Correct calculations |
| T-005 | Generate Excel report | Valid file created |
| T-006 | Delete downloaded model | Storage is reclaimed |
| T-007 | Clear temporary cache | Disposable cache is removed |
| T-008 | Disk becomes full | Safe error and recovery |
| T-009 | App crashes during a task | Recoverable state |
| T-010 | User rejects file operation | No changes performed |
| T-011 | Unauthorized folder requested | Access denied |
| T-012 | Internet disconnects | Local task continues |
| T-013 | Grok unavailable | Local functions unaffected |
| T-014 | Remove workspace | Original documents preserved |
| T-015 | Undo file organization | Supported changes restored |
| T-016 | Malicious instructions inside PDF | Agent permissions remain enforced |
| T-017 | AI generates invalid tool call | Tool rejected |
| T-018 | Model exceeds available memory | Clear compatibility error |
| T-019 | Destination filename collision | No silent overwrite |
| T-020 | User changes output directory | Files save to approved location |

## 28.3 Product Quality Targets

| Metric | Initial Target |
|---|---|
| Supported benchmark task completion | ≥90% |
| Crash-free sessions | ≥99% |
| Unauthorized destructive operations | 0 |
| Offline core workflow availability | 100% of defined offline workflows |
| Financial calculation accuracy | Exact agreement with validated calculation engine |
| First-task activation | ≥70% of test users |
| Usability success | ≥80% complete a basic task without assistance |

These targets must be measured during development and beta testing.

---

# 29. Key User Stories and Acceptance Criteria

## US-001: Install Without an Account

**As a user,** I want to download and install ActionDesk without creating an account.

**Acceptance Criteria:**

- No login is required.
- Local model installation is available.
- Core workflows operate offline.
- No personal document upload is required.

## US-002: Use My Own Storage

**As a user,** I want ActionDesk to use my computer's existing storage.

**Acceptance Criteria:**

- No mandatory cloud storage.
- Models stored locally.
- Documents remain local.
- Reports saved locally.
- Cache can be cleared.
- Storage use is visible.

## US-003: Organize Files Automatically

**As a user,** I want to organize my files through natural-language instructions.

**Acceptance Criteria:**

- Folder selection is supported.
- Changes are previewed.
- Approval is required for file moves.
- Results are verified.
- Supported modifications can be reversed.

## US-004: Analyze Documents Offline

**As a student,** I want to summarize academic documents without uploading them.

**Acceptance Criteria:**

- PDFs can be processed locally.
- Summaries are generated offline.
- Outputs are stored locally.
- Source references are shown when available.

## US-005: Generate Business Reports

**As a business owner,** I want to analyze spreadsheets and generate financial reports.

**Acceptance Criteria:**

- Supported files can be imported.
- Numeric data is validated.
- Calculations use deterministic code.
- Results are exportable.
- No mandatory API call is made.

## US-006: Manage AI Models

**As a user,** I want control over downloaded AI models.

**Acceptance Criteria:**

- Models can be downloaded.
- Download sizes are visible.
- Model compatibility is displayed.
- Models can be removed.
- Default model can be changed.

## US-007: Use Optional Cloud AI

**As a user,** I want to connect my own Grok API key when needed.

**Acceptance Criteria:**

- Integration is optional.
- API key is stored securely.
- Local mode remains available.
- Users control external data transmission.
- The application does not silently incur API charges.

## US-008: Recover from Failed Tasks

**As a user,** I want to understand what happened when an automation fails.

**Acceptance Criteria:**

- Completed steps are identified.
- Failed steps are identified.
- Modified files are listed.
- Supported actions can be undone.
- Partial completion is not reported as full success.

---

# 30. Product Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Small local models make planning mistakes | High | Restrict tools and benchmark models |
| Users have insufficient RAM | High | Lightweight model options |
| Users have insufficient storage | Medium | Storage manager and download checks |
| Agent moves files incorrectly | High | Approval and undo |
| Malicious document instructions | High | Isolate untrusted data and enforce permissions |
| Model download provider becomes unavailable | Medium | Support multiple permitted sources |
| Model licensing restrictions | High | Validate commercial licenses |
| No cloud backups | Medium | Local export and user-managed backups |
| Complex feature scope | High | Strict MVP prioritization |
| Weak monetization | Medium | Validate willingness to pay |
| Competition from established products | High | Focus on reliable workflow execution |
| macOS permission restrictions | Medium | Use approved native APIs |

---

# 31. Success Metrics

## 31.1 North Star Metric

**Weekly Successfully Completed Tasks**

The number of verified tasks successfully completed by users within one week.

## 31.2 Product KPIs

| KPI | Purpose |
|---|---|
| First Task Completion | Measure activation |
| Weekly Active Users | Measure engagement |
| Task Success Rate | Measure agent reliability |
| Average Execution Time | Measure performance |
| Repeat Workflow Usage | Measure sustained value |
| Local Model Installation Success | Measure onboarding quality |
| Storage-Related Failures | Monitor resource limitations |
| Task Cancellation Rate | Identify usability problems |
| Undo Frequency | Detect potentially incorrect operations |
| Premium Conversion | Measure monetization |

## 31.3 Analytics Policy

Analytics must be optional and privacy-preserving.

The application must not upload user documents, private paths, or conversation transcripts for product analytics.

If analytics are disabled, the application should still operate normally.

---

# 32. MVP Release Checklist

Before launching the first public version, the following must be complete.

| Requirement | Mandatory |
|---|---|
| Installable macOS application | Yes |
| Local AI model support | Yes |
| Offline AI inference | Yes |
| No required account | Yes |
| No required developer backend | Yes |
| User-owned data storage | Yes |
| Local SQLite database | Yes |
| AI task planning | Yes |
| Secure tool execution | Yes |
| File organization | Yes |
| PDF processing | Yes |
| Spreadsheet analysis | Yes |
| Local report generation | Yes |
| File access permission management | Yes |
| Approval before sensitive operations | Yes |
| Task history | Yes |
| Storage management | Yes |
| Model download management | Yes |
| Task verification | Yes |
| Undo for supported file operations | Yes |
| Graceful failures | Yes |
| Security testing | Yes |
| Signed and notarized distribution | Yes |
| User documentation | Yes |

---

# 33. Product Design Constraints

The following restrictions must be respected throughout development.

### Constraint 1: No Mandatory Cloud Storage

The application must never require Foxmen Studio-hosted storage for ordinary operation.

### Constraint 2: No Mandatory Paid AI API

Users must have access to meaningful local AI features without purchasing API credits.

### Constraint 3: No Mandatory Backend

Application startup, task execution, and local workspace access must not depend on a developer-controlled server.

### Constraint 4: User-Controlled Storage

Users must control where generated outputs are saved and be able to manage local application storage.

### Constraint 5: Limited System Access

The application must not request broader operating-system permissions than its supported workflows require.

### Constraint 6: Reliable Execution

The product must prioritize correct execution over unsupported claims of complete computer autonomy.

### Constraint 7: Startup-Friendly Engineering

The implementation must avoid unnecessary infrastructure, excessive dependencies, and features that significantly increase operating costs without validating user demand.

---

# 34. Founder-Focused MVP Recommendation

Although the long-term ambition is a general-purpose computer agent, Foxmen Studio should not attempt to build all advanced capabilities immediately.

The first product should focus on three workflows.

## Workflow A: Intelligent File Organizer

**Command:**

"Organize my Downloads folder by file category."

**Product Value:** Saves manual file-management time.

**Technical Complexity:** Moderate.

**Priority:** First.

## Workflow B: Document Intelligence

**Command:**

"Summarize these lecture PDFs and generate organized study notes."

**Product Value:** Useful for students, researchers, and professionals.

**Technical Complexity:** Moderate to high.

**Priority:** Second.

## Workflow C: Business Report Generator

**Command:**

"Analyze these sales spreadsheets and create a monthly report."

**Product Value:** Provides measurable business value.

**Technical Complexity:** Moderate.

**Priority:** Third.

These three workflows share important infrastructure:

- Natural-language understanding.
- Local AI inference.
- Task planning.
- File access.
- Permission management.
- Data processing.
- Local output generation.

Building a common execution engine for these workflows provides a foundation for future capabilities.

---

# 35. Final Product Specification

| Category | Decision |
|---|---|
| Product Name | ActionDesk AI — Temporary |
| Company | Foxmen Studio |
| Product Type | Agentic AI Desktop Application |
| Initial Platform | macOS |
| Target Architecture | Apple Silicon |
| Main Users | Students, Business Owners, Entrepreneurs |
| Primary AI | Local open-weight model |
| Optional AI | Grok and compatible providers |
| Local AI Framework | Embedded runtime; Ollama for prototyping |
| Desktop Framework | Tauri |
| Frontend | React + TypeScript |
| Backend | Rust |
| Native Integration | Swift |
| Database | SQLite |
| Search | Local indexing |
| File Storage | User's computer |
| Model Storage | User's computer |
| Conversation Storage | User's computer |
| Generated Reports | User's computer |
| Company Cloud Storage | Not required |
| Company AI Inference Servers | Not required |
| Mandatory User Account | No |
| Mandatory Subscription | No |
| Mandatory API Payments | No |
| Offline Functionality | Yes |
| Security | Permission-based tool execution |
| Distribution | Downloadable macOS installer |
| Initial Monetization | Free core, optional Pro |
| Future Expansion | Windows, workflows, integrations |

---

# 36. Final Vision

## ActionDesk AI — Your Computer, Your AI, Your Data

ActionDesk AI will be an intelligent desktop assistant that executes useful work directly on the user's computer.

The software will not depend on expensive cloud infrastructure for its core functionality.

Instead, it will utilize the computing resources that users already own.

### For Students

A private AI assistant that organizes academic materials, summarizes documents, and prepares study resources.

### For Business Owners

An AI-powered productivity tool that analyzes financial data, organizes business documents, and produces reports.

### For Entrepreneurs

A digital assistant that helps process client requirements, organize projects, and automate administrative work.

### For Foxmen Studio

A scalable software product that can serve users internationally without requiring proportional growth in cloud storage and AI inference expenses.

**The product's central promise:**

> **Download it once. Set up your local AI. Keep your files on your computer. Let the agent handle your work.**

---

## 37. Final Development Decision

For the first release, Foxmen Studio should adopt the following non-negotiable architecture:

| Component | Final Choice |
|---|---|
| Desktop Application | Tauri + React |
| Agent Core | Rust |
| AI Inference | On-device |
| AI Models | Downloaded to the user's Mac |
| Database | Local SQLite |
| File Storage | User-owned |
| Generated Outputs | User-selected folders |
| Embeddings and Indexes | Stored locally |
| Credentials | macOS Keychain |
| Cloud AI | Optional, user-authorized |
| Developer-Hosted User Storage | **None** |
| Developer-Hosted Inference | **None** |
| Mandatory Backend | **None** |

### Final MVP Definition

**ActionDesk AI MVP is a downloadable macOS application that uses local AI and the user's own computer resources to understand instructions, organize files, analyze documents, process spreadsheets, and generate reports—all without mandatory cloud storage, developer-hosted inference, or recurring AI API payments.**

This is the foundational technical and business requirement for the startup.

**PRD Version 2.0 — Complete.**