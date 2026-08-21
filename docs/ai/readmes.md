# Crate README Review Prompt

Review one publishable crate README against the crate's current code and
manifest. Update it only where doing so improves accuracy or helps a crate user.

## Sources

1. Read the existing README to understand its intended audience and avoid
   discarding useful explanations.
2. Treat the Rust source, `Cargo.toml`, tests, and runnable examples as the
   source of truth for behavior and public APIs.
3. Consult sibling READMEs only to place the crate at an integration boundary;
   do not copy their detailed responsibilities.

## Content

The README should answer, in this order:

1. What does this crate own?
2. When should a developer use it?
3. What is the smallest correct way to use it?
4. Which feature flags, lifecycle rules, persistence semantics, or failure
   modes would surprise a user?
5. Where should the reader go for adjacent concerns?

Keep the scope local to the crate. The main repository README owns the monorepo
overview; architecture documentation owns cross-crate data flow; each sibling
crate owns its own API details.

## Style

- Prefer a short, accurate guide over an exhaustive catalog of public types.
- Include code only when it is the clearest path to first use. Keep examples
  small and validate them with doctests or an existing runnable example.
- Link to the canonical sibling guide instead of re-explaining it.
- Use exact names from the current code and manifest.
- Avoid marketing adjectives, unmeasured performance claims, fixed generated
  counts, and version-specific dependency snippets unless they are maintained
  automatically.
- Distinguish current requirements from historical benchmark results.
- Do not state that the README was AI-generated.

## Validation

Before finishing:

- Verify every named type, method, feature, binary, environment variable, path,
  and command against the repository.
- Run the crate's doctests and relevant examples or tests when practical.
- Check relative links from the README's directory.
- Search for names removed or renamed in the current source.
- Re-read the result and remove material owned more clearly by another document.

If the existing README is already accurate, concise, and appropriately scoped,
leave it unchanged and report that outcome.
