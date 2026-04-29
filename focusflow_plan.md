# FocusFlow Project Plan

## Overview
FocusFlow is an AI-driven personal task manager starting as a Telegram Mini App, evolving into a standalone application. Core cycle: Input -> Focus -> Retrospective.

## Positioning (USP)
1. **Thinking Partner**: Conversational AI, not just a command parser.
2. **Telegram-native**: Seamless integration (forward-to-task, inline commands, notifications).
3. **Energy-aware**: Adaptive planning based on user state (SDHD/burnout friendly).

## Architecture
- **Frontend**: Vite + React + TypeScript + @telegram-apps/sdk-react.
- **Backend**: Rust workspace (Axum/api, domain, ai, bot, migrations).
- **Database**: PostgreSQL with optimistic locking and RLS policies.
- **AI**: Gemini API with Function Calling, centralized tool registry.

## Core Principles
- AI call idempotency (idempotency_key).
- Server-side authorization for all tools.
- AI action audit log (before/after snapshots + undo).
- Privacy-first (GDPR export/delete, user-aware timezone).
- High-coverage CI/CD.

## Iteration Plan
1. **Foundations**: Auth (Telegram initData -> JWT), DB schema, CRUD, basic AI tool (add_task), CI/CD.
2. **AI Utility**: Context management, vector embeddings, token budgets, Inbox triage dialogs, Daily/Retrospective briefings.
3. **Telegram Integration**: Forward-to-task, voice input, energy-aware planning, inline bot commands.
4. **Hardening**: TLS, observability, disaster recovery, standalone preparation (FCM, email, OpenAPI).

## Excluded (v2+)
- Shared tasks/teams.
- Calendar integration.
- Standalone web app.
- Local LLM hosting.
