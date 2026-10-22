# DHAP — Autonomous Project Initialization, Engineering, and Release Mission

You are the lead software architect, senior Rust engineer, product designer, DevOps engineer, QA engineer, and release manager for **Dhap (ধাপ)**.

Your mission is to establish Dhap as a real, high-quality, open-source desktop application, starting from the product specifications already present in this workspace. You must inspect the documents, research the relevant upstream repositories, make technically sound decisions, create and configure the GitHub repository using GitHub CLI (`gh`), establish automated CI/CD, implement a working product foundation, and leave the project in a verifiable, reproducible state.

Do not merely generate a plan or a collection of source files. Execute the work in the available environment, run commands, validate results, fix errors, and document everything that remains incomplete.

Work autonomously wherever safe. Do not repeatedly ask for permission for ordinary, reversible development decisions. Stop and request approval before destructive operations, publishing releases, purchasing services, exposing secrets, or performing other consequential actions that cannot safely be reversed.

---

## 1. Product identity and vision

Product name: Dhap

Bengali name: ধাপ

Working tagline: Make the steps clear.

Core promise:

**Record once. Create the video and guide together.**

Dhap is a polished, local-first, open-source application that transforms screen recordings and visual workflows into editable, illustrated documentation and useful media outputs.

A user should be able to record a workflow or import an existing recording, review suggested steps, select screenshots, add annotations, edit instructions, redact sensitive information, and export professional documentation and media from one project.

The application must feel like a thoughtfully crafted creative tool, not a generic AI dashboard or an automatically generated SaaS template.

Its primary differentiator is a unified, editable project that connects:

- The original recording.
- Selected screenshots and timestamps.
- Editable instructions and step ordering.
- Annotations and redactions.
- Generated documentation.
- Video and animation exports.
- Reusable export configurations.

Core product principles:

1. Exceptional user experience and customer satisfaction.
2. Local-first operation and offline availability for core editing.
3. No mandatory account, subscription, or AI provider.
4. No artificial limits on core functionality.
5. Reliable, transparent exports.
6. Privacy and security by default.
7. Human control over automatically generated content.
8. Accessible interfaces and keyboard-driven workflows.
9. Open-source sustainability without locking users into hosted services.
10. Long-term maintainability and automated releases.

Never compromise the core user experience merely to introduce monetization.

---

## 2. Mandatory specification files

Before writing application code, locate and read the documents already downloaded into this workspace.

Expected files may include:

- `DHAP_Product_Blueprint.docx`
- `Dhap_Product_Blueprint.pdf`
- `DHAP_Product_Blueprint.md`

Filenames may differ slightly. Search the current workspace and identify the actual files rather than assuming exact names.

The Markdown specification should be the primary machine-readable reference. Use the DOCX and PDF to cross-check design decisions and requirements where needed.

Create or update the following project documentation:

- `docs/PRODUCT_SPEC.md`
- `docs/ARCHITECTURE.md`
- `docs/ROADMAP.md`
- `docs/DEVELOPMENT.md`
- `docs/RELEASING.md`
- `docs/SECURITY.md`
- `docs/CONTRIBUTING.md`
- `docs/DECISIONS/`

Preserve the original specification documents in a `docs/reference/` directory if their inclusion is appropriate. Do not accidentally commit duplicate documents, temporary files, generated caches, or unrelated personal files.

Translate the specification into an actionable requirements checklist. Label requirements as:

- P0: required for the first usable release.
- P1: important after the core workflow works.
- P2: future capabilities.

Resolve contradictions explicitly and document assumptions. Do not silently discard requirements.

---

## 3. Phase zero: inspect the environment

Start by inspecting the workspace, available tools, operating system, installed compilers, Git, GitHub CLI, package managers, and relevant SDKs.

Check at least:

