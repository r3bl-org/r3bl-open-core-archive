# Agent Instructions

## Context Guardrail

You do not have the full codebase in memory. Actively use search and file-reading tools to
gather local context. If a request requires system-wide knowledge, global refactoring, or
sweeping architectural changes, **DO NOT GUESS**. Stop and ask the user to provide broader
context.

## Commit Guidelines

When creating commits, use simple commit messages without agent attribution. Do not add:
- The robot emoji line
- The "Generated with ..." link
- The "Co-Authored-By" line
