# Agent Onboarding

You are a **Copilot agent**.

## How to Work
- **Ask questions before proceeding:** Please make sure to ask questions before coding to ensure mutual understanding.
- **Stop at confusion:** If you come across anything that confuses you, please stop and ask questions. If a file has changed since you last saw it, assume I made the change.
- **Show me the code:** I can read code. So when explaining things to me, please show me the code so we can be sure. 

## Principles

- **Above all, Clean Code is non-negotiable.** Utilize "Clean Code" principles, clear code is clear thinking.
- **KISS > DRY.** Clarity beats deduplication. DRY must not create indirection.
- **Consistency is king.** Check existing files for conventions and reusable patterns.
- **YAGNI.** Don't add layers for problems we don't have.
- **Write code to be self-explanatory by default.** 
- **Add comments only above non-obvious logic** to explain intent, constraints, invariants, risks, tradeoffs, or subtle edge cases - never to restate what the code is doing.
- **Minimize assumptions.** Validate with code or tests, not speculation.
- **Follow conventions.** Check existing files before inventing a style.
- **No premature abstraction.** Don't generalize without proven need.

## Opinions

- **No large files.** Keep every file below 400 lines.
- **Dead code is unacceptable.** Remove it unless structurally necessary (rare in Rust).
- **FOSS > SaaS.** Self-hosting maximalists. No closed-source services when open-source exists.
- **Leverage before adding.** Can existing tools (e.g. SQLite) solve this before adding a dependency?
- **Idempotency.** We like idempotent setups.
- **Cohesion > Coupling.** Related things together; unrelated things apart.
- **Transparency > Magic.** Explicit beats clever. No hidden behaviors.
- **No hack solutions.** Don't patch poor designs—rethink them. Never wrangle strings if you don't have to.

## Test-Driven Development

- Cover both positive and negative cases.
- Write descriptive, context-rich test names.
- Minimize duplication across test cases.
- Never write test code in production files. Test code must be confined to tests.
- **Data:** Prefer Faker; otherwise use fixture files.
- **Scope:** Favor broad functional tests over micro-unit tests.
- **No external calls.** Tests must not depend on network connectivity.
- **Tests must be fast:** No test should feel “expensive” to run.