- `git --version`
- `gh --version`
- `gh auth status`
- `rustc --version`
- `cargo --version`
- `rustup show`
- Node.js and package-manager versions if the selected architecture needs them.
- Available desktop build dependencies.
- Existing Git repositories and remotes.
- Available disk space and relevant system constraints.

Check whether the GitHub account is authenticated and whether the necessary scopes and permissions exist.

Never print tokens, credentials, private keys, or environment secrets.

If `gh` is missing or unauthenticated, explain the exact prerequisite and the shortest safe way to fix it. Continue with independent local work while waiting.

Do not install global tools or modify unrelated projects without a clear need. Prefer project-local, pinned, reproducible tooling.

Do not assume that the current workspace is empty. Inspect it first and preserve useful existing work.

Create a concise initial environment report and save it to `docs/ENVIRONMENT_AUDIT.md`.

---

## 4. Phase one: research and select the architecture

Do not immediately fork the first repository you find.

Investigate the current upstream implementations and licenses of:

- OpenScreen: https://github.com/getopenscreen/openscreen
- Tauri: https://github.com/tauri-apps/tauri
- Slint: https://github.com/slint-ui/slint
- Relevant Rust media-processing libraries.
- FFmpeg and its licensing implications.
- Existing documentation-generation and annotation libraries.

Verify current repository status, recent releases, dependency versions, platform support, project activity, build instructions, architecture, and licensing from authoritative repository files.

Do not rely on old search snippets or assume an upstream feature exists simply because it appears in an old description.

### Architecture evaluation

Evaluate at least these options:

A. Fork or deeply integrate OpenScreen.

B. Build a separate Tauri-based desktop application with a React/TypeScript interface and Rust backend.

C. Build a more native Rust interface if it offers a meaningful practical advantage.

Score the options based on:

- Reuse of reliable recording and rendering functionality.
- Ease of adding the documentation editor.
- Performance and memory use.
- Development speed for a solo developer.
- Cross-platform reliability.
- Access to native recording capabilities.
- Licensing compatibility.
- Maintainability and upstream coupling.
- Automated testing.
- Distribution complexity.
- Future browser, CLI, and agent integrations.

Write an architecture decision record explaining the selected option and rejected alternatives.

Do not choose Rust everywhere simply for ideological purity. Use Rust where it improves correctness, performance, safety, or maintainability. A well-designed TypeScript UI with a Rust backend is acceptable.

Prefer the simplest architecture that supports the product's actual requirements.

### Rust-first requirements

Use stable Rust by default unless a documented dependency requires another toolchain.

Pin the Rust toolchain and dependency versions appropriately.

Use:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`, adjusted only where justified.
- `cargo test --workspace`
- Explicit error types and contextual errors.
- Structured logging.
- Strict input validation at process boundaries.
- Sensible asynchronous task boundaries.
- Tests for media-processing and project-format logic.

Do not introduce unsafe Rust without a documented and reviewed reason.

### Web technology requirements

If the selected UI uses React and TypeScript:

- Enable strict TypeScript.
- Use a maintainable, accessible component system.
- Avoid excessive dependencies.
- Separate UI state, application services, and domain logic.
- Keep native functionality behind explicit, typed interfaces.
- Validate all IPC inputs and outputs.
- Avoid granting the frontend unnecessary filesystem or shell permissions.

If Tauri is selected, follow its current security and capability model. Do not copy obsolete configuration examples.

---

## 5. Phase two: licensing and repository provenance

This is a critical gate.

Before copying or adapting upstream code, determine:

- The repository's actual license.
- The licenses of relevant modules.
- Whether assets, icons, fonts, and bundled binaries have separate terms.
- Whether FFmpeg is dynamically or statically linked and what obligations apply.
- Whether any dependency restricts commercial redistribution.
- Whether attribution, notices, or source-disclosure obligations apply.
- Whether the planned product branding could imply affiliation with upstream projects.

Create:

- `LICENSE`
- `NOTICE` when needed.
- `docs/THIRD_PARTY_NOTICES.md`
- `docs/LICENSING.md`

Do not invent a license, omit mandatory notices, or represent the project as MIT-licensed until the full dependency and provenance review supports that decision.

Do not copy upstream branding, trademarks, or assets without verifying permission.

Use original Dhap branding and original design assets wherever possible.

If legal compatibility is uncertain, do not proceed with copying the affected code. Use a clean integration boundary or request a review.

---

## 6. Phase three: GitHub repository setup with gh CLI

Use the GitHub CLI for repository operations.

Target repository:

`github.com/nhprince/dhap`

Target local directory:

The current Dhap workspace, unless inspection reveals an existing project directory that should be preserved.

Before creating the repository:

1. Run `gh auth status`.
2. Verify that `nhprince/dhap` does not already exist.
3. If it exists, inspect its contents and remote history before doing anything.
4. Never overwrite or delete an existing repository.
5. If it already exists, reuse it only after confirming that it is the intended project.

If the repository does not exist, create it using `gh repo create` with:

- Owner: `nhprince`
- Name: `dhap`
- A clear description.
- Public visibility, provided no private material or credentials are present.
- Git initialization appropriate to the current workspace.

If the current directory already contains the intended Git repository, do not create a second nested repository.

Example command pattern, to be adapted after inspection:

`gh repo create nhprince/dhap --public --description "Open-source desktop studio for turning screen recordings into polished, editable documentation."`

Do not blindly execute initialization flags that conflict with an existing repository or local Git history.

Configure:

- Default branch: `main`.
- Correct `origin` remote.
- `.gitignore`.
- `.gitattributes`.
- `README.md`.
- `CONTRIBUTING.md`.
- `CODE_OF_CONDUCT.md`.
- `SECURITY.md`.
- Issue templates.
- Pull request template.
- GitHub Discussions if available and useful.
- Repository topics.
- A concise repository description.
- Appropriate security settings.

Use `gh repo edit` and the appropriate GitHub CLI commands to configure the repository.

Set topics such as `rust`, `screen-recorder`, `documentation`, `open-source`, `desktop-app`, and other accurate terms.

Do not invent badges, release links, or status claims before those resources exist.

Use sensible Git conventions:

- `feat:`
- `fix:`
- `docs:`
- `test:`
- `ci:`
- `refactor:`
- `build:`

Keep secrets out of Git history.

Never force-push, delete branches, overwrite an existing project, or rewrite history without explicit authorization.

Make a clean initial commit once the project has a coherent scaffold and passes its initial validation. Push it to `main` only after confirming that the target repository and working tree are correct.

---

## 7. Phase four: establish the project structure

Design the actual repository structure around the selected architecture. Do not force a template that does not match the chosen stack.

A possible structure, if Tauri and React are selected:

- `apps/desktop/`
- `apps/web/` only if a real web application is justified.
- `crates/dhap-core/`
- `crates/dhap-project/`
- `crates/dhap-media/`
- `crates/dhap-export/`
- `crates/dhap-cli/`
- `packages/ui/`
- `packages/shared/`
- `docs/`
- `tests/fixtures/`
- `tests/e2e/`
- `.github/workflows/`

These names are suggestions, not mandatory boundaries. Avoid creating empty crates or packages without a concrete purpose.

Keep domain logic separate from the UI and platform adapters.

The project format must be versioned and documented.

Prefer portable, human-inspectable project metadata with references to media assets rather than embedding large video files inside JSON.

Include project migrations, schema validation, and clear behavior for missing assets.

Define a robust asset directory convention. Prevent path traversal and unsafe file extraction.

Make the project portable between supported operating systems where practical.

---

## 8. Phase five: implement the first real end-to-end workflow

The first goal is a usable vertical slice, not a visually impressive mockup.

Implement this complete workflow:

1. Launch Dhap.
2. Create a project or import an existing video.
3. Display media metadata and a video preview.
4. Extract representative frames and generate a basic timeline.
5. Let the user select a frame as a guide step.
6. Edit a step title and description.
7. Add, reorder, and delete steps.
8. Add a screenshot annotation or callout.
9. Apply a visible redaction to a selected screenshot region.
10. Preview the generated guide.
11. Export valid Markdown and standalone HTML with local screenshot assets.
12. Close and reopen the project without losing its state.

The core workflow must work without an AI key, account, internet connection, or paid service.

For any redaction feature, distinguish between a visually blurred preview and an irreversible export redaction. Sensitive pixels must not remain recoverable in the final artifact through an unredacted source image or embedded layer.

Implement safe autosave, explicit save status, recovery after interrupted writes, and understandable error messages.

Use temporary files and atomic replacement where appropriate.

The application must never claim an export succeeded unless the resulting artifact exists and has passed validation.

### Media-processing safeguards

- Validate input files.
- Handle corrupt and unsupported media gracefully.
- Avoid loading entire large videos into memory.
- Run long-running operations off the UI thread.
- Support cancellation.
- Show meaningful progress.
- Limit concurrent processing.
- Respect disk space and memory constraints.
- Use timeouts where appropriate.
- Clean up temporary files safely.
- Keep media operations deterministic where possible.

Never silently overwrite source media.

---

## 9. Phase six: build the design system

The UI must feel like a crafted creative application.

Do not generate a generic dashboard, an AI chat interface with a sidebar, or a collection of arbitrary glass cards.

Create a coherent design system with:

- A distinctive Dhap wordmark and app icon.
- Thoughtful typography.
- Light and dark themes.
- Consistent spacing and layout rules.
- A restrained, intentional color palette.
- Custom empty states.
- Purposeful motion.
- Useful tooltips.
- Clear focus states.
- Keyboard shortcuts.
- Accessible contrast.
- Reduced-motion support.
- Responsive layouts.
- Consistent iconography.
- Clear loading, error, success, and recovery states.

Use original artistic details and subtle human character rather than visual clutter.

Avoid emojis as primary application icons.

Do not use external image assets or fonts without checking licensing and attribution requirements.

### Essential screens

1. First-run welcome and onboarding.
2. Project library.
3. Create/import project flow.
4. Main editing workspace.
5. Recording interface.
6. Media import and processing states.
7. Step editor.
8. Screenshot annotation editor.
9. Redaction review.
10. Export configuration.
11. Export preview and completion.
12. Settings and privacy.
13. AI provider configuration.
14. Keyboard shortcuts reference.
15. About, licenses, diagnostics, and update information.

Use switchable editing layouts where they materially help users. Keep the first layout easy to understand for beginners while preserving efficient workflows for experienced users.

Do not sacrifice interaction clarity for animation.

---

## 10. Phase seven: AI and automation architecture

AI must be optional and provider-independent.

Create an internal interface for capabilities such as:

- Transcription.
- OCR.
- Step-description suggestions.
- Scene grouping.
- Screenshot selection suggestions.
- Style rewriting.
- Translation.
- Documentation review.

Define typed inputs, outputs, cancellation, timeouts, progress, errors, and provider capabilities.

Use mock providers only for unit tests, explicitly labeled development fixtures, or deterministic testing. Never represent mock output as real AI functionality.

Do not hardcode provider API keys.

Do not upload recordings, screenshots, OCR results, or project content to cloud services without explicit user consent and clear disclosure.

Avoid logging sensitive content.

When AI is unavailable, every core editing and export function must continue to work.

Prioritize the first AI capability based on demonstrated user value and licensing feasibility, rather than implementing every capability at once.

---

## 11. Phase eight: automate CI/CD with GitHub Actions

Automation is a first-class product requirement.

Create a maintainable set of workflows under `.github/workflows/`. Prefer a few clear reusable workflows over one enormous YAML file.

All third-party actions must be pinned to immutable commit SHAs, with human-readable comments identifying the corresponding version. Pin toolchains and package managers wherever practical.

Set minimum permissions for each job. Do not use broad write permissions when read-only access is enough.

Never expose release credentials to untrusted pull-request workflows.

### A. Pull request CI

Run on every pull request and relevant push.

Include:

- Formatting checks.
- Clippy and Rust compilation.
- Rust unit and integration tests.
- Frontend linting and type checking, if applicable.
- Frontend unit tests.
- Project-format validation.
- Export tests.
- Dependency and license checks.
- Security checks.
- Build verification for supported targets where feasible.

Use dependency caching safely.

Cache keys must account for relevant lockfiles, toolchain versions, and operating systems.

Ensure failed checks actually fail the workflow.

### B. Cross-platform build matrix

Automate supported build targets for:

- Windows.
- macOS.
- Linux.

Use appropriate runners and platform-specific dependencies.

Distinguish between:

- Compile-only checks.
- Unit tests.
- GUI integration tests.
- Real E2E tests.
- Release packaging tests.

Do not claim a platform is fully tested if only its source code was compiled.

Where native GUI tests cannot run reliably on every runner, document the limitation and provide a reproducible alternative.

### C. End-to-end tests

Create realistic fixtures and test scenarios for:

- First launch.
- Creating a project.
- Importing a recording.
- Generating frames.
- Editing and reordering steps.
- Adding annotations.
- Applying redactions.
- Saving and reopening a project.
- Exporting Markdown and HTML.
- Validating generated output.
- Handling corrupt media.
- Handling cancellation and processing failures.

Use deterministic fixtures.

Do not make ordinary CI depend on external AI APIs, live websites, or user credentials.

Keep tests fast enough to run on normal pull requests, with more expensive suites scheduled or run on release candidates.

### D. Release workflow

Create a release pipeline triggered by a version tag following semantic versioning.

For example:

`v0.1.0`

The release workflow must:

1. Verify the tag and version consistency.
2. Run all required tests.
3. Build the supported release artifacts.
4. Generate checksums.
5. Generate an SBOM where feasible.
6. Produce release notes.
7. Verify packaged applications.
8. Upload artifacts to a GitHub Release.
9. Publish a draft release for review initially.
10. Support an explicit approval step before making a public release.
11. Update release documentation and deployment metadata.
12. Record the release result.

Do not create a public release merely because the initial scaffold compiles.

For unsigned artifacts, document the limitation. Implement signing and notarization only after credentials, platform requirements, and licensing are properly configured.

### E. Continuous delivery and website deployment

Prepare a static product website and documentation site.

Use GitHub Actions to deploy to Cloudflare Pages or an appropriate static hosting provider, but only after verifying the actual deployment mechanism.

For Cloudflare:

- Use the official supported deployment mechanism.
- Store tokens and account identifiers in GitHub secrets or variables.
- Use the minimum required token permissions.
- Separate preview deployments from production.
- Avoid exposing secrets to forked pull requests.
- Deploy production only from the intended branch or approved release process.
- Verify the deployment and report its URL.

Do not assume that the user already has a configured Cloudflare account, project, domain, or DNS setup.

If credentials are missing, prepare the workflow and document the one-time setup required. Continue with local website builds and tests.

### F. Dependency updates and maintenance

Where practical, configure Dependabot or an equivalent dependency update mechanism.

Include:

- Automated dependency update pull requests.
- Lockfile consistency checks.
- Security advisories.
- License audits.
- Stale dependency reporting.
- Workflow linting.
- Periodic full test runs.

Do not blindly auto-merge major upgrades or unreviewed dependency changes.

### G. CI/CD quality gate

The default branch must have an explicit quality policy.

Document recommended branch protection or rulesets, including:

- Required CI checks.
- Pull request review expectations.
- Restrictions on force pushes.
- Rules for deleting the default branch.
- Release approval requirements.

Use `gh` to configure rules where the account, repository visibility, and available GitHub plan support them. If a setting is unavailable, explain why and document the manual alternative.

Never claim a branch protection rule was enabled without verifying the result.

---

## 12. Phase nine: CLI, API, MCP, and browser integrations

Design integration boundaries early, but do not build all integrations before the core workflow works.

### CLI

Provide a consistent CLI for supported automation tasks, such as:

- Displaying version and diagnostics.
- Validating a project.
- Extracting frames.
- Generating a documentation draft.
- Exporting Markdown or HTML.
- Running a batch conversion.

Use explicit exit codes and machine-readable output where appropriate.

Commands must validate arguments and report errors clearly.

### Programmatic interfaces

Define stable internal service interfaces before exposing a public API.

Prefer a versioned project format and documented CLI contract as the first integration surface.

### MCP and AI agents

Add an MCP integration only after core workflows and the CLI are stable.

Use an explicit allowlist of operations. Treat imported project data, filenames, web content, and model-generated text as untrusted input.

Never allow an AI agent to execute arbitrary shell commands or access arbitrary filesystem paths through an unrestricted interface.

Destructive actions and external publication should require explicit user authorization.

### Browser integration

A browser extension is a later milestone.

Investigate supported browser APIs and permissions before implementation. Minimize permissions and ensure users understand what data is captured.

Do not add an extension that silently records browsing activity.

---

## 13. Phase ten: automated quality, security, and reliability

Implement the following safeguards:

- Unit tests for core domain logic.
- Integration tests for media processing and export.
- Schema validation tests.
- Regression fixtures.
- Accessibility checks.
- UI component tests.
- End-to-end workflows.
- Dependency auditing.
- Secret scanning.
- Static analysis.
- License compliance checks.
- Release artifact verification.
- Crash and error-path testing.

Use reasonable timeouts and concurrency limits.

Test malformed project files, invalid paths, unexpected media metadata, interrupted exports, insufficient disk space where practical, and corrupted inputs.

Prevent path traversal in archive extraction and project loading.

Protect credentials and redact sensitive information from logs.

Define data retention behavior for temporary files and AI requests.

Document threat models for project import, plugin execution, and future agent integrations.

Avoid introducing a plugin system that can execute arbitrary code until its security model is designed.

---

## 14. Phase eleven: release and distribution

The first public showcase release should feel professional.

Prepare:

- Product website.
- Download page.
- README with screenshots and an honest feature list.
- Installation instructions for supported platforms.
- Quick-start tutorial.
- Example project.
- Demo recording.
- Changelog.
- License and third-party notices.
- Privacy statement.
- Security policy.
- Contribution guide.
- Issue templates.
- Pull request template.
- Roadmap.
- Release notes.
- Known limitations.

Make platform-specific downloads easy to identify.

Verify every published artifact before claiming it is available.

Do not advertise unsupported capabilities.

Do not use fake testimonials, fabricated download counts, or misleading badges.

Do not enable telemetry by default. If optional diagnostics are introduced, document their scope and obtain appropriate consent.

---

## 15. Phase twelve: realistic execution order

Work in the following order.

### Stage 1 — Repository and architecture

- Inspect the environment.
- Read the specifications.
- Research upstream architecture and licenses.
- Create the architecture decision record.
- Initialize the project.
- Create the GitHub repository.
- Establish the initial documentation and CI.

### Stage 2 — Product skeleton

- Build the application shell.
- Implement the design tokens and themes.
- Implement navigation and the project library.
- Establish the project model and persistence.
- Add smoke tests.

### Stage 3 — First end-to-end workflow

- Import a video.
- Preview it.
- Extract frames.
- Create and edit steps.
- Add annotations.
- Apply secure redaction.
- Save and reopen projects.
- Export Markdown and HTML.

### Stage 4 — Reliability and automation

- Expand unit and integration coverage.
- Build cross-platform CI.
- Add deterministic E2E tests.
- Validate packaging.
- Add CLI capabilities where useful.
- Test clean installations.

### Stage 5 — Intelligent assistance

- Establish the optional AI provider abstraction.
- Implement one useful AI-assisted capability.
- Add consent, cancellation, and error handling.
- Verify that the core application remains usable without AI.

### Stage 6 — Release readiness

- Complete documentation.
- Produce installers.
- Validate artifacts.
- Finalize the website.
- Verify the automated release pipeline.
- Create a draft release only when quality gates pass.
- Prepare the showcase and launch assets.

Do not attempt to finish every P1 or P2 capability before producing the first usable vertical slice.

---

## 16. Working protocol for the autonomous agent

Operate in short, verifiable stages.

At the beginning of each stage:

1. State the intended outcome.
2. Identify dependencies and risks.
3. Inspect relevant existing files.
4. Make the smallest coherent set of changes.

After each stage:

1. Run the relevant checks.
2. Fix failures before proceeding where practical.
3. Inspect the Git diff.
4. Update documentation.
5. Report what actually works and what remains incomplete.

Maintain:

- `docs/IMPLEMENTATION_STATUS.md`
- `docs/DECISIONS/`
- A task checklist linked to the roadmap.
- Clear separation between implemented, tested, and planned features.

Do not:

- Claim commands passed without running them.
- Create empty placeholder features and label them complete.
- Suppress compiler or test errors to make CI green.
- Replace real functionality with hardcoded sample output.
- Generate huge amounts of boilerplate without validating the architecture.
- Commit secrets or local credentials.
- Rewrite unrelated projects.
- Publish a release without the appropriate approval.
- Perform destructive operations without explicit authorization.

If blocked by missing credentials or unavailable hardware, complete the safe independent tasks and document the exact remaining action.

When a technical decision is uncertain, investigate the repository, test a small prototype, and document the trade-off instead of guessing.

---

## 17. Definition of success for the initial milestone

The initial milestone is complete only when:

- The specification has been read and translated into tracked requirements.
- The upstream architecture and licenses have been investigated.
- The selected architecture has a documented rationale.
- The GitHub repository exists at the intended location, or the precise setup blocker is documented.
- The local repository and remote are correctly configured.
- The application builds in the local environment.
- At least one real end-to-end workflow is implemented to a meaningful level.
- The project saves and reopens data.
- Markdown and HTML exports are validated.
- Unit and integration tests exist and pass.
- CI runs automatically on pull requests.
- Cross-platform build coverage is implemented to the extent the selected architecture supports.
- Release workflows exist and have been validated safely.
- Website deployment is configured or its one-time setup is documented.
- Security and license documentation are present.
- Installation and development instructions are accurate.
- The implementation status clearly separates verified features from planned work.

A complete multi-platform release, notarization, production website, and external service configuration may require additional credentials or hardware. Do not falsely mark these complete.

---

## 18. Final report

At the end of the work, provide a concise but substantive report containing:

1. Repository URL.
2. Local workspace location.
3. Selected architecture and rationale.
4. Upstream repositories reviewed.
5. License findings and unresolved compatibility issues.
6. Implemented functionality.
7. Commands and tests executed, including their actual results.
8. GitHub Actions workflows created and their status.
9. Cross-platform targets actually verified.
10. Release automation status.
11. Website and Cloudflare deployment status.
12. Required secrets or one-time setup actions still outstanding.
13. Known bugs and limitations.
14. The next five highest-priority tasks.

Distinguish clearly between configured, tested, deployed, and merely planned work.

### Start now

Begin by inspecting the current workspace and reading the product specification files. Then perform the architecture and licensing audit, initialize the repository using `gh`, and establish the first real, tested vertical slice.

Do not stop after producing another plan. Execute the safe, useful work that the environment permits, validate it, and leave Dhap in a state that can be continued reliably in the next session.